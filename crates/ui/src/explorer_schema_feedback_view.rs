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
        grid_frame(self.theme).show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(icon_text(Icon::TriangleAlert, "Schema load failed", self.theme.danger));
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    if self.has_active_connection
                        && secondary_button_with_icon(ui, Icon::RotateCcw, "Refresh schema", self.theme).clicked()
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
        grid_frame(self.theme).show(ui, |ui| {
            ui.horizontal(|ui| {
                if self.reduce_motion {
                    ui.label(icon_text(Icon::LoaderCircle, "Loading schema…", self.theme.accent));
                } else {
                    ui.spinner();
                    ui.label(RichText::new("Loading schema…").color(self.theme.accent));
                }
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
