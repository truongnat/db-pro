//! Schema introspection loading and error feedback rendering.
use super::*;
use egui::{Align, Layout, RichText};
use lucide_icons::Icon;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum ExplorerSchemaFeedbackAction {
    RefreshSchema,
}

pub(super) struct ExplorerSchemaFeedbackContext<'a> {
    pub(super) theme: DbProTheme,
    pub(super) error: Option<&'a str>,
    pub(super) loading: bool,
    pub(super) reduce_motion: bool,
    pub(super) has_active_connection: bool,
}

impl ExplorerSchemaFeedbackContext<'_> {
    pub(super) fn draw(&self, ui: &mut egui::Ui) -> Vec<ExplorerSchemaFeedbackAction> {
        if let Some(error) = self.error {
            return self.draw_error(ui, error);
        }
        if self.loading {
            self.draw_loading(ui);
        }
        Vec::new()
    }

    fn draw_error(&self, ui: &mut egui::Ui, error: &str) -> Vec<ExplorerSchemaFeedbackAction> {
        let mut actions = Vec::new();
        egui::Frame {
            fill: self.theme.surface_panel,
            inner_margin: egui::Margin::symmetric(SPACE_SM as i8, SPACE_XS as i8),
            stroke: egui::Stroke::new(STROKE_THIN, self.theme.border_subtle),
            corner_radius: egui::CornerRadius::same(RADIUS_SM as u8),
            ..Default::default()
        }
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(
                    RichText::new(char::from(Icon::TriangleAlert).to_string())
                        .font(font_icon(ICON_DEFAULT))
                        .color(self.theme.danger),
                );
                ui.label(RichText::new(t!("explorer.schema_load_failed")).strong().color(self.theme.danger));
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    if self.has_active_connection
                        && Button::new(self.theme)
                            .icon(Icon::RotateCcw)
                            .text(t!("explorer.refresh_schema"))
                            .variant(ButtonVariant::Secondary)
                            .size(ButtonSize::Sm)
                            .show(ui)
                            .clicked()
                    {
                        actions.push(ExplorerSchemaFeedbackAction::RefreshSchema);
                    }
                });
            });
            ui.add_space(6.0);
            egui::ScrollArea::vertical().max_height(120.0).show(ui, |ui| {
                ui.label(
                    RichText::new(error)
                        .small()
                        .monospace()
                        .color(self.theme.text_secondary),
                );
            });
        });
        ui.add_space(8.0);
        actions
    }

    fn draw_loading(&self, ui: &mut egui::Ui) {
        egui::Frame {
            fill: self.theme.surface_panel,
            inner_margin: egui::Margin::symmetric(SPACE_SM as i8, SPACE_XS as i8),
            stroke: egui::Stroke::new(STROKE_THIN, self.theme.border_subtle),
            corner_radius: egui::CornerRadius::same(RADIUS_SM as u8),
            ..Default::default()
        }
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                if self.reduce_motion {
                    ui.label(
                        RichText::new(char::from(Icon::LoaderCircle).to_string())
                            .font(font_icon(ICON_DEFAULT))
                            .color(self.theme.accent),
                    );
                } else {
                    Spinner::new(self.theme).show(ui);
                }
                ui.label(RichText::new(t!("explorer.loading_schema")).color(self.theme.accent));
                ui.label(
                    RichText::new("Large databases may take a moment.")
                        .small()
                        .color(self.theme.text_muted),
                );
            });
        });
        ui.add_space(8.0);
    }
}
