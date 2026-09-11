use crate::DbProTheme;
use egui::{Button, FontFamily, FontId, Frame, Margin, Response, RichText, Rounding, Stroke, Ui};
use lucide_icons::Icon;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AlertVariant {
    Default,
    Info,
    Success,
    Warning,
    Destructive,
}

pub struct ShadcnAlert<'a> {
    pub(crate) title: &'a str,
    pub(crate) description: &'a str,
    pub(crate) variant: AlertVariant,
    pub(crate) icon: Option<Icon>,
    pub(crate) dismissable: bool,
    pub(crate) theme: DbProTheme,
}

impl<'a> ShadcnAlert<'a> {
    pub fn new(title: &'a str, description: &'a str, theme: DbProTheme) -> Self {
        Self {
            title,
            description,
            variant: AlertVariant::Default,
            icon: None,
            dismissable: false,
            theme,
        }
    }

    pub fn variant(mut self, variant: AlertVariant) -> Self {
        self.variant = variant;
        self
    }

    pub fn icon(mut self, icon: Icon) -> Self {
        self.icon = Some(icon);
        self
    }

    pub fn dismissable(mut self, dismissable: bool) -> Self {
        self.dismissable = dismissable;
        self
    }

    pub fn show(self, ui: &mut Ui) -> Option<Response> {
        let (fill, border_color, icon_color, default_icon) = match self.variant {
            AlertVariant::Default => (
                self.theme.surface_panel,
                self.theme.border_default,
                self.theme.text_primary,
                Icon::Terminal,
            ),
            AlertVariant::Info => (
                self.theme.info.linear_multiply(0.08),
                self.theme.info.linear_multiply(0.3),
                self.theme.info,
                Icon::Info,
            ),
            AlertVariant::Success => (
                self.theme.success.linear_multiply(0.08),
                self.theme.success.linear_multiply(0.3),
                self.theme.success,
                Icon::CheckCircle2,
            ),
            AlertVariant::Warning => (
                self.theme.warning.linear_multiply(0.08),
                self.theme.warning.linear_multiply(0.3),
                self.theme.warning,
                Icon::AlertTriangle,
            ),
            AlertVariant::Destructive => (
                self.theme.danger.linear_multiply(0.08),
                self.theme.danger.linear_multiply(0.3),
                self.theme.danger,
                Icon::AlertCircle,
            ),
        };

        let icon = self.icon.unwrap_or(default_icon);
        let mut dismiss_response = None;

        Frame {
            fill,
            stroke: Stroke::new(1.0, border_color),
            inner_margin: Margin::same(12.0),
            rounding: Rounding::same(8.0),
            ..Default::default()
        }
        .show(ui, |ui| {
            ui.horizontal_top(|ui| {
                ui.label(
                    RichText::new(char::from(icon).to_string())
                        .font(FontId::new(16.0, FontFamily::Name("lucide".into())))
                        .color(icon_color),
                );
                ui.add_space(8.0);

                ui.vertical(|ui| {
                    ui.label(
                        RichText::new(self.title)
                            .size(13.5)
                            .strong()
                            .color(self.theme.text_primary),
                    );
                    ui.add_space(2.0);
                    ui.label(
                        RichText::new(self.description)
                            .size(12.0)
                            .color(self.theme.text_secondary),
                    );
                });

                if self.dismissable {
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::TOP), |ui| {
                        let resp = ui.add(
                            Button::new(
                                RichText::new(char::from(Icon::X).to_string())
                                    .font(FontId::new(13.0, FontFamily::Name("lucide".into())))
                                    .color(self.theme.text_muted),
                            )
                            .frame(false),
                        );
                        dismiss_response = Some(resp);
                    });
                }
            });
        });

        dismiss_response
    }
}
