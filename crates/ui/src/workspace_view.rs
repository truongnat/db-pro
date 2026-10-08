use super::*;

impl DbProApp {
    pub(super) fn draw_workspace_tabs(&mut self, ui: &mut egui::Ui) {
        let actions = {
            let mut context = workspace_tabs_surface_view::WorkspaceTabsViewContext::new(
                self.theme,
                &self.workspace,
                &self.query,
                &self.schema,
                &self.table,
            );
            context.draw_workspace_tabs(ui)
        };

        for action in actions {
            self.apply_workspace_tabs_action(action);
        }
    }

    fn apply_workspace_tabs_action(&mut self, action: workspace_tabs_surface_view::WorkspaceTabsAction) {
        use workspace_tabs_surface_view::WorkspaceTabsAction;

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

    fn draw_welcome(&mut self, ui: &mut egui::Ui) {
        welcome_surface_view::draw(self.theme, ui);
    }

    // cc-scan:allow LONG_FUNCTION — linear pipeline — one cohesive pass
    pub(super) fn draw_workspace(&mut self, ui: &mut egui::Ui) {
        if self.workspace.activity == Activity::Settings {
            self.draw_settings(ui);
            return;
        }

        self.draw_workspace_tabs(ui);
        match self.workspace.active_tab {
            WorkspaceTab::Welcome => self.draw_welcome(ui),
            WorkspaceTab::Query => self.draw_query(ui),
            WorkspaceTab::Table => self.draw_table_workspace(ui),
            WorkspaceTab::SchemaObject => self.draw_schema_object_workspace(ui),
            WorkspaceTab::Results => {
                let result = self.query.session.active_result().cloned();
                self.draw_results_pane(ui, result.as_ref());
            }
            WorkspaceTab::Diagram => {
                let active_driver = self.active_driver().to_owned();
                let connected = self.connection.lifecycle.is_connected();
                let connection_id = self.connection.lifecycle.active_connection_id().map(str::to_owned);
                let action = {
                    let mut context = diagram_view::DiagramViewContext {
                        theme: self.theme,
                        diagram: &mut self.schema.diagram,
                        explorer: &self.schema.explorer,
                        active_driver: &active_driver,
                        connected,
                        connection_id,
                    };
                    diagram_view::draw_diagram(&mut context, ui)
                };
                if let Some(action) = action {
                    self.apply_diagram_action(action);
                }
            }
            WorkspaceTab::SchemaWorkbench => self.draw_schema_workbench(ui),
            WorkspaceTab::SchemaCompare => {
                let action = {
                    let connection_name = self.active_connection_name().to_owned();
                    let active_schema = self.active_schema().to_owned();
                    let driver = self.active_driver().to_owned();
                    let mut context = schema_compare_view::SchemaCompareViewContext {
                        theme: self.theme,
                        compare: &mut self.schema.compare,
                        schema: &self.schema.explorer.schema,
                        connection_name: &connection_name,
                        active_schema: &active_schema,
                        driver: &driver,
                        feedback: &mut self.feedback,
                    };
                    schema_compare_view::draw_schema_compare(&mut context, ui)
                };
                if let Some(action) = action {
                    self.apply_schema_compare_action(action);
                }
            }
            WorkspaceTab::ComponentGallery => self.draw_component_gallery(ui),
        }
        self.draw_discard_changes_confirmation(ui);
    }
}
