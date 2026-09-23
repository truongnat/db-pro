//! Open-query list rendering for the Queries activity.
use super::*;
use egui::{Align, Layout};
use lucide_icons::Icon;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum SidebarQueriesAction {
    NewQuery,
    NewScratch,
    Select(usize),
    Duplicate(usize),
    Rename(usize),
    Close(usize),
}

pub(super) struct SidebarQueriesContext<'a> {
    pub(super) theme: DbProTheme,
    pub(super) documents: &'a [QueryDocument],
    pub(super) active_tab: WorkspaceTab,
    pub(super) active_document_index: usize,
}

impl SidebarQueriesContext<'_> {
    pub(super) fn draw(&self, ui: &mut egui::Ui) -> Vec<SidebarQueriesAction> {
        let mut actions = self.draw_header(ui);
        ui.add_space(8.0);
        for (index, document) in self.documents.iter().enumerate() {
            actions.extend(self.draw_document(ui, index, document));
        }
        actions
    }

    fn draw_header(&self, ui: &mut egui::Ui) -> Vec<SidebarQueriesAction> {
        let mut actions = Vec::new();
        ui.horizontal(|ui| {
            section_label(ui, "OPEN QUERIES", self.theme);
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                if compact_icon_button(ui, Icon::FilePlus2, self.theme)
                    .on_hover_text("New scratch query")
                    .clicked()
                {
                    actions.push(SidebarQueriesAction::NewScratch);
                }
                if compact_icon_button(ui, Icon::Plus, self.theme)
                    .on_hover_text("New query")
                    .clicked()
                {
                    actions.push(SidebarQueriesAction::NewQuery);
                }
            });
        });
        actions
    }

    fn draw_document(&self, ui: &mut egui::Ui, index: usize, document: &QueryDocument) -> Vec<SidebarQueriesAction> {
        let selected = self.active_tab == WorkspaceTab::Query && self.active_document_index == index;
        let title = if document.is_dirty() {
            format!("{}  •", document.title)
        } else {
            document.title.clone()
        };
        let response = sidebar_item(ui, Icon::FileCode2, &title, selected, self.theme);
        let is_context_menu = is_context_menu_triggered(&response, ui);
        let mut actions = self.draw_document_menu(ui, &response, index);
        if response.clicked() && !is_context_menu {
            actions.push(SidebarQueriesAction::Select(index));
        }
        actions
    }

    fn draw_document_menu(
        &self,
        ui: &mut egui::Ui,
        response: &egui::Response,
        index: usize,
    ) -> Vec<SidebarQueriesAction> {
        let mut actions = Vec::new();
        context_action_menu(ui, response, self.theme, |ui, close_menu| {
            if ctx_menu_item(
                ui,
                Some(Icon::Copy),
                "Duplicate query",
                None,
                self.theme.text_primary,
                self.theme,
            )
            .clicked()
            {
                actions.push(SidebarQueriesAction::Duplicate(index));
                *close_menu = true;
            }
            if ctx_menu_item(
                ui,
                Some(Icon::Pencil),
                "Rename tab",
                None,
                self.theme.text_primary,
                self.theme,
            )
            .clicked()
            {
                actions.push(SidebarQueriesAction::Rename(index));
                *close_menu = true;
            }
            if self.documents.len() > 1
                && ctx_menu_item(
                    ui,
                    Some(Icon::Trash2),
                    "Close query",
                    None,
                    self.theme.danger,
                    self.theme,
                )
                .clicked()
            {
                actions.push(SidebarQueriesAction::Close(index));
                *close_menu = true;
            }
        });
        actions
    }
}
