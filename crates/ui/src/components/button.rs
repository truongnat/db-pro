use crate::components::animation::{self, hover_t, lerp_color, press_scale, press_t};
use crate::components::interact::{button_info, paint_focus_ring};
use crate::components::overlay::Tooltip;
use crate::DbProTheme;
use egui::{
    text::{LayoutJob, TextFormat},
    Color32, FontFamily, FontId, Pos2, Rect, Response, Rounding, Sense, Stroke, Ui, Vec2,
};
use lucide_icons::Icon;

const BUTTON_ROUNDING: f32 = 6.0;
const ICON_TEXT_GAP: f32 = 8.0;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ButtonVariant {
    Default,
    Secondary,
    Outline,
    Ghost,
    Destructive,
    Link,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ButtonSize {
    Sm,
    Default,
    Lg,
    Icon,
    IconSm,
}

pub struct Button {
    pub(crate) label: Option<String>,
    pub(crate) icon: Option<Icon>,
    pub(crate) variant: ButtonVariant,
    pub(crate) size: ButtonSize,
    pub(crate) enabled: bool,
    pub(crate) loading: bool,
    pub(crate) full_width: bool,
    pub(crate) access_label: Option<String>,
    pub(crate) tooltip: Option<String>,
    pub(crate) theme: DbProTheme,
}

struct SizeTokens {
    min_height: f32,
    font_size: f32,
    icon_size: f32,
    padding: Vec2,
    default_width: f32,
}

impl Button {
    pub fn new(theme: DbProTheme) -> Self {
        Self {
            label: None,
            icon: None,
            variant: ButtonVariant::Default,
            size: ButtonSize::Default,
            enabled: true,
            loading: false,
            full_width: false,
            access_label: None,
            tooltip: None,
            theme,
        }
    }

    pub fn text(mut self, text: impl Into<String>) -> Self {
        self.label = Some(text.into());
        self
    }

    pub fn tooltip(mut self, tooltip: impl Into<String>) -> Self {
        self.tooltip = Some(tooltip.into());
        self
    }

    pub fn icon(mut self, icon: Icon) -> Self {
        self.icon = Some(icon);
        self
    }

    pub fn variant(mut self, variant: ButtonVariant) -> Self {
        self.variant = variant;
        self
    }

    pub fn size(mut self, size: ButtonSize) -> Self {
        self.size = size;
        self
    }

    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    pub fn loading(mut self, loading: bool) -> Self {
        self.loading = loading;
        self
    }

    pub fn full_width(mut self, full_width: bool) -> Self {
        self.full_width = full_width;
        self
    }

    pub fn access_label(mut self, label: impl Into<String>) -> Self {
        self.access_label = Some(label.into());
        self
    }

    fn size_tokens(&self) -> SizeTokens {
        match self.size {
            ButtonSize::Sm => SizeTokens {
                min_height: 26.0,
                font_size: 11.5,
                icon_size: 13.0,
                padding: egui::vec2(8.0, 3.0),
                default_width: 0.0,
            },
            ButtonSize::Default => SizeTokens {
                min_height: 32.0,
                font_size: 13.0,
                icon_size: 14.5,
                padding: egui::vec2(12.0, 5.0),
                default_width: 0.0,
            },
            ButtonSize::Lg => SizeTokens {
                min_height: 38.0,
                font_size: 14.5,
                icon_size: 16.0,
                padding: egui::vec2(16.0, 7.0),
                default_width: 0.0,
            },
            ButtonSize::Icon => SizeTokens {
                min_height: 32.0,
                font_size: 13.0,
                icon_size: 16.0,
                padding: egui::vec2(6.0, 6.0),
                default_width: 32.0,
            },
            ButtonSize::IconSm => SizeTokens {
                min_height: 24.0,
                font_size: 11.5,
                icon_size: 13.0,
                padding: egui::vec2(4.0, 4.0),
                default_width: 24.0,
            },
        }
    }

    fn accessible_name(&self) -> String {
        self.access_label
            .clone()
            .or_else(|| self.label.clone())
            .unwrap_or_else(|| "Button".to_owned())
    }

    pub fn show(self, ui: &mut Ui) -> Response {
        let tokens = self.size_tokens();
        if self.loading {
            return self.show_loading(ui, &tokens);
        }
        self.show_interactive(ui, &tokens)
    }

    fn show_loading(self, ui: &mut Ui, tokens: &SizeTokens) -> Response {
        let (text_color, fill_color, border_stroke) = rest_colors(self.variant, self.theme);
        let rounding = Rounding::same(BUTTON_ROUNDING);
        let text_galley = self.label.as_ref().map(|txt| {
            ui.painter()
                .layout_no_wrap(txt.clone(), FontId::proportional(tokens.font_size), text_color)
        });
        let gap = if text_galley.is_some() { ICON_TEXT_GAP } else { 0.0 };
        let text_w = text_galley.as_ref().map_or(0.0, |g| g.size().x);
        let content_w = tokens.icon_size + gap + text_w;
        let width = if self.full_width {
            ui.available_width()
        } else {
            tokens.default_width.max(content_w + tokens.padding.x * 2.0)
        };
        let (rect, response) = ui.allocate_exact_size(Vec2::new(width, tokens.min_height), Sense::hover());
        response.widget_info(|| button_info(false, &self.accessible_name()));
        ui.painter().rect_filled(rect, rounding, fill_color);
        if border_stroke != Stroke::NONE {
            ui.painter().rect_stroke(rect, rounding, border_stroke);
        }
        let start_x = rect.center().x - content_w * 0.5;
        animation::paint_spinner(
            ui.painter(),
            Pos2::new(start_x + tokens.icon_size * 0.5, rect.center().y),
            (tokens.icon_size - 2.0) * 0.5,
            1.8,
            text_color,
            text_color.linear_multiply(0.25),
            animation::spinner_angle(ui),
        );
        if let Some(galley) = text_galley {
            let text_pos = Pos2::new(
                start_x + tokens.icon_size + gap,
                rect.center().y - galley.size().y * 0.5,
            );
            ui.painter().galley(text_pos, galley, Color32::PLACEHOLDER);
        }
        response.on_hover_cursor(egui::CursorIcon::Wait)
    }

    fn show_interactive(self, ui: &mut Ui, tokens: &SizeTokens) -> Response {
        let name = self.accessible_name();
        let job = content_job(&self, tokens);
        let galley = ui.fonts(|fonts| fonts.layout_job(job));
        let width = if self.full_width {
            ui.available_width()
        } else {
            tokens.default_width.max(galley.size().x + tokens.padding.x * 2.0)
        };
        let sense = if self.enabled { Sense::click() } else { Sense::hover() };
        let (rect, mut response) = ui.allocate_exact_size(Vec2::new(width, tokens.min_height), sense);
        response.widget_info(|| button_info(self.enabled, &name));
        if self.enabled {
            response = response.on_hover_cursor(egui::CursorIcon::PointingHand);
        }

        let hover = hover_t(
            ui.ctx(),
            response.id.with("hover"),
            self.enabled && (response.hovered() || response.has_focus()),
        );
        let press = press_t(
            ui.ctx(),
            response.id.with("press"),
            self.enabled && response.is_pointer_button_down_on(),
        );
        let (rest_fill, hover_fill, rest_stroke, hover_stroke, text_color) =
            interactive_colors(self.variant, self.theme);
        let fill = lerp_color(rest_fill, hover_fill, hover);
        let stroke_color = lerp_color(rest_stroke.color, hover_stroke.color, hover);
        let stroke = if rest_stroke == Stroke::NONE && hover_stroke == Stroke::NONE {
            Stroke::NONE
        } else {
            Stroke::new(1.0, stroke_color)
        };

        let scale = press_scale(press);
        let paint_rect = Rect::from_center_size(rect.center(), rect.size() * scale);
        let rounding = Rounding::same(BUTTON_ROUNDING);
        ui.painter().rect_filled(paint_rect, rounding, fill);
        if stroke != Stroke::NONE {
            ui.painter().rect_stroke(paint_rect, rounding, stroke);
        }

        let text_pos = Pos2::new(
            paint_rect.center().x - galley.size().x * 0.5,
            paint_rect.center().y - galley.size().y * 0.5,
        );
        ui.painter().galley(text_pos, galley, text_color);

        if response.has_focus() {
            paint_focus_ring(ui, rect, BUTTON_ROUNDING, self.theme);
        }

        if let Some(ref tooltip_text) = self.tooltip {
            response = Tooltip::new(tooltip_text, self.theme).show(&response);
        }

        response
    }
}

fn content_job(button: &Button, tokens: &SizeTokens) -> LayoutJob {
    let (_, _, _, _, text_color) = interactive_colors(button.variant, button.theme);
    let mut job = LayoutJob::default();
    if let Some(icon) = button.icon {
        job.append(
            &char::from(icon).to_string(),
            0.0,
            TextFormat {
                font_id: FontId::new(tokens.icon_size, FontFamily::Name("lucide".into())),
                color: text_color,
                ..Default::default()
            },
        );
        if button.label.is_some() {
            job.append("  ", 0.0, TextFormat::default());
        }
    }
    if let Some(ref text) = button.label {
        job.append(
            text,
            0.0,
            TextFormat {
                font_id: FontId::proportional(tokens.font_size),
                color: text_color,
                ..Default::default()
            },
        );
    }
    job
}

fn rest_colors(variant: ButtonVariant, theme: DbProTheme) -> (Color32, Color32, Stroke) {
    match variant {
        ButtonVariant::Default => (theme.accent_foreground, theme.accent, Stroke::NONE),
        ButtonVariant::Secondary => (theme.text_primary, theme.surface_hover, Stroke::NONE),
        ButtonVariant::Outline => (
            theme.text_primary,
            Color32::TRANSPARENT,
            Stroke::new(1.0, theme.border_default),
        ),
        ButtonVariant::Ghost => (theme.text_secondary, Color32::TRANSPARENT, Stroke::NONE),
        ButtonVariant::Destructive => (theme.text_inverse, theme.danger, Stroke::NONE),
        ButtonVariant::Link => (theme.accent, Color32::TRANSPARENT, Stroke::NONE),
    }
}

fn interactive_colors(variant: ButtonVariant, theme: DbProTheme) -> (Color32, Color32, Stroke, Stroke, Color32) {
    match variant {
        ButtonVariant::Default => (
            theme.accent,
            theme.accent_hover,
            Stroke::NONE,
            Stroke::NONE,
            theme.accent_foreground,
        ),
        ButtonVariant::Secondary => (
            theme.surface_hover,
            theme.surface_active,
            Stroke::NONE,
            Stroke::NONE,
            theme.text_primary,
        ),
        ButtonVariant::Outline => (
            Color32::TRANSPARENT,
            theme.surface_hover,
            Stroke::new(1.0, theme.border_default),
            Stroke::new(1.0, theme.border_strong),
            theme.text_primary,
        ),
        ButtonVariant::Ghost => (
            Color32::TRANSPARENT,
            theme.surface_hover,
            Stroke::NONE,
            Stroke::new(1.0, theme.border_subtle),
            theme.text_secondary,
        ),
        ButtonVariant::Destructive => (
            theme.danger,
            theme.danger.linear_multiply(0.85),
            Stroke::NONE,
            Stroke::NONE,
            theme.text_inverse,
        ),
        ButtonVariant::Link => (
            Color32::TRANSPARENT,
            Color32::TRANSPARENT,
            Stroke::NONE,
            Stroke::NONE,
            theme.accent,
        ),
    }
}
