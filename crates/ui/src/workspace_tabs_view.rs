use super::workspace_tabs_surface_view::{WorkspaceTabsAction, WorkspaceTabsViewContext};
use super::*;

impl DbProApp {
    pub(super) fn draw_workspace_tabs(&mut self, ui: &mut egui::Ui) {
        let actions = {
            let mut context =
                WorkspaceTabsViewContext::new(self.theme, &self.workspace, &self.query, &self.schema, &self.table);
            context.draw_workspace_tabs(ui)
        };

        for action in actions {
            self.apply_workspace_tabs_action(action);
        }
    }

    fn apply_workspace_tabs_action(&mut self, action: WorkspaceTabsAction) {
        match action {
            WorkspaceTabsAction::ActivateTab(tab) => self.workspace.active_tab = tab,
            WorkspaceTabsAction::ActivateWelcome => self.activate_welcome_tab(),
            WorkspaceTabsAction::CloseWelcome => self.close_welcome_tab(),
            WorkspaceTabsAction::CloseAllTabs => self.close_all_tabs(),
            WorkspaceTabsAction::SwitchQuery(index) => {
                self.switch_query_document(index);
                self.workspace.active_tab = WorkspaceTab::Query;
            }
            WorkspaceTabsAction::RequestCloseQuery(index) => self.request_close_query_document(index),
            WorkspaceTabsAction::DuplicateQuery(index) => self.duplicate_query_document(index),
            WorkspaceTabsAction::CloseOtherQueries(index) => self.close_other_query_documents(index),
            WorkspaceTabsAction::CloseQueriesToRight(index) => self.close_query_documents_to_right(index),
            WorkspaceTabsAction::RunQuery(index) => {
                self.switch_query_document(index);
                self.workspace.active_tab = WorkspaceTab::Query;
                self.dispatch_query();
            }
            WorkspaceTabsAction::CloseWorkspaceTab(tab) => self.request_close_workspace_tab(tab),
            WorkspaceTabsAction::RefreshTable => self.request_table_data(),
            WorkspaceTabsAction::NewQuery => self.new_query_document(),
        }
    }
}
