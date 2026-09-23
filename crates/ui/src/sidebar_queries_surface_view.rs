//! Queries-activity sidebar composition and typed intents.
use super::sidebar_queries_view::{SidebarQueriesAction, SidebarQueriesContext};
use super::sidebar_query_library_view::{SidebarQueryLibraryAction, SidebarQueryLibraryContext};
use super::sidebar_query_shortcuts_view::{SidebarQueryShortcutAction, SidebarQueryShortcutsContext};
use super::*;
use egui::RichText;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum SidebarQueriesSurfaceAction {
    OpenQuery(SidebarQueriesAction),
    Library(SidebarQueryLibraryAction),
    Shortcut(SidebarQueryShortcutAction),
}

pub(super) struct SidebarQueriesSurfaceContext<'a> {
    pub(super) theme: DbProTheme,
    pub(super) documents: &'a [QueryDocument],
    pub(super) active_tab: WorkspaceTab,
    pub(super) active_document_index: usize,
    pub(super) saved_queries: &'a [UiSavedQuerySummary],
    pub(super) query_folders: &'a [UiQueryFolderSummary],
    pub(super) history: &'a [String],
    pub(super) delete_confirmation_id: Option<&'a str>,
}

impl SidebarQueriesSurfaceContext<'_> {
    pub(super) fn draw(&self, ui: &mut egui::Ui) -> Vec<SidebarQueriesSurfaceAction> {
        let mut actions = self.draw_open_queries(ui);
        self.draw_library_sections(ui, &mut actions);
        self.draw_shortcuts(ui, &mut actions);
        actions
    }

    pub(super) fn draw_history(&self, ui: &mut egui::Ui) -> Vec<SidebarQueriesSurfaceAction> {
        let library = self.library_context();
        let mut actions = Vec::new();
        ui.label(
            RichText::new("Saved queries")
                .small()
                .strong()
                .color(self.theme.text_muted),
        );
        actions.extend(
            library
                .draw_saved_queries(ui)
                .into_iter()
                .map(SidebarQueriesSurfaceAction::Library),
        );
        ui.separator();
        ui.label(
            RichText::new("Local history")
                .small()
                .strong()
                .color(self.theme.text_muted),
        );
        actions.extend(
            library
                .draw_local_history(ui)
                .into_iter()
                .map(SidebarQueriesSurfaceAction::Library),
        );
        if let Some(id) = self.delete_confirmation_id {
            if let Some(action) = library.draw_delete_confirmation(ui, id) {
                actions.push(SidebarQueriesSurfaceAction::Library(action));
            }
        }
        actions
    }

    fn draw_open_queries(&self, ui: &mut egui::Ui) -> Vec<SidebarQueriesSurfaceAction> {
        SidebarQueriesContext {
            theme: self.theme,
            documents: self.documents,
            active_tab: self.active_tab,
            active_document_index: self.active_document_index,
        }
        .draw(ui)
        .into_iter()
        .map(SidebarQueriesSurfaceAction::OpenQuery)
        .collect()
    }

    fn draw_library_sections(&self, ui: &mut egui::Ui, actions: &mut Vec<SidebarQueriesSurfaceAction>) {
        let library = self.library_context();
        ui.add_space(SPACE_MD);
        section_label(ui, "SAVED QUERIES", self.theme);
        ui.add_space(SPACE_XS);
        actions.extend(
            library
                .draw_saved_queries(ui)
                .into_iter()
                .map(SidebarQueriesSurfaceAction::Library),
        );
        ui.add_space(SPACE_MD);
        section_label(ui, "HISTORY", self.theme);
        ui.add_space(SPACE_XS);
        actions.extend(
            library
                .draw_local_history(ui)
                .into_iter()
                .map(SidebarQueriesSurfaceAction::Library),
        );
        if let Some(id) = self.delete_confirmation_id {
            if let Some(action) = library.draw_delete_confirmation(ui, id) {
                actions.push(SidebarQueriesSurfaceAction::Library(action));
            }
        }
    }

    fn draw_shortcuts(&self, ui: &mut egui::Ui, actions: &mut Vec<SidebarQueriesSurfaceAction>) {
        ui.add_space(SPACE_MD);
        actions.extend(
            SidebarQueryShortcutsContext { theme: self.theme }
                .draw(ui)
                .into_iter()
                .map(SidebarQueriesSurfaceAction::Shortcut),
        );
    }

    fn library_context(&self) -> SidebarQueryLibraryContext<'_> {
        SidebarQueryLibraryContext {
            theme: self.theme,
            saved_queries: self.saved_queries,
            query_folders: self.query_folders,
            history: self.history,
        }
    }
}
