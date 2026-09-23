//! Pinned and recent table activity rendering.
use super::*;
use egui::{Align, Layout, RichText};
use lucide_icons::Icon;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum SidebarDataAction {
    OpenData(String),
    OpenStructure(String),
    OpenQuery(String),
    TogglePinned(String),
    RemoveRecent(String),
}

pub(super) struct SidebarDataContext<'a> {
    pub(super) theme: DbProTheme,
    pub(super) pinned_tables: &'a [String],
    pub(super) recent_tables: &'a [String],
    pub(super) selected_table: Option<&'a str>,
}

impl SidebarDataContext<'_> {
    pub(super) fn draw(&self, ui: &mut egui::Ui) -> Vec<SidebarDataAction> {
        let mut actions = Vec::new();
        self.draw_section_header(ui, "PINNED TABLES", self.pinned_tables.len());
        ui.add_space(8.0);
        if self.pinned_tables.is_empty() {
            ui.label(
                RichText::new("Pin tables from Explorer or Quick Open for fast reopen.")
                    .small()
                    .color(self.theme.text_muted),
            );
        } else {
            for table in self.pinned_tables {
                actions.extend(self.draw_table_row(ui, table, true));
            }
        }

        ui.add_space(16.0);
        self.draw_section_header(ui, "RECENT TABLES", self.recent_tables.len());
        ui.add_space(8.0);
        if self.recent_tables.is_empty() {
            ui.label(
                RichText::new("Tables you open appear here in most-recent order.")
                    .small()
                    .color(self.theme.text_muted),
            );
        } else {
            for table in self.recent_tables {
                actions.extend(self.draw_table_row(ui, table, false));
            }
        }

        ui.add_space(12.0);
        ui.label(
            RichText::new("Right-click for open, pin, or remove")
                .small()
                .color(self.theme.text_muted),
        );
        actions
    }

    fn draw_section_header(&self, ui: &mut egui::Ui, title: &str, count: usize) {
        ui.horizontal(|ui| {
            section_label(ui, title, self.theme);
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                badge(ui, &count.to_string(), self.theme.surface_hover, self.theme.text_muted);
            });
        });
    }

    fn draw_table_row(&self, ui: &mut egui::Ui, table: &str, from_pinned: bool) -> Vec<SidebarDataAction> {
        let selected = self.selected_table == Some(table);
        let icon = if from_pinned { Icon::Pin } else { Icon::Table2 };
        let response = sidebar_item(ui, icon, table, selected, self.theme);
        let is_context_menu = is_context_menu_triggered(&response, ui);
        let mut actions = self.draw_table_menu(ui, &response, table, from_pinned);

        if response.clicked() && !is_context_menu {
            actions.push(SidebarDataAction::OpenData(table.to_owned()));
        }
        actions
    }

    fn draw_table_menu(
        &self,
        ui: &mut egui::Ui,
        response: &egui::Response,
        table: &str,
        from_pinned: bool,
    ) -> Vec<SidebarDataAction> {
        let is_pinned = self.pinned_tables.iter().any(|item| item == table);
        let pin_label = if is_pinned { "Unpin table" } else { "Pin table" };
        let mut actions = Vec::new();

        context_action_menu(ui, response, self.theme, |ui, close_menu| {
            if ctx_menu_item(
                ui,
                Some(Icon::Table2),
                "Open data",
                None,
                self.theme.text_primary,
                self.theme,
            )
            .clicked()
            {
                actions.push(SidebarDataAction::OpenData(table.to_owned()));
                *close_menu = true;
            }
            if ctx_menu_item(
                ui,
                Some(Icon::Columns3),
                "Open structure",
                None,
                self.theme.text_primary,
                self.theme,
            )
            .clicked()
            {
                actions.push(SidebarDataAction::OpenStructure(table.to_owned()));
                *close_menu = true;
            }
            if ctx_menu_item(
                ui,
                Some(Icon::FileCode2),
                "New query for table",
                None,
                self.theme.text_primary,
                self.theme,
            )
            .clicked()
            {
                actions.push(SidebarDataAction::OpenQuery(table.to_owned()));
                *close_menu = true;
            }
            ui.separator();
            if ctx_menu_item(
                ui,
                Some(Icon::Pin),
                pin_label,
                None,
                self.theme.text_primary,
                self.theme,
            )
            .clicked()
            {
                actions.push(SidebarDataAction::TogglePinned(table.to_owned()));
                *close_menu = true;
            }
            if !from_pinned
                && ctx_menu_item(
                    ui,
                    Some(Icon::Trash2),
                    "Remove from recent",
                    None,
                    self.theme.text_primary,
                    self.theme,
                )
                .clicked()
            {
                actions.push(SidebarDataAction::RemoveRecent(table.to_owned()));
                *close_menu = true;
            }
        });
        actions
    }
}
