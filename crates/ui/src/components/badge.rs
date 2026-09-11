use crate::DbProTheme;
use egui::{Color32, FontFamily, FontId, Frame, Margin, RichText, Rounding, Stroke, Ui};
use lucide_icons::Icon;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BadgeVariant {
    Default,
    Secondary,
    Outline,
    Destructive,
    Success,
    Warning,
    Info,
}

pub struct ShadcnBadge<'a> {
    pub(crate) text: &'a str,
    pub(crate) variant: BadgeVariant,
    pub(crate) dot: bool,
    pub(crate) icon: Option<Icon>,
    pub(crate) theme: DbProTheme,
}

impl<'a> ShadcnBadge<'a> {
    pub fn new(text: &'a str, theme: DbProTheme) -> Self {
        Self {
            text,
            variant: BadgeVariant::Default,
            dot: false,
            icon: None,
            theme,
        }
    }

    pub fn variant(mut self, variant: BadgeVariant) -> Self {
        self.variant = variant;
        self
    }

    pub fn dot(mut self, dot: bool) -> Self {
        self.dot = dot;
        self
    }

    pub fn icon(mut self, icon: Icon) -> Self {
        self.icon = Some(icon);
        self
    }

    pub fn show(self, ui: &mut Ui) {
        let (fill, text_color, border_stroke, dot_color) = match self.variant {
            BadgeVariant::Default => (
                self.theme.accent_soft,
                self.theme.accent,
                Stroke::NONE,
                self.theme.accent,
            ),
            BadgeVariant::Secondary => (
                self.theme.surface_hover,
                self.theme.text_secondary,
                Stroke::NONE,
                self.theme.text_secondary,
            ),
            BadgeVariant::Outline => (
                Color32::TRANSPARENT,
                self.theme.text_primary,
                Stroke::new(1.0, self.theme.border_default),
                self.theme.text_muted,
            ),
            BadgeVariant::Destructive => (
                self.theme.danger.linear_multiply(0.15),
                self.theme.danger,
                Stroke::NONE,
                self.theme.danger,
            ),
            BadgeVariant::Success => (
                self.theme.success.linear_multiply(0.15),
                self.theme.success,
                Stroke::NONE,
                self.theme.success,
            ),
            BadgeVariant::Warning => (
                self.theme.warning.linear_multiply(0.15),
                self.theme.warning,
                Stroke::NONE,
                self.theme.warning,
            ),
            BadgeVariant::Info => (
                self.theme.info.linear_multiply(0.15),
                self.theme.info,
                Stroke::NONE,
                self.theme.info,
            ),
        };

        Frame {
            fill,
            stroke: border_stroke,
            inner_margin: Margin::symmetric(7.0, 3.0),
            rounding: Rounding::same(12.0),
            ..Default::default()
        }
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                if self.dot {
                    let (rect, _) = ui.allocate_exact_size(egui::vec2(6.0, 6.0), egui::Sense::hover());
                    ui.painter().circle_filled(rect.center(), 2.5, dot_color);
                    ui.add_space(3.0);
                } else if let Some(icon) = self.icon {
                    ui.label(
                        RichText::new(char::from(icon).to_string())
                            .font(FontId::new(11.0, FontFamily::Name("lucide".into())))
                            .color(text_color),
                    );
                    ui.add_space(2.0);
                }
                ui.label(RichText::new(self.text).size(11.0).strong().color(text_color));
            });
        });
    }
}
