//! Saved-query library and local history sidebar rendering.
use super::*;
use lucide_icons::Icon;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum SidebarQueryLibraryAction {
    NewQuery,
    OpenQuery(String),
    CopySql { name: String, sql: String },
    Rename(UiSavedQuerySummary),
    RequestDelete(String),
    ConfirmDelete(String),
    CancelDelete,
    RequestDeleteFolder(Option<String>),
    OpenHistory(String),
}

pub(super) struct SidebarQueryLibraryContext<'a> {
    pub(super) theme: DbProTheme,
    pub(super) saved_queries: &'a [UiSavedQuerySummary],
    pub(super) query_folders: &'a [UiQueryFolderSummary],
    pub(super) history: &'a [String],
}

impl SidebarQueryLibraryContext<'_> {
    pub(super) fn draw_saved_queries(&self, ui: &mut egui::Ui) -> Vec<SidebarQueryLibraryAction> {
        if self.saved_queries.is_empty() {
            return self.draw_empty_saved_queries(ui);
        }

        let mut groups: Vec<(String, Vec<UiSavedQuerySummary>)> = Vec::new();
        for query in self.saved_queries {
            let folder = query.folder.clone().unwrap_or_else(|| "Unfiled".to_owned());
            if let Some((_, queries)) = groups.iter_mut().find(|(name, _)| name == &folder) {
                queries.push(query.clone());
            } else {
                groups.push((folder, vec![query.clone()]));
            }
        }

        let mut actions = Vec::new();
        for (folder, queries) in groups {
            actions.extend(self.draw_saved_query_folder(ui, folder, queries));
        }
        actions
    }

    pub(super) fn draw_local_history(&self, ui: &mut egui::Ui) -> Vec<SidebarQueryLibraryAction> {
        if self.history.is_empty() {
            ui.label(RichText::new("No queries run yet").small().color(self.theme.text_muted));
            return Vec::new();
        }

        let mut actions = Vec::new();
        for query in self.history.iter().rev().take(15) {
            let first_line = query.lines().next().unwrap_or("query").trim();
            let display = crate::components::truncate_ellipsis(first_line, 26);
            if sidebar_item(ui, Icon::History, &display, false, self.theme)
                .on_hover_text(query)
                .clicked()
            {
                actions.push(SidebarQueryLibraryAction::OpenHistory(query.clone()));
            }
            ui.add_space(2.0);
        }
        actions
    }

    pub(super) fn draw_delete_confirmation(
        &self,
        ui: &mut egui::Ui,
        query_id: &str,
    ) -> Option<SidebarQueryLibraryAction> {
        let mut action = None;
        ui.colored_label(self.theme.warning, "Delete this saved query?");
        ui.horizontal(|ui| {
            if compact_button(ui, "Confirm delete", self.theme).clicked() {
                action = Some(SidebarQueryLibraryAction::ConfirmDelete(query_id.to_owned()));
            }
            if compact_button(ui, "Cancel", self.theme).clicked() {
                action = Some(SidebarQueryLibraryAction::CancelDelete);
            }
        });
        action
    }

    fn draw_empty_saved_queries(&self, ui: &mut egui::Ui) -> Vec<SidebarQueryLibraryAction> {
        let mut actions = Vec::new();
        card_frame(self.theme).show(ui, |ui| {
            ui.vertical_centered(|ui| {
                ui.label(icon_text(Icon::Bookmark, "", self.theme.accent));
                ui.add_space(6.0);
                ui.label(RichText::new("No saved queries yet").strong());
                ui.label(
                    RichText::new("Save a query to keep it close at hand.")
                        .small()
                        .color(self.theme.text_muted),
                );
                ui.add_space(8.0);
                if compact_button_with_icon(ui, Icon::Plus, "New query", self.theme).clicked() {
                    actions.push(SidebarQueryLibraryAction::NewQuery);
                }
            });
        });
        actions
    }

    fn draw_saved_query_folder(
        &self,
        ui: &mut egui::Ui,
        folder: String,
        queries: Vec<UiSavedQuerySummary>,
    ) -> Vec<SidebarQueryLibraryAction> {
        let folder_id = self
            .query_folders
            .iter()
            .find(|item| item.name == folder)
            .map(|item| item.id.clone());
        let mut actions = Vec::new();
        let header = egui::collapsing_header::CollapsingState::load_with_default_open(
            ui.ctx(),
            ui.make_persistent_id(("saved-query-folder", folder.as_str())),
            true,
        )
        .show_header(ui, |ui| {
            ui.label(icon_text(Icon::FolderOpen, &folder, self.theme.text_primary));
            ui.label(
                RichText::new(format!("{} queries", queries.len()))
                    .small()
                    .color(self.theme.text_muted),
            );
        });
        let (_, header_response, _) = header.body(|ui| {
            for query in &queries {
                actions.extend(self.draw_saved_query_entry(ui, query));
            }
        });
        let theme = self.theme;
        context_action_menu(ui, &header_response.response, theme, |ui, close_menu| {
            if folder_id.is_some()
                && ctx_menu_item(ui, Some(Icon::Trash2), "Delete folder", None, theme.danger, theme).clicked()
            {
                actions.push(SidebarQueryLibraryAction::RequestDeleteFolder(folder_id.clone()));
                *close_menu = true;
            }
        });
        actions
    }

    fn draw_saved_query_entry(&self, ui: &mut egui::Ui, query: &UiSavedQuerySummary) -> Vec<SidebarQueryLibraryAction> {
        let response = sidebar_item(ui, Icon::FileCode2, &query.name, false, self.theme);
        let is_context_menu = is_context_menu_triggered(&response, ui);
        let mut actions = self.draw_saved_query_menu(ui, &response, query);
        if response.clicked() && !is_context_menu {
            actions.push(SidebarQueryLibraryAction::OpenQuery(query.sql.clone()));
        }
        actions
    }

    fn draw_saved_query_menu(
        &self,
        ui: &mut egui::Ui,
        response: &egui::Response,
        query: &UiSavedQuerySummary,
    ) -> Vec<SidebarQueryLibraryAction> {
        let mut actions = Vec::new();
        context_action_menu(ui, response, self.theme, |ui, close_menu| {
            if ctx_menu_item(
                ui,
                Some(Icon::Play),
                "Open in Editor",
                None,
                self.theme.text_primary,
                self.theme,
            )
            .clicked()
            {
                *close_menu = true;
            }
            if ctx_menu_item(
                ui,
                Some(Icon::Copy),
                "Copy SQL",
                None,
                self.theme.text_primary,
                self.theme,
            )
            .clicked()
            {
                actions.push(SidebarQueryLibraryAction::CopySql {
                    name: query.name.clone(),
                    sql: query.sql.clone(),
                });
                *close_menu = true;
            }
            ui.separator();
            if ctx_menu_item(
                ui,
                Some(Icon::Pencil),
                "Rename query",
                None,
                self.theme.text_primary,
                self.theme,
            )
            .clicked()
            {
                actions.push(SidebarQueryLibraryAction::Rename(query.clone()));
                *close_menu = true;
            }
            if ctx_menu_item(
                ui,
                Some(Icon::Trash2),
                "Delete query",
                None,
                self.theme.danger,
                self.theme,
            )
            .clicked()
            {
                actions.push(SidebarQueryLibraryAction::RequestDelete(query.id.clone()));
                *close_menu = true;
            }
        });
        actions
    }
}
