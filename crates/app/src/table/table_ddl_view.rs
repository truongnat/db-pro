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

        // "Apply DDL" never executes directly — it raises the confirmation
        // gate that `draw_ddl_confirmation_card` renders (#361).
        if self.draw_ddl_script_card(ui, can_mutate, &mut ddl) {
            self.table.state.table_ddl = Some(ddl.clone());
            self.table.state.ddl_execute_confirmation = true;
        }

        if self.table.state.ddl_execute_confirmation {
            let impact = ddl_impact_summary(self.table.state.table_ddl.as_deref().unwrap_or(""), table_name);
            ui.add_space(8.0);
            self.draw_ddl_confirmation_card(ui, &impact);
        }

        if self.table.state.table_ddl_discard_confirm {
            ui.add_space(8.0);
            self.draw_ddl_discard_card(ui);
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
                if self.table.state.table_ddl_dirty() {
                    // Never drop in-place edits silently (#357).
                    self.table.state.table_ddl_discard_confirm = true;
                } else {
                    self.table.state.table_ddl = None;
                    self.table.state.table_ddl_baseline = None;
                    self.request_table_ddl();
                }
            }
            Some(table_ddl_surface_view::DdlToolbarAction::OpenInQuery) => {
                // Open in a new document — the active buffer keeps its content (#420).
                self.open_query_document_with(ddl.to_owned(), None);
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
