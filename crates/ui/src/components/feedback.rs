use crate::components::animation;
use crate::DbProTheme;
use egui::{Color32, Frame, Margin, Pos2, Rect, RichText, Rounding, Stroke, Ui, Vec2};

pub struct Progress {
    pub(crate) fraction: f32, // 0.0 to 1.0
    pub(crate) height: f32,
    pub(crate) animated: bool,
    pub(crate) indeterminate: bool,
    pub(crate) theme: DbProTheme,
}

impl Progress {
    pub fn new(fraction: f32, theme: DbProTheme) -> Self {
        Self {
            fraction: fraction.clamp(0.0, 1.0),
            height: 6.0,
            animated: true,
            indeterminate: false,
            theme,
        }
    }

    pub fn indeterminate(theme: DbProTheme) -> Self {
        Self {
            fraction: 0.0,
            height: 6.0,
            animated: true,
            indeterminate: true,
            theme,
        }
    }

    pub fn height(mut self, height: f32) -> Self {
        self.height = height;
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
        let rounding = Rounding::same(self.height * 0.5);

        ui.painter().rect_filled(rect, rounding, self.theme.surface_hover);

        if self.indeterminate {
            let (tail, head) = animation::indeterminate_beam(ui);
            let x_start = rect.left() + rect.width() * tail;
            let x_end = rect.left() + rect.width() * head;
            let beam_w = (x_end - x_start).max(12.0);
            let beam_rect = Rect::from_min_size(Pos2::new(x_start, rect.top()), Vec2::new(beam_w, self.height));
            ui.painter().rect_filled(beam_rect, rounding, self.theme.accent);
            return 0.0;
        }

        let target_fraction = self.fraction;
        let display_fraction = if self.animated {
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
}

impl Spinner {
    pub fn new(theme: DbProTheme) -> Self {
        Self {
            size: 18.0,
            color: None,
            theme,
        }
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
        let (rect, _) = ui.allocate_exact_size(Vec2::splat(self.size), egui::Sense::hover());
        let center = rect.center();
        let radius = (self.size - 3.0) * 0.5;
        let color = self.color.unwrap_or(self.theme.accent);

        animation::paint_spinner(
            ui.painter(),
            center,
            radius,
            2.0,
            color,
            self.theme.border_subtle,
            animation::spinner_angle(ui),
        );
    }
}

pub fn kbd_badge(ui: &mut Ui, shortcut: &str, theme: DbProTheme) {
    Frame {
        fill: theme.surface_elevated,
        stroke: Stroke::new(1.0, theme.border_default),
        inner_margin: Margin::symmetric(6.0, 2.5),
        rounding: Rounding::same(4.0),
        shadow: egui::epaint::Shadow {
            offset: egui::vec2(0.0, 1.0),
            blur: 0.0,
            spread: 0.0,
            color: theme.border_strong,
        },
        ..Default::default()
    }
    .show(ui, |ui| {
        ui.label(
            RichText::new(shortcut)
                .font(crate::DbProTheme::ui_medium_font(11.0))
                .color(theme.text_primary),
        );
    });
}

pub fn kbd_combo(ui: &mut Ui, keys: &[&str], theme: DbProTheme) {
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing = Vec2::new(3.0, 0.0);
        for (i, key) in keys.iter().enumerate() {
            if i > 0 {
                ui.label(RichText::new("+").size(10.0).color(theme.text_muted));
            }
            kbd_badge(ui, key, theme);
        }
    });
}

pub fn separator_with_text(ui: &mut Ui, text: &str, theme: DbProTheme) {
    ui.horizontal(|ui| {
        let text_layout =
            ui.painter()
                .layout_no_wrap(text.to_owned(), egui::FontId::proportional(11.0), theme.text_muted);
        let text_width = text_layout.size().x;
        let total_width = ui.available_width();
        let line_width = ((total_width - text_width - 16.0) * 0.5).max(10.0);

        let (rect_left, _) = ui.allocate_exact_size(Vec2::new(line_width, 14.0), egui::Sense::hover());
        ui.painter().hline(
            rect_left.x_range(),
            rect_left.center().y,
            Stroke::new(1.0, theme.border_default),
        );

        ui.label(RichText::new(text).size(11.0).color(theme.text_muted));

        let (rect_right, _) = ui.allocate_exact_size(Vec2::new(line_width, 14.0), egui::Sense::hover());
        ui.painter().hline(
            rect_right.x_range(),
            rect_right.center().y,
            Stroke::new(1.0, theme.border_default),
        );
    });
}
