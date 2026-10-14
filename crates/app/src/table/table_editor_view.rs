// cc-scan:allow-file HUGE_FUNCTION,LONG_FUNCTION,HARD_COMPLEXITY,COMPLEXITY,DEEP_NESTING
// cc-scan:allow-file HARD_PARAMS,TOO_MANY_PARAMS,DUPLICATE_BLOCK
// egui painter/view file: fns are linear layout code; branches are per-state paint variants.
use super::*;

struct TableTarget {
    connection_id: String,
    schema: String,
    table: String,
}

fn build_table_info_command(request_id: RequestId, target: TableTarget) -> UiCommand {
    UiCommand::LoadTableInfo {
        request_id,
        connection_id: target.connection_id,
        schema: target.schema,
        table: target.table,
    }
}

fn build_table_ddl_command(request_id: RequestId, target: TableTarget) -> UiCommand {
    UiCommand::LoadTableDdl {
        request_id,
        connection_id: target.connection_id,
        schema: target.schema,
        table: target.table,
    }
}

fn build_execute_ddl_command(
    state: &TableState,
    request_id: RequestId,
    connection_id: String,
) -> Result<UiCommand, String> {
    Ok(UiCommand::ExecuteDdl {
        request_id,
        connection_id,
        sql: state.ddl_sql()?.to_owned(),
    })
}

impl DbProApp {
    pub(crate) fn submit_ddl(&mut self) {
        if !self.can_mutate_active_connection() {
            self.feedback.runtime_message = "Connect with write access to execute DDL".to_owned();
            return;
        }
        if self.table.state.ddl_execution_request.is_some() {
            return;
        }
        let Some(connection) = self.active_connection().cloned() else {
            self.feedback.runtime_message = "Connect to a database before executing DDL".to_owned();
            return;
        };
        let request_id = self.next_request_id();
        let command = match build_execute_ddl_command(&self.table.state, request_id, connection.id) {
            Ok(command) => command,
            Err(error) => {
                self.feedback.runtime_message = error;
                return;
            }
        };
        if self.dispatch_command(command) {
            self.table.state.ddl_execution_request = Some(request_id);
            self.table.state.ddl_execute_confirmation = false;
            self.feedback.runtime_message = "Executing DDL…".to_owned();
        }
    }

    /// Loading / failed placeholder shown while the DDL is not available.
    pub(super) fn draw_table_ddl_placeholder(&mut self, ui: &mut egui::Ui, table_name: &str) {
        table_ddl_surface_view::draw_placeholder(
            &table_ddl_surface_view::DdlPlaceholderContext {
                theme: self.theme,
                table_name,
                error: self.table.state.table_ddl_error.as_deref(),
            },
            ui,
        );
    }

    /// Editable CREATE SCRIPT card. Returns true when "Apply DDL" was pressed.
    pub(super) fn draw_ddl_script_card(&mut self, ui: &mut egui::Ui, writable: bool, ddl: &mut String) -> bool {
        let mut context = table_ddl_surface_view::DdlScriptContext {
            theme: self.theme,
            writable,
            executing: self.table.state.ddl_execution_request.is_some(),
            error: self.table.state.table_ddl_error.as_deref(),
            ddl,
        };
        match table_ddl_surface_view::draw_script_card(&mut context, ui) {
            Some(table_ddl_surface_view::DdlScriptAction::Apply) => true,
            Some(table_ddl_surface_view::DdlScriptAction::Changed) => {
                self.table.state.table_ddl_error = None;
                false
            }
            None => false,
        }
    }

    /// Confirmation gate shown before the DDL is executed against the database.
    pub(super) fn draw_ddl_confirmation_card(&mut self, ui: &mut egui::Ui, impact: &str) {
        let Some(ddl) = self.table.state.table_ddl.as_deref() else {
            return;
        };
        let risk = if impact.contains("destructive") || impact.contains("drop") {
            RiskLevel::Destructive
        } else {
            RiskLevel::Medium
        };
        let approval = ExecutionApproval::new("Review DDL Migration", impact, ddl, risk, self.theme);
        if let Some(action) = approval.show(ui) {
            match action {
                ExecutionApprovalAction::Run => {
                    self.submit_ddl();
                }
                ExecutionApprovalAction::Cancel => {
                    self.table.state.ddl_execute_confirmation = false;
                }
                _ => {}
            }
        }
    }
    pub(crate) fn request_table_info(&mut self) {
        let (Some(connection_id), Some(table)) = (
            self.connection.lifecycle.active_connection_id().map(str::to_owned),
            self.schema.explorer.selected_table.clone(),
        ) else {
            return;
        };
        let request_id = self.next_request_id();
        let command = build_table_info_command(
            request_id,
            TableTarget {
                connection_id,
                schema: self.active_schema().to_owned(),
                table,
            },
        );
        if self.dispatch_command(command) {
            self.table.state.table_info_request = Some(request_id);
            self.feedback.runtime_message = "Loading table structure…".to_owned();
        }
    }

    pub(crate) fn request_table_ddl(&mut self) {
        let (Some(connection_id), Some(table)) = (
            self.connection.lifecycle.active_connection_id().map(str::to_owned),
            self.schema.explorer.selected_table.clone(),
        ) else {
            return;
        };
        let request_id = self.next_request_id();
        let command = build_table_ddl_command(
            request_id,
            TableTarget {
                connection_id,
                schema: self.active_schema().to_owned(),
                table,
            },
        );
        if self.dispatch_command(command) {
            self.table.state.table_ddl_request = Some(request_id);
            self.feedback.runtime_message = "Loading table DDL…".to_owned();
        }
    }

    pub(crate) fn request_table_data(&mut self) {
        if self.table.data_query.inline_query_request.is_some() {
            // cc-scan:allow LINE_TOO_LONG — literal must not wrap
            self.feedback.runtime_message = "Wait for the current query to finish before refreshing table data".to_owned();
            return;
        }
        if let Some(sql) = self.table.data_query.active_inline_query_sql.clone() {
            let Some(connection_id) = self.connection.lifecycle.active_connection_id().map(str::to_owned) else {
                return;
            };
            self.send_table_data_query_run(connection_id, sql);
            return;
        }
        let Some(connection_id) = self.connection.lifecycle.active_connection_id().map(str::to_owned) else {
            return;
        };
        let table = self.schema.explorer.selected_table.clone().or_else(|| {
            match self.schema.explorer.selected_schema_object.as_ref() {
                Some(SchemaObjectSelection::View(name)) => Some(name.clone()),
                _ => None,
            }
        });
        let Some(table) = table else {
            return;
        };
        let request_id = self.next_request_id();
        let command = table_data_view::build_load_data_command(
            &self.table.data_query,
            request_id,
            table_data_view::TableDataTarget {
                connection_id,
                schema: self.active_schema().to_owned(),
                table,
            },
        );
        if self.dispatch_command(command) {
            self.table.data_query.request = Some(request_id);
            self.feedback.runtime_message = "Loading table data…".to_owned();
        }
    }

    pub(crate) fn commit_table_filter_draft(&mut self) {
        if !self.table.mutation.staged_changes.is_empty() {
            self.feedback.runtime_message = "Apply or discard staged changes before changing filters".to_owned();
            return;
        }
        match self
            .table
            .data_query
            .commit_filter_draft(self.table.state.table_info.as_ref())
        {
            Ok(true) => {
                self.table.data_query.offset = 0;
                self.request_table_data();
            }
            Ok(false) => {}
            Err(error) => self.feedback.runtime_message = error,
        }
    }

    pub(crate) fn remove_table_filter(&mut self, index: usize) {
        if !self.table.mutation.staged_changes.is_empty() {
            self.feedback.runtime_message = "Apply or discard staged changes before changing filters".to_owned();
            return;
        }
        if self.table.data_query.remove_filter(index) {
            self.table.data_query.offset = 0;
            self.request_table_data();
        }
    }

    pub(crate) fn clear_table_filters(&mut self) {
        if !self.table.mutation.staged_changes.is_empty() {
            self.feedback.runtime_message = "Apply or discard staged changes before changing filters".to_owned();
            return;
        }
        self.table.data_query.clear_filters();
        self.table.data_query.offset = 0;
        self.request_table_data();
    }

    pub(crate) fn reset_table_data_page(&mut self) {
        self.table.data_query.offset = 0;
        self.request_table_data();
    }

    pub(crate) fn reload_table_data_from_start(&mut self) {
        if !self.table.mutation.staged_changes.is_empty() {
            self.feedback.runtime_message = "Apply or discard staged changes before reloading".to_owned();
            return;
        }
        self.table.data_query.reset_page();
        self.table.data_query.invalidate_result();
        self.request_table_data();
    }
}

#[cfg(test)]
#[path = "tests/table_editor_view_tests.rs"]
mod tests;

