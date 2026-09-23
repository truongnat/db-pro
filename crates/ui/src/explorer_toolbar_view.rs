//! Schema explorer toolbar and empty state rendering.
use super::*;
use egui::{Align, FontFamily, Layout, RichText};
use lucide_icons::Icon;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum ExplorerToolbarAction {
    RefreshSchema,
    NewConnection,
}

pub(super) struct ExplorerToolbarContext<'a> {
    pub(super) theme: DbProTheme,
    pub(super) search: &'a mut String,
}

impl ExplorerToolbarContext<'_> {
    pub(super) fn draw_toolbar(&mut self, ui: &mut egui::Ui) -> Vec<ExplorerToolbarAction> {
        let mut actions = Vec::new();
        ui.horizontal(|ui| {
            ui.allocate_ui_with_layout(ui.available_size(), Layout::right_to_left(Align::Center), |ui| {
                let mut refresh_schema = false;
                let refresh_button =
                    compact_icon_button(ui, Icon::RotateCcw, self.theme).on_hover_text("Refresh active schema");
                refresh_button.context_menu(|ui| {
                    if ctx_menu_item(
                        ui,
                        Some(Icon::RotateCcw),
                        "Refresh Schema",
                        Some("F5"),
                        self.theme.text_primary,
                        self.theme,
                    )
                    .clicked()
                    {
                        refresh_schema = true;
                        ui.close_menu();
                    }
                });
                if refresh_button.clicked() || refresh_schema {
                    actions.push(ExplorerToolbarAction::RefreshSchema);
                }
                ui.with_layout(Layout::left_to_right(Align::Center), |ui| {
                    SearchInput::new(self.search, "Filter objects…", self.theme).show(ui);
                });
            });
        });
        actions
    }

    pub(super) fn draw_empty_state(&self, ui: &mut egui::Ui) -> Vec<ExplorerToolbarAction> {
        let mut actions = Vec::new();
        ui.add_space(36.0);
        ui.vertical_centered(|ui| {
            ui.label(
                RichText::new(char::from(Icon::Database).to_string())
                    .family(FontFamily::Name("lucide".into()))
                    .size(28.0)
                    .color(self.theme.text_muted),
            );
            ui.add_space(8.0);
            ui.label(RichText::new("No connections").strong().color(self.theme.text_primary));
            ui.add_space(3.0);
            ui.label(
                RichText::new("Create a database connection to begin.")
                    .small()
                    .color(self.theme.text_muted),
            );
            ui.add_space(12.0);
            if compact_button_with_icon(ui, Icon::Plus, "New connection", self.theme).clicked() {
                actions.push(ExplorerToolbarAction::NewConnection);
            }
        });
        actions
    }
}
