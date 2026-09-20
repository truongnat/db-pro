use super::*;
use crate::components::button::{Button, ButtonSize, ButtonVariant};
use egui::FontId;
use lucide_icons::Icon;

/// Whether the table editor executes DDL itself.
///
/// It does not: v0.1 ships schema/DDL *inspection* from the table editor and DDL
/// *execution* through the query editor (`docs/release/known-limitations.md`, "DDL via
/// query editor"). The confirmation card and `submit_ddl` stay compiled behind this
/// flag so the capability can be enabled by a deliberate change (with its own
/// qualification) instead of by a stray click.
const DDL_APPLY_ENABLED: bool = false;

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
        self.dispatch_command(command);
        self.table.state.ddl_execution_request = Some(request_id);
        self.table.state.ddl_execute_confirmation = false;
        self.feedback.runtime_message = "Executing DDL…".to_owned();
    }

    /// Loading / failed placeholder shown while the DDL is not available.
    pub(super) fn draw_table_ddl_placeholder(&mut self, ui: &mut egui::Ui, table_name: &str) {
        grid_frame(self.theme).show(ui, |ui| {
            ui.vertical_centered(|ui| {
                ui.add_space(28.0);
                let failed = self.table.state.table_ddl_error.as_deref();
                ui.label(icon_text(
                    if failed.is_some() {
                        Icon::TriangleAlert
                    } else {
                        Icon::Code2
                    },
                    "",
                    if failed.is_some() {
                        self.theme.warning
                    } else {
                        self.theme.accent
                    },
                ));
                ui.add_space(8.0);
                ui.label(
                    RichText::new(if failed.is_some() {
                        format!("DDL for {table_name} could not be loaded")
                    } else {
                        format!("Loading DDL for {table_name}…")
                    })
                    .strong()
                    .color(self.theme.text_primary),
                );
                if let Some(error) = failed {
                    ui.label(RichText::new(error).small().color(self.theme.text_secondary));
                }
                ui.add_space(28.0);
            });
        });
    }

    /// Editable CREATE SCRIPT card. Returns true when "Apply DDL" was pressed.
    pub(super) fn draw_ddl_script_card(&mut self, ui: &mut egui::Ui, writable: bool, ddl: &mut String) -> bool {
        let mut request_execution = false;
        card_frame(self.theme).show(ui, |ui| {
            ui.horizontal_wrapped(|ui| {
                section_label(ui, "CREATE SCRIPT", self.theme);
                ui.label(
                    RichText::new(if writable {
                        "Editable preview · execute DDL in the query editor (Open in Query)"
                    } else {
                        "Read-only preview"
                    })
                    .small()
                    .color(self.theme.text_muted),
                );
                // DDL execution from the table editor is not enabled in v0.1: the shipped
                // path is the query editor (`docs/release/known-limitations.md`). The
                // control stays visible and says why rather than looking live and doing
                // nothing — pressing it used to store the buffer back and stop there.
                if writable
                    && self.table.state.ddl_execution_request.is_none()
                    && Button::new(self.theme)
                        .icon(Icon::Play)
                        .text("Apply DDL")
                        .variant(ButtonVariant::Default)
                        .size(ButtonSize::Sm)
                        .enabled(DDL_APPLY_ENABLED)
                        .tooltip(if DDL_APPLY_ENABLED {
                            "Execute the DDL script"
                        } else {
                            "Not enabled in v0.1 — run DDL with Open in Query"
                        })
                        .show(ui)
                        .clicked()
                {
                    request_execution = true;
                }
            });
            ui.add_space(8.0);
            if let Some(error) = self.table.state.table_ddl_error.as_deref() {
                ui.label(
                    RichText::new(format!("DDL execution failed · {error}"))
                        .small()
                        .color(self.theme.danger),
                );
                ui.add_space(6.0);
            }
            editor_frame(self.theme).show(ui, |ui| {
                let response = ui.add(
                    TextEdit::multiline(&mut *ddl)
                        .font(FontId::monospace(13.0))
                        .desired_width(ui.available_width())
                        .desired_rows(18)
                        .interactive(writable),
                );
                if response.changed() {
                    self.table.state.table_ddl_error = None;
                }
            });
        });
        request_execution
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
        self.table.state.table_info_request = Some(request_id);
        self.dispatch_command(self.table.state.load_info_command(
            request_id,
            connection_id,
            self.active_schema().to_owned(),
            table,
        ));
        self.feedback.runtime_message = "Loading table structure…".to_owned();
    }

    pub(crate) fn request_table_ddl(&mut self) {
        let (Some(connection_id), Some(table)) = (
            self.connection.lifecycle.active_connection_id().map(str::to_owned),
            self.schema.explorer.selected_table.clone(),
        ) else {
            return;
        };
        let request_id = self.task_bridge.next_request_id();
        self.table.state.table_ddl_request = Some(request_id);
        self.dispatch_command(self.table.state.load_ddl_command(
            request_id,
            connection_id,
            self.active_schema().to_owned(),
            table,
        ));
        self.feedback.runtime_message = "Loading table DDL…".to_owned();
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
        self.table.data_query.request = Some(request_id);
        self.dispatch_command(self.table.data_query.load_data_command(
            request_id,
            connection_id,
            self.active_schema().to_owned(),
            table,
        ));
        self.feedback.runtime_message = "Loading table data…".to_owned();
    }

    pub(crate) fn commit_table_filter_draft(&mut self) {
        if !self.table.mutation.staged_changes.is_empty() {
            self.feedback.runtime_message = "Apply or discard staged changes before changing filters".to_owned();
            return;
        }
        let column = self.table.data_query.filter_column.trim();
        if column.is_empty() {
            return;
        }
        let is_null_operator = matches!(
            self.table.data_query.filter_operator,
            UiTableFilterOperator::IsNull | UiTableFilterOperator::IsNotNull
        );
        let data_type = self
            .table
            .state
            .table_info
            .as_ref()
            .and_then(|info| info.columns.iter().find(|item| item.name == column))
            .map(|item| item.data_type.clone())
            .unwrap_or_else(|| "text".to_owned());
        if !TableDataQueryState::filter_operator_supported(&data_type, &self.table.data_query.filter_operator) {
            self.feedback.runtime_message = format!("That filter operator is not supported for {data_type}");
            return;
        }
        if !is_null_operator
            && self.table.data_query.filter_value.is_empty()
            && !table_editor_values::is_text_type(&data_type.to_ascii_lowercase())
        {
            self.feedback.runtime_message = "Enter a filter value first".to_owned();
            return;
        }
        if !is_null_operator {
            if let Err(error) = table_editor_values::parse_update_value(&self.table.data_query.filter_value, &data_type)
            {
                self.feedback.runtime_message = format!("Invalid filter for {column}: {error}");
                return;
            }
        }
        let filter = UiTableDataFilter {
            column: column.to_owned(),
            data_type,
            operator: self.table.data_query.filter_operator.clone(),
            value: if is_null_operator {
                String::new()
            } else {
                self.table.data_query.filter_value.clone()
            },
        };
        if let Some(index) = self.table.data_query.filter_editing.take() {
            if let Some(existing) = self.table.data_query.filters.get_mut(index) {
                *existing = filter;
            } else {
                self.table.data_query.filters.push(filter);
            }
        } else {
            self.table.data_query.filters.push(filter);
        }
        self.table.data_query.offset = 0;
        self.request_table_data();
    }

    pub(crate) fn remove_table_filter(&mut self, index: usize) {
        if !self.table.mutation.staged_changes.is_empty() {
            self.feedback.runtime_message = "Apply or discard staged changes before changing filters".to_owned();
            return;
        }
        if index < self.table.data_query.filters.len() {
            self.table.data_query.filters.remove(index);
            self.table.data_query.filter_editing = match self.table.data_query.filter_editing {
                Some(editing) if editing == index => None,
                Some(editing) if editing > index => Some(editing - 1),
                other => other,
            };
            self.table.data_query.offset = 0;
            self.request_table_data();
        }
    }

    pub(crate) fn clear_table_filters(&mut self) {
        if !self.table.mutation.staged_changes.is_empty() {
            self.feedback.runtime_message = "Apply or discard staged changes before changing filters".to_owned();
            return;
        }
        self.table.data_query.filters.clear();
        self.table.data_query.filter_editing = None;
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
