use super::*;
use crate::RequestId;

/// A statement (or script) the classifier rates `Destructive`, held until the user
/// confirms the exact text the prompt displayed.
#[derive(Debug, Clone)]
pub(crate) struct PendingDestructiveRun {
    pub(crate) sql: String,
    pub(crate) execution_range: (usize, usize),
    pub(crate) version: u64,
    pub(crate) all_statements: bool,
}

pub(super) struct QueryHistoryRecord {
    pub(super) sql: String,
    pub(super) connection_id: Option<String>,
    pub(super) schema: Option<String>,
    pub(super) started_at: String,
    pub(super) status: UiQueryHistoryStatus,
    pub(super) duration_ms: u64,
    pub(super) row_count: Option<u64>,
    pub(super) affected_rows: Option<u64>,
    pub(super) error_code: Option<String>,
    pub(super) error_summary: Option<String>,
}

impl DbProApp {
    pub(super) fn apply_runtime_events(&mut self) {
        let events: Vec<UiEvent> = self.task_bridge.drain_events().collect();
        for event in events {
            self.apply_runtime_event(event);
        }
    }

    /// Applies a single runtime event. Each arm delegates to a focused handler so
    /// the dispatch table stays readable and every event family is independently
    /// testable.
    /// Connection list refreshed; auto-select and auto-connect the first one when nothing is active.
    pub(super) fn on_connections_loaded(&mut self, connections: Vec<UiConnectionSummary>) {
        self.connection_lifecycle.connections_request_pending = false;
        self.connection_catalog.replace(connections);
        if self.connection_lifecycle.active_connection_id.is_none() {
            self.connection_lifecycle.active_connection_id = self
                .connection_catalog
                .connections
                .first()
                .map(|connection| connection.id.clone());
        }
        if !self.connection_lifecycle.connected && self.connection_lifecycle.pending_request.is_none() {
            if let Some(active) = self.active_connection().cloned() {
                self.connect_to_connection(&active);
            }
        }
        self.feedback.runtime_message = format!("Loaded {} connections", self.connection_catalog.connections.len());
    }

    /// Schema introspection result, revalidating the current schema/table/object selection.
    pub(super) fn on_schema_loaded(&mut self, request_id: RequestId, schema: UiSchemaSummary) {
        if self
            .schema_explorer
            .schema_request
            .is_some_and(|expected_request| expected_request != request_id)
        {
            return;
        }
        self.schema_explorer.schema_request = None;
        self.schema_explorer.schema_error = None;
        let refresh_selected_table = self.table_state.refresh_table_info_after_schema;
        self.table_state.refresh_table_info_after_schema = false;
        let mut schema = schema;
        schema.schemas.retain(|name| is_user_visible_schema(name));
        self.schema_explorer.schema_symbol_index = SchemaSymbolIndex::build(&schema);
        self.schema_explorer.schema = schema;
        self.schema_explorer.explorer_nav_cache = None;
        self.palette.search_index.invalidate();
        if self.schema_explorer.selected_schema.as_ref().is_none_or(|selected| {
            !self
                .schema_explorer
                .schema
                .schemas
                .iter()
                .any(|schema| schema == selected)
        }) {
            self.schema_explorer.selected_schema = self.schema_explorer.schema.schemas.first().cloned();
        }
        if self.schema_explorer.selected_table.as_ref().is_some_and(|table| {
            !self
                .schema_explorer
                .schema
                .tables
                .iter()
                .any(|candidate| candidate == table)
        }) {
            self.clear_missing_selected_table();
        }
        if !self.selected_schema_object_exists() {
            self.schema_explorer.selected_schema_object = None;
            if self.workspace.active_tab == WorkspaceTab::SchemaObject {
                self.activate_welcome_tab();
            }
        }
        self.feedback.runtime_message = format!(
            "Schema loaded · {} tables · {} views · {} triggers · {} functions",
            self.schema_explorer.schema.tables.len(),
            self.schema_explorer.schema.views.len(),
            self.schema_explorer.schema.triggers.len(),
            self.schema_explorer.schema.functions.len()
        );
        if refresh_selected_table
            && self.workspace.active_tab == WorkspaceTab::Table
            && self.schema_explorer.selected_table.is_some()
        {
            self.request_table_info();
        }
    }

    /// Drops the selected table after a refresh proves it no longer exists.
    fn clear_missing_selected_table(&mut self) {
        self.schema_explorer.selected_table = None;
        self.schema_explorer.selected_schema_object = None;
        self.table_state.table_info = None;
        self.table_state.table_ddl = None;
        self.table_state.table_info_error = None;
        self.table_state.table_ddl_error = None;
        self.table_state.ddl_execute_confirmation = false;
        self.table_state.ddl_execution_request = None;
        self.table_state.table_data_result = None;
        self.table_state.table_data_total_rows = None;
        self.table_state.table_data_offset = 0;
        self.table_state.table_data_filter_column.clear();
        self.table_state.table_data_filter_operator = UiTableFilterOperator::default();
        self.table_state.table_data_filter_value.clear();
        self.table_state.table_data_filters.clear();
        self.table_state.table_data_sorts.clear();
        self.table_state.table_data_error = None;
        self.table_state.table_info_request = None;
        self.table_state.table_ddl_request = None;
        self.table_state.table_data_request = None;
        if self.workspace.active_tab == WorkspaceTab::Table {
            self.activate_welcome_tab();
        }
    }

    /// Whether the selected schema object (view / trigger / function) still exists.
    fn selected_schema_object_exists(&self) -> bool {
        match self.schema_explorer.selected_schema_object.as_ref() {
            Some(SchemaObjectSelection::View(name)) => {
                self.schema_explorer.schema.views.iter().any(|view| &view.name == name)
            }
            Some(SchemaObjectSelection::Trigger(name)) => self
                .schema_explorer
                .schema
                .triggers
                .iter()
                .any(|trigger| &trigger.name == name),
            Some(SchemaObjectSelection::Function {
                name,
                identity_arguments,
            }) => self
                .schema_explorer
                .schema
                .functions
                .iter()
                .any(|function| &function.name == name && &function.identity_arguments == identity_arguments),
            None => true,
        }
    }

    pub(super) fn on_agent_completed(&mut self, _request_id: RequestId, provider: String, message: AgentMessage) {
        let provider_detail = format!("{provider} Responses API · SQL drafts stay unexecuted");
        self.agent.provider_label = provider;
        self.agent.provider_detail = provider_detail;
        self.agent.messages.push(message);
        self.feedback.runtime_message = "Agent response received".to_owned();
    }

    pub(super) fn on_agent_failed(&mut self, request_id: RequestId, message: String) {
        if let Some(session) = self
            .agent
            .sessions
            .values_mut()
            .find(|session| session.request_id == Some(request_id))
        {
            if !session.streaming_text.is_empty() {
                session.messages.push(AgentMessage {
                    role: AgentRole::Assistant,
                    content: std::mem::take(&mut session.streaming_text),
                    sql: None,
                    requires_confirmation: false,
                });
            }
            session.messages.push(AgentMessage {
                role: AgentRole::Assistant,
                content: message,
                sql: None,
                requires_confirmation: false,
            });
            super::agent_state::finish_agent_session(session, db_pro_core::domain::agent::AgentSessionState::Failed);
            self.feedback.runtime_message = "Agent workflow failed".to_owned();
        }
    }

    pub(super) fn on_agent_configured(&mut self, request_id: RequestId, provider: String, detail: String) {
        if self.agent.configure_request != Some(request_id) {
            return;
        }
        self.agent.configure_request = None;
        self.agent.provider_label = provider.clone();
        self.agent.provider_detail = detail;
        self.agent.settings_open = false;
        self.agent.api_key_draft.clear();
        self.agent.api_key_show_password = false;
        let message = format!("{provider} API key saved · provider active");
        self.feedback.runtime_message = message.clone();
        self.show_toast_success(message);
    }

    pub(super) fn on_agent_forgotten(&mut self, request_id: RequestId) {
        if self.agent.configure_request != Some(request_id) {
            return;
        }
        self.agent.configure_request = None;
        self.agent.provider_label = "Offline draft".to_owned();
        self.agent.provider_detail = "AI provider not configured · local drafts stay unexecuted".to_owned();
        self.agent.settings_open = false;
        self.agent.api_key_draft.clear();
        self.agent.api_key_show_password = false;
        let message = "API key forgotten · provider inactive".to_owned();
        self.feedback.runtime_message = message.clone();
        self.show_toast_success(message);
    }

    /// Table structure arrived: seed filter/sort defaults on first load.
    pub(super) fn on_table_info_loaded(&mut self, request_id: RequestId, table_info: UiTableInfo) {
        if self.table_state.table_info_request != Some(request_id) {
            return;
        }
        if self.table_state.table_data_filter_column.is_empty() {
            self.table_state.table_data_filter_column = table_info
                .columns
                .first()
                .map(|column| column.name.clone())
                .unwrap_or_default();
        }
        if self.table_state.table_data_sorts.is_empty() {
            if let Some(column) = table_info
                .primary_key
                .as_ref()
                .and_then(|columns| columns.first().cloned())
                .or_else(|| table_info.columns.first().map(|column| column.name.clone()))
            {
                self.table_state.table_data_sorts.push(UiTableDataSort {
                    column,
                    descending: false,
                });
            }
        }
        self.table_state.table_info = Some(table_info);
        self.invalidate_grid_row_caches();
        self.table_state.table_info_error = None;
        self.table_state.table_info_request = None;
        self.feedback.runtime_message = "Table structure loaded".to_owned();
    }

    pub(super) fn on_table_ddl_loaded(&mut self, request_id: RequestId, sql: String) {
        if self.table_state.table_ddl_request == Some(request_id) {
            self.table_state.table_ddl = Some(sql);
            self.table_state.ddl_execute_confirmation = false;
            self.table_state.table_ddl_error = None;
            self.table_state.table_ddl_request = None;
            self.feedback.runtime_message = "Table DDL loaded".to_owned();
        }
    }

    /// Table data arrived: seed filter/sort defaults on first load.
    pub(super) fn on_table_data_loaded(&mut self, request_id: RequestId, result: UiQueryResult, total_rows: u64) {
        if self.table_state.table_row_reload_request == Some(request_id) {
            self.on_table_row_reloaded(result);
            return;
        }
        if self.table_state.table_data_request != Some(request_id) {
            return;
        }
        if self.table_state.table_data_filter_column.is_empty() {
            self.table_state.table_data_filter_column = result
                .columns
                .first()
                .map(|column| column.name.clone())
                .unwrap_or_default();
        }
        if self.table_state.table_data_sorts.is_empty() {
            if let Some(column) = result.columns.first().map(|column| column.name.clone()) {
                self.table_state.table_data_sorts.push(UiTableDataSort {
                    column,
                    descending: false,
                });
            }
        }
        self.table_state.table_data_result = Some(result);
        self.invalidate_grid_row_caches();
        self.table_state.table_data_total_rows = Some(total_rows);
        if self.table_mutation.staged_changes.is_empty() {
            self.table_data.selected_cell = None;
            self.table_data.selected_row = None;
            self.table_data.selected_rows.clear();
            self.table_data.selection_anchor_row = None;
            self.table_data.selection_anchor_cell = None;
        }
        self.table_state.table_data_error = None;
        self.table_state.table_data_request = None;
        self.feedback.runtime_message = format!("Table data loaded · {total_rows} rows");
        if self.table_mutation.table_mutation_retry_after_reload {
            self.table_mutation.table_mutation_retry_after_reload = false;
            self.apply_staged_changes();
        }
    }

    pub(crate) fn on_table_row_reloaded(&mut self, result: UiQueryResult) {
        self.table_state.table_row_reload_request = None;
        let Some(identity) = self.table_state.table_row_reload_identity.take() else {
            return;
        };
        let Some(server_row) = result.rows.into_iter().next() else {
            self.feedback.runtime_message = "Row was deleted".to_owned();
            if self.table_mutation.table_mutation_retry_after_reload {
                self.table_mutation.table_mutation_retry_after_reload = false;
                self.table_mutation.table_mutation_retry_target = None;
            }
            return;
        };

        let mut replaced_row = false;
        if let Some(table_result) = self.table_state.table_data_result.as_mut() {
            let column_indexes: std::collections::HashMap<&str, usize> = table_result
                .columns
                .iter()
                .enumerate()
                .map(|(index, column)| (column.name.as_str(), index))
                .collect();
            if let Some(row_index) = table_result
                .rows
                .iter()
                .position(|row| Self::row_matches_identity(row, &column_indexes, &identity))
            {
                table_result.rows[row_index] = server_row;
                replaced_row = true;
            }
        }
        // Outside the borrow above: a replaced row invalidates both grid caches at once.
        if replaced_row {
            self.invalidate_grid_row_caches();
        }
        self.table_state.table_data_error = None;
        self.feedback.runtime_message = "Row reloaded from database".to_owned();
        if self.table_mutation.table_mutation_retry_after_reload {
            self.table_mutation.table_mutation_retry_after_reload = false;
            self.apply_staged_changes();
        }
    }

    pub(crate) fn row_matches_identity(
        row: &[UiCell],
        column_indexes: &std::collections::HashMap<&str, usize>,
        identity: &RowIdentity,
    ) -> bool {
        identity
            .original_pk_columns
            .iter()
            .zip(&identity.original_pk_values)
            .all(|(column, value)| {
                column_indexes
                    .get(column.as_str())
                    .and_then(|index| row.get(*index))
                    .is_some_and(|candidate| candidate == value)
            })
    }

    /// A native file picker returned (or was cancelled).
    pub(super) fn on_file_picked(&mut self, kind: &str, path: Option<String>) {
        if let Some(path) = path {
            if kind == "sqlite" {
                self.connection_dialog.draft.database = path;
            } else if kind == "ssh-key" {
                self.connection_dialog.draft.ssh_private_key = path;
            } else if kind == "backup" {
                self.overlay.backup_output_path = path;
            } else if kind == "restore" {
                self.overlay.restore_input_path = path;
            } else if kind == "workspace-folder" {
                self.open_workspace_folder(std::path::PathBuf::from(path));
            }
            self.connection_dialog.error.clear();
            self.connection_dialog.test_valid = false;
        } else if kind == "sqlite" || kind == "ssh-key" {
            self.connection_dialog.error = "File selection was cancelled".to_owned();
            self.connection_dialog.test_valid = false;
        } else if kind == "workspace-folder" {
            self.feedback.runtime_message = "Workspace folder selection was cancelled".to_owned();
        }
    }

    /// DDL applied; re-introspect so the tree and table view pick up the change.
    pub(super) fn on_ddl_completed(&mut self, request_id: RequestId, affected_rows: u64) {
        if self.table_state.ddl_execution_request == Some(request_id) {
            self.table_state.ddl_execution_request = None;
            self.table_state.ddl_execute_confirmation = false;
            self.table_state.table_ddl_error = None;
            self.table_state.refresh_table_info_after_schema = self.schema_explorer.selected_table.is_some();
            self.feedback.runtime_message = format!("DDL applied · {affected_rows} affected rows");
            if let Some(connection_id) = self.connection_lifecycle.active_connection_id.clone() {
                self.request_schema_introspection(connection_id, true);
            }
            if !self.database_operations.security_rls_table.trim().is_empty() {
                self.request_security_rls();
            }
            self.database_operations.security_rls_confirm_apply = false;
        }
    }

    /// Generic completion for connection, table-row and query operations.
    pub(super) fn on_operation_completed(&mut self, request_id: RequestId, operation: String) {
        let pending_connection_request = self.connection_lifecycle.pending_request == Some(request_id);
        if matches!(
            operation.as_str(),
            "connection.created" | "connection.updated" | "connection.deleted" | "connection.tested"
        ) && !pending_connection_request
        {
            return;
        }
        self.feedback.runtime_message = operation.clone();
        if pending_connection_request {
            self.connection_lifecycle.clear_pending_request();
        }
        if matches!(
            operation.as_str(),
            "create_role"
                | "drop_role"
                | "alter_role"
                | "update_role_password"
                | "grant_membership"
                | "revoke_membership"
                | "grant_privilege"
                | "revoke_privilege"
        ) {
            self.database_operations.security_drop_confirm = None;
            self.database_operations.security_password.clear();
            self.request_security_users();
            if let Some(role) = self.database_operations.security_selected_role.clone() {
                self.request_security_role_details(&role);
            }
        }
        if matches!(
            operation.as_str(),
            "connection.created" | "connection.updated" | "connection.deleted"
        ) {
            self.connection_lifecycle.connections_requested = false;
            self.request_connections_once();
        }
        if operation == "connection.tested" && pending_connection_request {
            self.connection_dialog.error.clear();
            self.refresh_connection_diagnostics(true, "Authentication succeeded");
            if self.connection_dialog.test_draft.as_ref() == Some(&self.connection_dialog.draft) {
                self.connection_dialog.test_valid = true;
                self.feedback.runtime_message = self
                    .connection_dialog
                    .diagnostics
                    .as_ref()
                    .map(|r| r.summary())
                    .unwrap_or_else(|| "Connection test succeeded".to_owned());
            } else {
                self.connection_dialog.test_valid = false;
                self.feedback.runtime_message = "Connection changed · test again before saving".to_owned();
            }
        }
        if operation.starts_with("table-row.") || operation == "table-changes.applied" {
            self.on_table_row_operation_completed(request_id);
        }
        if operation == "connection.created" || operation == "connection.updated" {
            self.connection_dialog
                .transition(super::connection::state::ConnectionDialogAction::Close);
            self.connection_dialog.editing_connection_id = None;
        }
        if operation.starts_with("query") || operation.starts_with("query-folder") {
            self.request_saved_queries_refresh();
        }
        if operation == "connection.deleted" {
            // `pending_connection_id` is the delete target (set by the confirm dialog).
            let deleted_id = self.connection_lifecycle.pending_connection_id.take();
            if let Some(ref id) = deleted_id {
                self.connection_lifecycle.clear_connection_error(id);
            }
            let deleted_was_active = deleted_id
                .as_ref()
                .is_some_and(|id| self.connection_lifecycle.active_connection_id.as_deref() == Some(id.as_str()));
            // Only tear down the live session when the deleted connection was active.
            // Deleting a sibling must not force a reconnect / schema reload of the open one.
            if deleted_was_active {
                self.connection_lifecycle.active_connection_id = None;
                self.connection_lifecycle.connected = false;
            }
        }
    }

    /// Row mutation follow-up: finalise a staged apply, or refresh the grid.
    fn on_table_row_operation_completed(&mut self, request_id: RequestId) {
        self.table_data.data_editing_cell = None;
        self.table_data.data_edit_value.clear();
        self.table_data.data_edit_error = None;
        self.table_data.data_delete_confirmation = false;
        if self.table_mutation.staged_apply_request == Some(request_id) {
            self.staged_apply_completed();
        } else if self.table_mutation.table_mutation_request == Some(request_id) {
            self.table_mutation.table_mutation_request = None;
            self.table_state.table_data_result = None;
            self.table_state.table_data_total_rows = None;
            self.table_state.table_data_error = None;
            self.table_state.table_data_request = None;
            if self.workspace.active_tab == WorkspaceTab::Table {
                self.request_table_data();
            }
        }
    }

    /// Re-reads saved queries for the active connection.
    fn request_saved_queries_refresh(&mut self) {
        if let Some(connection_id) = self.connection_lifecycle.active_connection_id.clone() {
            let request_id = self.task_bridge.next_request_id();
            self.dispatch_command(UiCommand::ListSavedQueries {
                request_id,
                connection_id,
            });
        }
    }

    /// Connection established: load schema, saved queries and query folders.
    pub(super) fn on_connected(&mut self, request_id: RequestId, connection_id: String) {
        if self
            .connection_lifecycle
            .pending_request
            .is_some_and(|expected_request| expected_request != request_id)
        {
            return;
        }
        self.connection_lifecycle.pending_request = None;
        self.connection_lifecycle.pending_connection_id = None;
        self.connection_lifecycle.active_connection_id = Some(connection_id.clone());
        self.connection_lifecycle.connected = true;
        self.connection_lifecycle.clear_connection_error(&connection_id);
        self.feedback.runtime_message = "Connection established".to_owned();
        if let Some(connection_id) = self.connection_lifecycle.active_connection_id.clone() {
            self.request_schema_introspection(connection_id.clone(), false);
            let request_id = self.task_bridge.next_request_id();
            self.dispatch_command(UiCommand::ListSavedQueries {
                request_id,
                connection_id: connection_id.clone(),
            });
            let request_id = self.task_bridge.next_request_id();
            self.dispatch_command(UiCommand::ListQueryFolders {
                request_id,
                connection_id,
            });
        }
    }

    // Query completion/history/prediction: `events_query.rs`.
    // Shortcuts / dispatch: `events_query_dispatch.rs`.

    pub(super) fn on_table_changes_failed(
        &mut self,
        request_id: RequestId,
        code: String,
        message: String,
        statement_index: usize,
        rolled_back: bool,
    ) {
        if self.table_mutation.staged_apply_request == Some(request_id) {
            self.staged_apply_failed(statement_index, &code, &message, rolled_back);
        }
    }
}

#[cfg(test)]
mod row_reload_tests {
    use super::*;
    use crate::{UiColumn, UiTableColumn};

    fn row_result(name: &str) -> UiQueryResult {
        UiQueryResult {
            columns: vec![
                UiColumn {
                    name: "id".to_owned(),
                    data_type: "integer".to_owned(),
                    nullable: false,
                },
                UiColumn {
                    name: "name".to_owned(),
                    data_type: "text".to_owned(),
                    nullable: false,
                },
            ],
            rows: vec![vec![UiCell::Number("7".to_owned()), UiCell::Text(name.to_owned())]],
            row_count: 1,
            duration_ms: 0,
        }
    }

    fn row_reload_app() -> DbProApp {
        DbProApp {
            table_state: TableState {
                table_info: Some(UiTableInfo {
                    schema: "public".to_owned(),
                    name: "customers".to_owned(),
                    row_count: Some(1),
                    columns: vec![
                        UiTableColumn {
                            name: "id".to_owned(),
                            data_type: "integer".to_owned(),
                            nullable: false,
                            default: None,
                            is_primary_key: true,
                            ..Default::default()
                        },
                        UiTableColumn {
                            name: "name".to_owned(),
                            data_type: "text".to_owned(),
                            nullable: false,
                            default: None,
                            is_primary_key: false,
                            ..Default::default()
                        },
                    ],
                    primary_key: Some(vec!["id".to_owned()]),
                    indexes: Vec::new(),
                    foreign_keys: Vec::new(),
                    check_constraints: Vec::new(),
                    dependencies: Vec::new(),
                }),
                table_data_result: Some(row_result("local server value")),
                table_row_reload_request: Some(RequestId(9)),
                table_row_reload_identity: Some(RowIdentity {
                    original_pk_columns: vec!["id".to_owned()],
                    original_pk_values: vec![UiCell::Number("7".to_owned())],
                }),
                ..Default::default()
            },
            ..Default::default()
        }
    }

    #[test]
    fn row_reload_merges_server_values_by_original_identity() {
        let mut app = row_reload_app();
        app.on_table_data_loaded(RequestId(9), row_result("fresh server value"), 1);

        let result = app
            .table_state
            .table_data_result
            .expect("table result should remain visible");
        assert_eq!(result.rows[0][1], UiCell::Text("fresh server value".to_owned()));
        assert!(app.table_state.table_row_reload_request.is_none());
    }

    #[test]
    fn row_reload_reports_deleted_row_without_dropping_local_result() {
        let mut app = row_reload_app();
        let empty = UiQueryResult {
            columns: row_result("unused").columns,
            rows: Vec::new(),
            row_count: 0,
            duration_ms: 0,
        };
        app.on_table_data_loaded(RequestId(9), empty, 0);

        assert_eq!(app.feedback.runtime_message, "Row was deleted");
        assert!(app.table_state.table_data_result.is_some());
    }

    #[test]
    fn deleting_non_active_connection_preserves_active_session() {
        let mut app = DbProApp {
            connection_lifecycle: ConnectionLifecycleState {
                connected: true,
                active_connection_id: Some("conn-a".to_owned()),
                pending_request: Some(RequestId(11)),
                pending_connection_id: Some("conn-b".to_owned()),
                failed_connection_ids: ["conn-a".to_owned(), "conn-b".to_owned()].into_iter().collect(),
                errors: [
                    ("conn-a".to_owned(), "stale".to_owned()),
                    ("conn-b".to_owned(), "gone".to_owned()),
                ]
                .into_iter()
                .collect(),
                ..Default::default()
            },
            ..Default::default()
        };

        app.on_operation_completed(RequestId(11), "connection.deleted".to_owned());

        assert_eq!(app.connection_lifecycle.active_connection_id.as_deref(), Some("conn-a"));
        assert!(app.connection_lifecycle.connected);
        assert!(app.connection_lifecycle.pending_connection_id.is_none());
        assert!(app.connection_lifecycle.failed_connection_ids.contains("conn-a"));
        assert!(!app.connection_lifecycle.failed_connection_ids.contains("conn-b"));
        assert_eq!(
            app.connection_lifecycle.errors.get("conn-a").map(String::as_str),
            Some("stale")
        );
        assert!(!app.connection_lifecycle.errors.contains_key("conn-b"));
    }

    #[test]
    fn deleting_active_connection_clears_session() {
        let mut app = DbProApp {
            connection_lifecycle: ConnectionLifecycleState {
                connected: true,
                active_connection_id: Some("conn-a".to_owned()),
                pending_request: Some(RequestId(12)),
                pending_connection_id: Some("conn-a".to_owned()),
                failed_connection_ids: ["conn-a".to_owned()].into_iter().collect(),
                ..Default::default()
            },
            ..Default::default()
        };

        app.on_operation_completed(RequestId(12), "connection.deleted".to_owned());

        assert!(app.connection_lifecycle.active_connection_id.is_none());
        assert!(!app.connection_lifecycle.connected);
        assert!(app.connection_lifecycle.pending_connection_id.is_none());
        assert!(app.connection_lifecycle.failed_connection_ids.is_empty());
    }
}
