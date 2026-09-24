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
        let active_connection_id = self.connection.lifecycle.active_connection_id().map(str::to_owned);
        let actions = welcome_surface_view::WelcomeSurfaceContext {
            theme: self.theme,
            welcome: &mut self.welcome,
            catalog: &self.connection.catalog,
            active_connection_id: active_connection_id.as_deref(),
        }
        .draw(ui);
        self.apply_welcome_actions(actions);
    }

    fn apply_welcome_actions(&mut self, actions: Vec<welcome_surface_view::WelcomeAction>) {
        use welcome_surface_view::WelcomeAction;

        for action in actions {
            match action {
                WelcomeAction::NewConnection => self.connection.open_new(),
                WelcomeAction::NewQuery => {
                    self.new_query_document();
                    self.workspace.active_tab = WorkspaceTab::Query;
                }
                WelcomeAction::OpenPalette => self.palette.open(PaletteMode::Commands),
                WelcomeAction::OpenDraftQuery(draft) => {
                    self.set_active_query_text(draft);
                    self.workspace.active_tab = WorkspaceTab::Query;
                    self.feedback.runtime_message = "Opened draft in Query".to_owned();
                }
                WelcomeAction::Connect(connection_id) => {
                    if let Some(connection) = self.connection.catalog.find(&connection_id).cloned() {
                        self.connect_to_connection(&connection);
                    }
                }
            }
        }
    }

    pub(super) fn draw_workspace(&mut self, ui: &mut egui::Ui) {
        self.draw_workspace_tabs(ui);
        match self.workspace.active_tab {
            WorkspaceTab::Welcome => self.draw_welcome(ui),
            WorkspaceTab::Query => self.draw_query(ui),
            WorkspaceTab::Table => self.draw_table_workspace(ui),
            WorkspaceTab::SchemaObject => self.draw_schema_object_workspace(ui),
            WorkspaceTab::Diagram => {
                let active_driver = self.active_driver().to_owned();
                let connected = self.connection.lifecycle.is_connected();
                let action = {
                    let mut context = diagram_view::DiagramViewContext {
                        theme: self.theme,
                        diagram: &mut self.schema.diagram,
                        explorer: &self.schema.explorer,
                        active_driver: &active_driver,
                        connected,
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
                    let driver = self.active_driver().to_owned();
                    let mut context = schema_compare_view::SchemaCompareViewContext {
                        theme: self.theme,
                        compare: &mut self.schema.compare,
                        schema: &self.schema.explorer.schema,
                        connection_name: &connection_name,
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
