use super::*;

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
        let request_id = self.task_bridge.next_request_id();
        let command = match self.table.state.execute_ddl_command(request_id, connection.id) {
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
        let request_id = self.task_bridge.next_request_id();
        let command =
            self.table
                .state
                .load_info_command(request_id, connection_id, self.active_schema().to_owned(), table);
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
        let request_id = self.task_bridge.next_request_id();
        let command =
            self.table
                .state
                .load_ddl_command(request_id, connection_id, self.active_schema().to_owned(), table);
        if self.dispatch_command(command) {
            self.table.state.table_ddl_request = Some(request_id);
            self.feedback.runtime_message = "Loading table DDL…".to_owned();
        }
    }

    pub(crate) fn request_table_data(&mut self) {
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
        let request_id = self.task_bridge.next_request_id();
        let command =
            self.table
                .data_query
                .load_data_command(request_id, connection_id, self.active_schema().to_owned(), table);
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
mod tests {
    use super::*;

    #[test]
    fn test_generate_sample_value_types() {
        let uuid_val = table_editor_values::generate_sample_value("id", "uuid");
        assert!(uuid::Uuid::parse_str(&uuid_val).is_ok());

        let time_val = table_editor_values::generate_sample_value("created_at", "timestamptz");
        assert!(chrono::DateTime::parse_from_rfc3339(&time_val).is_ok());

        let date_val = table_editor_values::generate_sample_value("birth_date", "date");
        assert_eq!(date_val.len(), 10);

        let email_val = table_editor_values::generate_sample_value("user_email", "varchar");
        assert!(email_val.contains('@'));

        let bool_val = table_editor_values::generate_sample_value("is_active", "boolean");
        assert!(bool_val == "true" || bool_val == "false");
    }

    #[test]
    fn test_parse_insert_value_uuid() {
        let valid_uuid = "a0eebc99-9c0b-4ef8-bb6d-6bb9bd380a11";
        let parsed = table_editor_values::parse_insert_value(valid_uuid, "uuid");
        assert_eq!(parsed, Ok(Some(UiCell::Text(valid_uuid.to_owned()))));

        let invalid_uuid = "not-a-uuid";
        assert!(table_editor_values::parse_insert_value(invalid_uuid, "uuid").is_err());
    }

    #[test]
    fn update_parser_preserves_null_empty_and_whitespace_text() {
        assert_eq!(
            table_editor_values::parse_update_value("NULL", "text"),
            Ok(UiCell::Null)
        );
        assert_eq!(
            table_editor_values::parse_update_value("", "text"),
            Ok(UiCell::Text(String::new()))
        );
        assert_eq!(
            table_editor_values::parse_update_value("   ", "text"),
            Ok(UiCell::Text("   ".to_owned()))
        );
        assert!(table_editor_values::parse_update_value("", "integer").is_err());
    }

    #[test]
    fn update_parser_validates_temporal_json_and_binary_values() {
        assert!(table_editor_values::parse_update_value("2026-09-12", "date").is_ok());
        assert!(table_editor_values::parse_update_value("12:30:45", "time").is_ok());
        assert!(table_editor_values::parse_update_value("2026-09-12T12:30:45Z", "timestamptz").is_ok());
        assert!(table_editor_values::parse_update_value("not-a-time", "time").is_err());
        assert!(table_editor_values::parse_update_value("{\"ok\":true}", "jsonb").is_ok());
        assert!(table_editor_values::parse_update_value("not-json", "jsonb").is_err());
        assert!(table_editor_values::parse_update_value("deadbeef", "bytea").is_err());
    }

    #[test]
    fn filter_operators_follow_column_type_and_keep_null_operators_universally() {
        assert!(TableDataQueryState::filter_operator_supported(
            "numeric(20,4)",
            &UiTableFilterOperator::GreaterThan
        ));
        assert!(!TableDataQueryState::filter_operator_supported(
            "numeric(20,4)",
            &UiTableFilterOperator::Contains
        ));
        assert!(TableDataQueryState::filter_operator_supported(
            "uuid",
            &UiTableFilterOperator::Equals
        ));
        assert!(!TableDataQueryState::filter_operator_supported(
            "uuid",
            &UiTableFilterOperator::StartsWith
        ));
        assert!(TableDataQueryState::filter_operator_supported(
            "boolean",
            &UiTableFilterOperator::IsNotNull
        ));
    }

    #[test]
    fn filter_draft_supports_multiple_and_editable_same_column_filters() {
        let mut app = DbProApp {
            table: TableEditorState {
                state: TableState {
                    table_info: Some(UiTableInfo {
                        schema: "public".to_owned(),
                        name: "orders".to_owned(),
                        row_count: None,
                        columns: vec![crate::UiTableColumn {
                            name: "amount".to_owned(),
                            data_type: "numeric(12,2)".to_owned(),
                            nullable: false,
                            default: None,
                            is_primary_key: false,
                            ..Default::default()
                        }],
                        primary_key: None,
                        indexes: Vec::new(),
                        foreign_keys: Vec::new(),
                        check_constraints: Vec::new(),
                        dependencies: Vec::new(),
                    }),
                    ..Default::default()
                },
                data_query: TableDataQueryState {
                    filter_column: "amount".to_owned(),
                    filter_operator: UiTableFilterOperator::GreaterThan,
                    filter_value: "10.00".to_owned(),
                    ..Default::default()
                },
                ..Default::default()
            },
            ..Default::default()
        };
        app.commit_table_filter_draft();
        app.table.data_query.filter_operator = UiTableFilterOperator::LessThan;
        app.table.data_query.filter_value = "20.00".to_owned();
        app.commit_table_filter_draft();

        assert_eq!(app.table.data_query.filters.len(), 2);
        app.table.data_query.filter_editing = Some(0);
        app.table.data_query.filter_value = "11.00".to_owned();
        app.commit_table_filter_draft();
        assert_eq!(app.table.data_query.filters[0].value, "11.00");
        assert_eq!(app.table.data_query.filters.len(), 2);
    }
}
