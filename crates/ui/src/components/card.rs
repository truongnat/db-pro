use crate::DbProTheme;
use egui::{FontFamily, FontId, Frame, Margin, RichText, Rounding, Stroke, Ui};
use lucide_icons::Icon;

pub struct ShadcnCard {
    theme: DbProTheme,
}

impl ShadcnCard {
    pub fn new(theme: DbProTheme) -> Self {
        Self { theme }
    }

    pub fn frame(&self) -> Frame {
        Frame {
            fill: self.theme.surface_elevated,
            stroke: Stroke::new(1.0, self.theme.border_subtle),
            inner_margin: Margin::same(16.0),
            rounding: Rounding::same(8.0),
            shadow: egui::epaint::Shadow {
                offset: egui::vec2(0.0, 2.0),
                blur: 8.0,
                spread: 0.0,
                color: egui::Color32::from_black_alpha(10),
            },
            ..Default::default()
        }
    }

    pub fn show<R>(&self, ui: &mut Ui, add_contents: impl FnOnce(&mut Ui) -> R) -> R {
        self.frame().show(ui, add_contents).inner
    }
}

pub fn card_header(ui: &mut Ui, title: &str, description: Option<&str>, theme: DbProTheme) {
    ui.vertical(|ui| {
        ui.label(RichText::new(title).size(15.0).strong().color(theme.text_primary));
        if let Some(desc) = description {
            ui.add_space(2.0);
            ui.label(RichText::new(desc).size(12.0).color(theme.text_secondary));
        }
    });
    ui.add_space(10.0);
}

pub struct MetricCard<'a> {
    title: &'a str,
    value: &'a str,
    change: Option<(&'a str, bool)>, // (e.g. "+12.5%", is_positive)
    icon: Option<Icon>,
    theme: DbProTheme,
}

impl<'a> MetricCard<'a> {
    pub fn new(title: &'a str, value: &'a str, theme: DbProTheme) -> Self {
        Self {
            title,
            value,
            change: None,
            icon: None,
            theme,
        }
    }

    pub fn change(mut self, text: &'a str, is_positive: bool) -> Self {
        self.change = Some((text, is_positive));
        self
    }

    pub fn icon(mut self, icon: Icon) -> Self {
        self.icon = Some(icon);
        self
    }

    pub fn show(self, ui: &mut Ui) {
        ShadcnCard::new(self.theme).show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(
                    RichText::new(self.title)
                        .size(12.5)
                        .strong()
                        .color(self.theme.text_muted),
                );
                if let Some(icon) = self.icon {
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.label(
                            RichText::new(char::from(icon).to_string())
                                .font(FontId::new(14.0, FontFamily::Name("lucide".into())))
                                .color(self.theme.text_muted),
                        );
                    });
                }
            });

            ui.add_space(8.0);
            ui.label(
                RichText::new(self.value)
                    .size(22.0)
                    .strong()
                    .color(self.theme.text_primary),
            );

            if let Some((change_text, is_positive)) = self.change {
                ui.add_space(4.0);
                let color = if is_positive {
                    self.theme.success
                } else {
                    self.theme.danger
                };
                let arrow = if is_positive { "↑" } else { "↓" };
                ui.label(
                    RichText::new(format!("{arrow} {change_text}"))
                        .size(11.5)
                        .strong()
                        .color(color),
                );
            }
        });
    }
}
