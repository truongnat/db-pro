use super::config::*;
use super::handler::*;
use crate::components::animation;
use crate::tokens::{RADIUS_XS, STROKE_THICK, STROKE_THIN};
use crate::DbProTheme;
use egui::{Color32, CornerRadius, Frame, Margin, Pos2, Rect, RichText, Stroke, Ui, Vec2};

pub struct Progress {
    pub(crate) fraction: f32, // 0.0 to 1.0
    pub(crate) height: f32,
    pub(crate) animated: bool,
    pub(crate) indeterminate: bool,
    pub(crate) theme: DbProTheme,
    pub(crate) accessible_label: String,
}

impl Progress {
    pub fn new(fraction: f32, theme: DbProTheme) -> Self {
        Self {
            fraction: normalize_progress_fraction(fraction),
            height: PROGRESS_DEFAULT_HEIGHT,
            animated: true,
            indeterminate: false,
            theme,
            accessible_label: "Progress".to_owned(),
        }
    }

    pub fn indeterminate(theme: DbProTheme) -> Self {
        Self {
            fraction: 0.0,
            height: PROGRESS_DEFAULT_HEIGHT,
            animated: true,
            indeterminate: true,
            theme,
            accessible_label: "Loading".to_owned(),
        }
    }

    /// Sets the accessibility label without changing the visual API.
    pub fn label(mut self, label: impl Into<String>) -> Self {
        self.accessible_label = label.into();
        self
    }

    pub fn height(mut self, height: f32) -> Self {
        self.height = normalize_progress_height(height);
        self
    }

    pub fn animated(mut self, animated: bool) -> Self {
        self.animated = animated;
        self
    }

    pub fn is_indeterminate(mut self, indet: bool) -> Self {
        self.indeterminate = indet;
        self
    }

    /// Paints the bar and returns the fill fraction actually drawn (`0.0` when indeterminate).
    pub fn show(self, ui: &mut Ui) -> f32 {
        let width = ui.available_width();
        let (rect, response) = ui.allocate_exact_size(Vec2::new(width, self.height), egui::Sense::hover());
        response.widget_info(|| {
            progress_indicator_info(
                self.fraction,
                self.indeterminate,
                &self.accessible_label,
                ui.is_enabled(),
            )
        });
        let rounding = CornerRadius::same((self.height * 0.5) as u8);

        ui.painter().rect_filled(rect, rounding, self.theme.surface_hover);

        if self.indeterminate {
            // Reduced motion keeps a visible busy indicator without scheduling a moving beam.
            let (tail, head) = if animation::should_animate(true, self.theme.reduce_motion) {
                animation::indeterminate_beam(ui)
            } else {
                (PROGRESS_STATIC_BEAM_TAIL, PROGRESS_STATIC_BEAM_HEAD)
            };
            let (x_start, beam_w) = calculate_beam_geometry(rect.left(), rect.width(), tail, head);
            let beam_rect = Rect::from_min_size(Pos2::new(x_start, rect.top()), Vec2::new(beam_w, self.height));
            ui.painter().rect_filled(beam_rect, rounding, self.theme.accent);
            return 0.0;
        }

        let target_fraction = self.fraction;
        let display_fraction = if animation::should_animate(self.animated, self.theme.reduce_motion) {
            ui.ctx().animate_value_with_time(
                response.id.with("progress_fraction_smooth"),
                target_fraction,
                animation::PROGRESS_LERP_SECS,
            )
        } else {
            target_fraction
        };

        if display_fraction > 0.001 {
            let fill_width = (rect.width() * display_fraction).min(rect.width());
            let fill_rect = Rect::from_min_size(rect.left_top(), Vec2::new(fill_width, self.height));
            ui.painter().rect_filled(fill_rect, rounding, self.theme.accent);
        }

        display_fraction
    }
}

pub struct Spinner {
    size: f32,
    color: Option<Color32>,
    theme: DbProTheme,
    accessible_label: String,
}

impl Spinner {
    pub fn new(theme: DbProTheme) -> Self {
        Self {
            size: SPINNER_DEFAULT_SIZE,
            color: None,
            theme,
            accessible_label: "Loading".to_owned(),
        }
    }

    /// Sets the accessibility label without changing the visual API.
    pub fn label(mut self, label: impl Into<String>) -> Self {
        self.accessible_label = label.into();
        self
    }

    pub fn size(mut self, size: f32) -> Self {
        self.size = size;
        self
    }

    pub fn color(mut self, color: Color32) -> Self {
        self.color = Some(color);
        self
    }

    pub fn show(self, ui: &mut Ui) {
        let size = if self.size.is_finite() {
            self.size.max(SPINNER_MIN_SIZE)
        } else {
            SPINNER_MIN_SIZE
        };
        let (rect, response) = ui.allocate_exact_size(Vec2::splat(size), egui::Sense::hover());
        response.widget_info(|| progress_indicator_info(0.0, true, &self.accessible_label, ui.is_enabled()));
        let center = rect.center();
        let radius = calculate_spinner_radius(size);
        let color = self.color.unwrap_or(self.theme.accent);

        animation::paint_spinner(
            ui.painter(),
            center,
            radius,
            STROKE_THICK,
            color,
            self.theme.border_subtle,
            if animation::should_animate(true, self.theme.reduce_motion) {
                animation::spinner_angle(ui)
            } else {
                0.0
            },
        );
    }
}

pub fn kbd_badge(ui: &mut Ui, shortcut: &str, theme: DbProTheme) {
    Frame {
        fill: theme.surface_elevated,
        stroke: Stroke::new(STROKE_THIN, theme.border_default),
        inner_margin: Margin::symmetric(KBD_PAD_X as i8, KBD_PAD_Y as i8),
        corner_radius: CornerRadius::same(RADIUS_XS as u8),
        shadow: egui::epaint::Shadow {
            offset: [0, 1],
            blur: 0,
            spread: 0,
            color: theme.border_strong,
        },
        ..Default::default()
    }
    .show(ui, |ui| {
        ui.label(
            RichText::new(shortcut)
                .font(DbProTheme::ui_medium_font(KBD_FONT_SIZE))
                .color(theme.text_primary),
        );
    });
}

pub fn kbd_combo(ui: &mut Ui, keys: &[&str], theme: DbProTheme) {
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing = Vec2::new(KBD_COMBO_GAP, 0.0);
        for (i, key) in keys.iter().enumerate() {
            if i > 0 {
                ui.label(RichText::new("+").size(KBD_PLUS_SIZE).color(theme.text_muted));
            }
            kbd_badge(ui, key, theme);
        }
    });
}

pub fn separator_with_text(ui: &mut Ui, text: &str, theme: DbProTheme) {
    ui.horizontal(|ui| {
        let text_layout = ui.painter().layout_no_wrap(
            text.to_owned(),
            egui::FontId::proportional(SEPARATOR_TEXT_SIZE),
            theme.text_muted,
        );
        let text_width = text_layout.size().x;
        let total_width = ui.available_width();
        let line_width = calculate_separator_line_width(total_width, text_width);

        let (rect_left, _) = ui.allocate_exact_size(Vec2::new(line_width, SEPARATOR_HEIGHT), egui::Sense::hover());
        ui.painter().hline(
            rect_left.x_range(),
            rect_left.center().y,
            Stroke::new(STROKE_THIN, theme.border_default),
        );

        ui.label(RichText::new(text).size(SEPARATOR_TEXT_SIZE).color(theme.text_muted));

        let (rect_right, _) = ui.allocate_exact_size(Vec2::new(line_width, SEPARATOR_HEIGHT), egui::Sense::hover());
        ui.painter().hline(
            rect_right.x_range(),
            rect_right.center().y,
            Stroke::new(STROKE_THIN, theme.border_default),
        );
    });
}
