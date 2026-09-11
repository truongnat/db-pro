use crate::DbProTheme;
use egui::{
    text::{LayoutJob, TextFormat},
    Button, Color32, FontFamily, FontId, Response, Rounding, Stroke, Ui, Vec2,
};
use lucide_icons::Icon;

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

pub struct ShadcnButton {
    pub(crate) label: Option<String>,
    pub(crate) icon: Option<Icon>,
    pub(crate) variant: ButtonVariant,
    pub(crate) size: ButtonSize,
    pub(crate) enabled: bool,
    pub(crate) loading: bool,
    pub(crate) full_width: bool,
    pub(crate) theme: DbProTheme,
}

impl ShadcnButton {
    pub fn new(theme: DbProTheme) -> Self {
        Self {
            label: None,
            icon: None,
            variant: ButtonVariant::Default,
            size: ButtonSize::Default,
            enabled: true,
            loading: false,
            full_width: false,
            theme,
        }
    }

    pub fn text(mut self, text: impl Into<String>) -> Self {
        self.label = Some(text.into());
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

    pub fn show(self, ui: &mut Ui) -> Response {
        let (min_height, font_size, icon_size, _padding, default_width) = match self.size {
            ButtonSize::Sm => (26.0, 11.5, 13.0, egui::vec2(8.0, 3.0), 0.0),
            ButtonSize::Default => (32.0, 13.0, 14.5, egui::vec2(12.0, 5.0), 0.0),
            ButtonSize::Lg => (38.0, 14.5, 16.0, egui::vec2(16.0, 7.0), 0.0),
            ButtonSize::Icon => (32.0, 13.0, 16.0, egui::vec2(6.0, 6.0), 32.0),
            ButtonSize::IconSm => (24.0, 11.5, 13.0, egui::vec2(4.0, 4.0), 24.0),
        };

        let width = if self.full_width {
            ui.available_width()
        } else {
            default_width
        };

        let (text_color, fill_color, border_stroke) = match self.variant {
            ButtonVariant::Default => (self.theme.accent_foreground, self.theme.accent, Stroke::NONE),
            ButtonVariant::Secondary => (self.theme.text_primary, self.theme.surface_hover, Stroke::NONE),
            ButtonVariant::Outline => (
                self.theme.text_primary,
                Color32::TRANSPARENT,
                Stroke::new(1.0, self.theme.border_default),
            ),
            ButtonVariant::Ghost => (self.theme.text_secondary, Color32::TRANSPARENT, Stroke::NONE),
            ButtonVariant::Destructive => (self.theme.text_inverse, self.theme.danger, Stroke::NONE),
            ButtonVariant::Link => (self.theme.accent, Color32::TRANSPARENT, Stroke::NONE),
        };

        let mut job = LayoutJob::default();

        if self.loading {
            let time = ui.input(|i| i.time);
            ui.ctx().request_repaint();
            let spinner_chars = ['◐', '◓', '◑', '◒'];
            let idx = ((time * 6.0) as usize) % spinner_chars.len();
            job.append(
                &spinner_chars[idx].to_string(),
                0.0,
                TextFormat {
                    font_id: FontId::proportional(font_size),
                    color: text_color,
                    ..Default::default()
                },
            );
            if self.label.is_some() {
                job.append(" ", 0.0, TextFormat::default());
            }
        } else if let Some(icon) = self.icon {
            job.append(
                &char::from(icon).to_string(),
                0.0,
                TextFormat {
                    font_id: FontId::new(icon_size, FontFamily::Name("lucide".into())),
                    color: text_color,
                    ..Default::default()
                },
            );
            if self.label.is_some() {
                job.append("  ", 0.0, TextFormat::default());
            }
        }

        if let Some(ref text) = self.label {
            job.append(
                text,
                0.0,
                TextFormat {
                    font_id: FontId::proportional(font_size),
                    color: text_color,
                    ..Default::default()
                },
            );
        }

        let rounding = Rounding::same(6.0);
        let button = Button::new(job)
            .fill(fill_color)
            .stroke(border_stroke)
            .min_size(Vec2::new(width, min_height))
            .rounding(rounding);

        let enabled = self.enabled && !self.loading;
        let response = ui.add_enabled(enabled, button);

        // Hover feedback for Outline and Ghost variants
        if self.variant == ButtonVariant::Outline && response.hovered() && enabled {
            ui.painter()
                .rect_filled(response.rect, rounding, self.theme.surface_hover.linear_multiply(0.6));
        }

        response
    }
}
