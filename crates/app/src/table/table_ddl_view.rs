use super::*;
use crate::components::overlay::ToastPosition;

impl DbProApp {
    /// Draw the DDL / Schema tab.
    pub(super) fn draw_table_ddl_view(&mut self, ui: &mut egui::Ui, table_name: &str) {
        let Some(mut ddl) = self.table.state.table_ddl.clone() else {
            self.draw_table_ddl_placeholder(ui, table_name);
            return;
        };

        let schema = self.active_schema().to_owned();
        let can_mutate = self.can_mutate_active_connection();

        let action =
            table_ddl_surface_view::draw_toolbar(&table_ddl_surface_view::DdlToolbarContext { theme: self.theme }, ui);
        self.apply_ddl_toolbar_action(action, &ddl, &schema, table_name, ui);

        ui.add_space(8.0);

        let content_changed = self.draw_ddl_script_card(ui, can_mutate, &mut ddl);
        if content_changed {
            self.table.state.table_ddl = Some(ddl);
        }

        if self.table.state.ddl_execute_confirmation {
            let impact = ddl_impact_summary(self.table.state.table_ddl.as_deref().unwrap_or(""), table_name);
            ui.add_space(8.0);
            self.draw_ddl_confirmation_card(ui, &impact);
        }
    }

    fn apply_ddl_toolbar_action(
        &mut self,
        action: Option<table_ddl_surface_view::DdlToolbarAction>,
        ddl: &str,
        schema: &str,
        table_name: &str,
        ui: &mut egui::Ui,
    ) {
        match action {
            Some(table_ddl_surface_view::DdlToolbarAction::Refresh) => {
                self.table.state.table_ddl = None;
                self.request_table_ddl();
            }
            Some(table_ddl_surface_view::DdlToolbarAction::OpenInQuery) => {
                self.set_active_query_text(ddl.to_owned());
                self.workspace.active_tab = WorkspaceTab::Query;
                self.feedback.runtime_message = format!("Opened DDL for {schema}.{table_name} in Query editor");
            }
            Some(table_ddl_surface_view::DdlToolbarAction::Copy) => {
                ui.ctx().copy_text(ddl.to_owned());
                self.feedback
                    .toasts
                    .info("DDL copied to clipboard", ToastPosition::BottomRight);
                self.feedback.runtime_message = "DDL copied to clipboard".to_owned();
            }
            None => {}
        }
    }
}
