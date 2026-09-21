use super::*;

#[path = "table_profile_surface_view.rs"]
mod table_profile_surface_view;

impl DbProApp {
    pub(super) fn draw_table_workspace(&mut self, ui: &mut egui::Ui) {
        let Some(table_name) = self.schema.explorer.selected_table.clone() else {
            self.activate_welcome_tab();
            return;
        };
        self.ensure_table_workspace_requests();

        let schema = self.active_schema().to_owned();
        let connection_name = self
            .active_connection()
            .map(|c| c.name.clone())
            .unwrap_or_else(|| "Connected".to_owned());

        let surface_context = table_workspace_surface_view::TableWorkspaceSurfaceContext {
            theme: self.theme,
            connection_name: &connection_name,
            schema: &schema,
            table_name: &table_name,
            active_view: self.table.state.table_view,
            row_count: self.table.state.table_info.as_ref().and_then(|info| info.row_count),
        };
        let surface_actions = surface_context.draw(ui);
        for action in surface_actions {
            self.apply_table_workspace_surface_action(action, &schema, &table_name, ui.ctx());
        }
        ui.add_space(8.0);
        self.draw_table_workspace_content(ui, &table_name);
    }

    fn ensure_table_workspace_requests(&mut self) {
        if self.table.state.table_view == TableView::Ddl
            && self.table.state.table_ddl.is_none()
            && self.table.state.table_ddl_request.is_none()
        {
            self.request_table_ddl();
        }
        if (self.table.state.table_view == TableView::Data || self.table.state.table_view == TableView::Profile)
            && self.table.data_query.result.is_none()
            && self.table.data_query.request.is_none()
            && self.table.data_query.error.is_none()
        {
            self.request_table_data();
        }
    }

    fn draw_table_workspace_content(&mut self, ui: &mut egui::Ui, table_name: &str) {
        match self.table.state.table_view {
            TableView::Data => self.draw_table_data(ui, table_name),
            TableView::Profile => self.draw_column_profile_pane(ui, self.table.data_query.result.as_ref()),
            view => self.draw_table_metadata_view(ui, table_name, view),
        }
    }

    fn draw_table_metadata_view(&mut self, ui: &mut egui::Ui, table_name: &str, view: TableView) {
        match view {
            TableView::Structure => self.draw_scrollable_table_pane(ui, "table-structure-scroll", |app, ui| {
                app.draw_table_structure_view(ui);
            }),
            TableView::Indexes => self.draw_scrollable_table_pane(ui, "table-indexes-scroll", |app, ui| {
                app.draw_table_indexes_view(ui);
            }),
            TableView::Relations => self.draw_scrollable_table_pane(ui, "table-relations-scroll", |app, ui| {
                app.draw_table_relations_view(ui);
            }),
            TableView::Constraints => self.draw_scrollable_table_pane(ui, "table-constraints-scroll", |app, ui| {
                app.draw_table_constraints_view(ui);
            }),
            TableView::Dependencies => self.draw_scrollable_table_pane(ui, "table-dependencies-scroll", |app, ui| {
                app.draw_table_dependencies_view(ui);
            }),
            TableView::Ddl => self.draw_scrollable_table_pane(ui, "table-ddl-scroll", |app, ui| {
                app.draw_table_ddl_view(ui, table_name);
            }),
            TableView::Data | TableView::Profile => {
                unreachable!("data and profile are rendered by the workspace router")
            }
        }
    }

    fn draw_scrollable_table_pane(
        &mut self,
        ui: &mut egui::Ui,
        id: &'static str,
        draw: impl FnOnce(&mut Self, &mut egui::Ui),
    ) {
        egui::ScrollArea::vertical()
            .id_salt(id)
            .auto_shrink([false, false])
            .show(ui, |ui| draw(self, ui));
    }

    fn apply_table_workspace_surface_action(
        &mut self,
        action: table_workspace_surface_view::TableWorkspaceSurfaceAction,
        schema: &str,
        table_name: &str,
        context: &egui::Context,
    ) {
        use table_workspace_surface_view::TableWorkspaceSurfaceAction as Action;

        match action {
            Action::AskAgent => self.open_agent_prompt(
                format!("Explain the `{schema}.{table_name}` table and suggest queries"),
                context,
            ),
            Action::NewQuery => {
                self.set_active_query_text(format!("SELECT *\nFROM {schema}.{table_name}\nLIMIT 100;"));
                self.workspace.active_tab = WorkspaceTab::Query;
            }
            Action::Refresh => {
                self.request_table_info();
                match self.table.state.table_view {
                    TableView::Data => {
                        self.reset_table_data_page();
                        self.request_table_data();
                    }
                    TableView::Ddl => {
                        self.table.state.table_ddl = None;
                        self.request_table_ddl();
                    }
                    _ => {}
                }
            }
            Action::SelectView(view) => self.table.state.table_view = view,
        }
    }

    /// Adapts the table profile state into the presentation surface.
    pub(super) fn draw_column_profile_pane(&self, ui: &mut egui::Ui, result: Option<&UiQueryResult>) {
        table_profile_surface_view::draw_profile_pane(self.theme, result, ui);
    }
}
