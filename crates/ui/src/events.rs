use super::*;
use crate::RequestId;

/// A statement (or script) the classifier rates `Destructive`, held until the user
/// confirms the exact text the prompt displayed.
#[derive(Debug, Clone)]
pub(super) struct PendingDestructiveRun {
    pub(super) sql: String,
    pub(super) execution_range: (usize, usize),
    pub(super) version: u64,
    pub(super) all_statements: bool,
}

struct QueryHistoryRecord {
    sql: String,
    connection_id: Option<String>,
    schema: Option<String>,
    started_at: String,
    status: UiQueryHistoryStatus,
    duration_ms: u64,
    row_count: Option<u64>,
    affected_rows: Option<u64>,
    error_code: Option<String>,
    error_summary: Option<String>,
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
    pub(crate) fn apply_runtime_event(&mut self, event: UiEvent) {
        match event {
            UiEvent::ConnectionsLoaded { connections, .. } => self.on_connections_loaded(connections),
            UiEvent::SavedQueriesLoaded { queries, .. } => self.saved_queries = queries,
            UiEvent::QueryFoldersLoaded { folders, .. } => self.query_folders = folders,
            UiEvent::SchemaLoaded { request_id, schema } => self.on_schema_loaded(request_id, schema),
            UiEvent::AgentCompleted {
                request_id,
                provider,
                message,
            } => self.on_agent_completed(request_id, provider, message),
            UiEvent::AgentProviderReady { provider, detail } => {
                self.agent_provider_label = provider;
                self.agent_provider_detail = detail;
            }
            UiEvent::AgentFailed { request_id, message } => self.on_agent_failed(request_id, message),
            UiEvent::AgentToolCompleted { .. } | UiEvent::AgentToolFailed { .. } => {}
            UiEvent::AgentWorkflow { event, .. } => self.on_agent_workflow_event(event),
            UiEvent::AgentConfigured {
                request_id,
                provider,
                detail,
            } => self.on_agent_configured(request_id, provider, detail),
            UiEvent::TableInfoLoaded { request_id, table_info } => self.on_table_info_loaded(request_id, table_info),
            UiEvent::TableDdlLoaded { request_id, sql } => self.on_table_ddl_loaded(request_id, sql),
            UiEvent::TableDataLoaded {
                request_id,
                result,
                total_rows,
            } => self.on_table_data_loaded(request_id, result, total_rows),
            UiEvent::FilePicked { kind, path, .. } => self.on_file_picked(&kind, path),
            UiEvent::OperationProgress { operation, status, .. } => {
                self.runtime_message = format!("{operation}: {status}");
            }
            UiEvent::BackupCompleted {
                output_path,
                size_bytes,
                ..
            } => {
                self.runtime_message = format!("Backup completed · {output_path} · {size_bytes} bytes");
            }
            UiEvent::DdlCompleted {
                request_id,
                affected_rows,
            } => self.on_ddl_completed(request_id, affected_rows),
            UiEvent::OperationCompleted { request_id, operation } => self.on_operation_completed(request_id, operation),
            UiEvent::TableChangesFailed {
                request_id,
                code,
                message,
                statement_index,
                rolled_back,
            } => self.on_table_changes_failed(request_id, code, message, statement_index, rolled_back),
            UiEvent::Connected {
                request_id,
                connection_id,
            } => self.on_connected(request_id, connection_id),
            UiEvent::QueryQueued { request_id } => {
                self.runtime_message = format!("Query queued · request {}", request_id.0);
            }
            UiEvent::QueryCompleted { request_id, result } => self.on_query_completed(request_id, result),
            UiEvent::QueryMultiCompleted { request_id, output } => self.on_query_multi_completed(request_id, output),
            UiEvent::QuerySaved { request_id, query } => self.on_query_saved(request_id, query),
            UiEvent::ExplainCompleted { request_id, plan } => self.on_explain_completed(request_id, plan),
            UiEvent::QueryCancelled { request_id } => self.on_query_cancelled(request_id),
            UiEvent::QueryFailed { request_id, message } => self.on_query_failed(request_id, message, None, None),
            UiEvent::QueryFailedDetailed {
                request_id,
                code,
                message,
                position,
            } => self.on_query_failed(request_id, message, position, Some(code)),
            UiEvent::SqlPredictionReady {
                request_id,
                document_id,
                document_version,
                anchor,
                replacement_range,
                prediction,
            } => self.on_sql_prediction_ready(
                request_id,
                document_id,
                document_version,
                anchor,
                replacement_range,
                prediction,
            ),
            UiEvent::SqlPredictionFailed {
                request_id,
                document_id,
                document_version,
                anchor,
                replacement_range,
                message,
            } => self.on_sql_prediction_failed(
                request_id,
                document_id,
                document_version,
                anchor,
                replacement_range,
                message,
            ),
        }
    }

    /// Connection list refreshed; auto-select and auto-connect the first one when nothing is active.
    fn on_connections_loaded(&mut self, connections: Vec<UiConnectionSummary>) {
        self.connections_request_pending = false;
        self.connections = connections;
        if self.active_connection_id.is_none() {
            self.active_connection_id = self.connections.first().map(|connection| connection.id.clone());
        }
        if !self.connected && self.pending_connection_request.is_none() {
            if let Some(active) = self.active_connection().cloned() {
                self.connect_to_connection(&active);
            }
        }
        self.runtime_message = format!("Loaded {} connections", self.connections.len());
    }

    /// Schema introspection result, revalidating the current schema/table/object selection.
    fn on_schema_loaded(&mut self, request_id: RequestId, schema: UiSchemaSummary) {
        if self
            .schema_request
            .is_some_and(|expected_request| expected_request != request_id)
        {
            return;
        }
        self.schema_request = None;
        self.schema_error = None;
        let refresh_selected_table = self.refresh_table_info_after_schema;
        self.refresh_table_info_after_schema = false;
        self.schema = schema;
        if self
            .selected_schema
            .as_ref()
            .is_none_or(|selected| !self.schema.schemas.iter().any(|schema| schema == selected))
        {
            self.selected_schema = self.schema.schemas.first().cloned();
        }
        if self
            .selected_table
            .as_ref()
            .is_some_and(|table| !self.schema.tables.iter().any(|candidate| candidate == table))
        {
            self.clear_missing_selected_table();
        }
        if !self.selected_schema_object_exists() {
            self.selected_schema_object = None;
            if self.active_tab == WorkspaceTab::SchemaObject {
                self.activate_welcome_tab();
            }
        }
        self.runtime_message = format!(
            "Schema loaded · {} tables · {} views · {} triggers · {} functions",
            self.schema.tables.len(),
            self.schema.views.len(),
            self.schema.triggers.len(),
            self.schema.functions.len()
        );
        if refresh_selected_table && self.active_tab == WorkspaceTab::Table && self.selected_table.is_some() {
            self.request_table_info();
        }
    }

    /// Drops the selected table after a refresh proves it no longer exists.
    fn clear_missing_selected_table(&mut self) {
        self.selected_table = None;
        self.selected_schema_object = None;
        self.table_info = None;
        self.table_ddl = None;
        self.table_info_error = None;
        self.table_ddl_error = None;
        self.ddl_execute_confirmation = false;
        self.ddl_execution_request = None;
        self.table_data_result = None;
        self.table_data_total_rows = None;
        self.table_data_offset = 0;
        self.table_data_filter_column.clear();
        self.table_data_filter_operator = UiTableFilterOperator::default();
        self.table_data_filter_value.clear();
        self.table_data_filters.clear();
        self.table_data_sorts.clear();
        self.table_data_error = None;
        self.table_info_request = None;
        self.table_ddl_request = None;
        self.table_data_request = None;
        if self.active_tab == WorkspaceTab::Table {
            self.activate_welcome_tab();
        }
    }

    /// Whether the selected schema object (view / trigger / function) still exists.
    fn selected_schema_object_exists(&self) -> bool {
        match self.selected_schema_object.as_ref() {
            Some(SchemaObjectSelection::View(name)) => self.schema.views.iter().any(|view| &view.name == name),
            Some(SchemaObjectSelection::Trigger(name)) => {
                self.schema.triggers.iter().any(|trigger| &trigger.name == name)
            }
            Some(SchemaObjectSelection::Function {
                name,
                identity_arguments,
            }) => self
                .schema
                .functions
                .iter()
                .any(|function| &function.name == name && &function.identity_arguments == identity_arguments),
            None => true,
        }
    }

    fn on_agent_completed(&mut self, _request_id: RequestId, provider: String, message: AgentMessage) {
        let provider_detail = format!("{provider} Responses API · SQL drafts stay unexecuted");
        self.agent_provider_label = provider;
        self.agent_provider_detail = provider_detail;
        self.agent_messages.push(message);
        self.runtime_message = "Agent response received".to_owned();
    }

    fn on_agent_failed(&mut self, request_id: RequestId, message: String) {
        if let Some(session) = self
            .agent_sessions
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
            self.runtime_message = "Agent workflow failed".to_owned();
        }
    }

    fn on_agent_configured(&mut self, request_id: RequestId, provider: String, detail: String) {
        if self.agent_configure_request != Some(request_id) {
            return;
        }
        self.agent_configure_request = None;
        self.agent_provider_label = provider.clone();
        self.agent_provider_detail = detail;
        self.agent_settings_open = false;
        self.agent_api_key_draft.clear();
        let message = format!("{provider} API key saved · provider active");
        self.runtime_message = message.clone();
        self.show_toast_success(message);
    }

    /// Table structure arrived: seed filter/sort defaults on first load.
    fn on_table_info_loaded(&mut self, request_id: RequestId, table_info: UiTableInfo) {
        if self.table_info_request != Some(request_id) {
            return;
        }
        if self.table_data_filter_column.is_empty() {
            self.table_data_filter_column = table_info
                .columns
                .first()
                .map(|column| column.name.clone())
                .unwrap_or_default();
        }
        if self.table_data_sorts.is_empty() {
            if let Some(column) = table_info
                .primary_key
                .as_ref()
                .and_then(|columns| columns.first().cloned())
                .or_else(|| table_info.columns.first().map(|column| column.name.clone()))
            {
                self.table_data_sorts.push(UiTableDataSort {
                    column,
                    descending: false,
                });
            }
        }
        self.table_info = Some(table_info);
        self.invalidate_grid_row_caches();
        self.table_info_error = None;
        self.table_info_request = None;
        self.runtime_message = "Table structure loaded".to_owned();
    }

    fn on_table_ddl_loaded(&mut self, request_id: RequestId, sql: String) {
        if self.table_ddl_request == Some(request_id) {
            self.table_ddl = Some(sql);
            self.ddl_execute_confirmation = false;
            self.table_ddl_error = None;
            self.table_ddl_request = None;
            self.runtime_message = "Table DDL loaded".to_owned();
        }
    }

    /// Table data arrived: seed filter/sort defaults on first load.
    fn on_table_data_loaded(&mut self, request_id: RequestId, result: UiQueryResult, total_rows: u64) {
        if self.table_row_reload_request == Some(request_id) {
            self.on_table_row_reloaded(result);
            return;
        }
        if self.table_data_request != Some(request_id) {
            return;
        }
        if self.table_data_filter_column.is_empty() {
            self.table_data_filter_column = result
                .columns
                .first()
                .map(|column| column.name.clone())
                .unwrap_or_default();
        }
        if self.table_data_sorts.is_empty() {
            if let Some(column) = result.columns.first().map(|column| column.name.clone()) {
                self.table_data_sorts.push(UiTableDataSort {
                    column,
                    descending: false,
                });
            }
        }
        self.table_data_result = Some(result);
        self.invalidate_grid_row_caches();
        self.table_data_total_rows = Some(total_rows);
        if self.staged_changes.is_empty() {
            self.selected_cell = None;
            self.selected_row = None;
            self.selected_rows.clear();
            self.selection_anchor_row = None;
            self.selection_anchor_cell = None;
        }
        self.table_data_error = None;
        self.table_data_request = None;
        self.runtime_message = format!("Table data loaded · {total_rows} rows");
        if self.table_mutation_retry_after_reload {
            self.table_mutation_retry_after_reload = false;
            self.apply_staged_changes();
        }
    }

    pub(crate) fn on_table_row_reloaded(&mut self, result: UiQueryResult) {
        self.table_row_reload_request = None;
        let Some(identity) = self.table_row_reload_identity.take() else {
            return;
        };
        let Some(server_row) = result.rows.into_iter().next() else {
            self.runtime_message = "Row was deleted".to_owned();
            if self.table_mutation_retry_after_reload {
                self.table_mutation_retry_after_reload = false;
                self.table_mutation_retry_target = None;
            }
            return;
        };

        let mut replaced_row = false;
        if let Some(table_result) = self.table_data_result.as_mut() {
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
        self.table_data_error = None;
        self.runtime_message = "Row reloaded from database".to_owned();
        if self.table_mutation_retry_after_reload {
            self.table_mutation_retry_after_reload = false;
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
    fn on_file_picked(&mut self, kind: &str, path: Option<String>) {
        if let Some(path) = path {
            if kind == "sqlite" {
                self.connection_draft.database = path;
            } else if kind == "ssh-key" {
                self.connection_draft.ssh_private_key = path;
            } else if kind == "backup" {
                self.backup_output_path = path;
            } else if kind == "restore" {
                self.restore_input_path = path;
            } else if kind == "workspace-folder" {
                self.open_workspace_folder(std::path::PathBuf::from(path));
            }
            self.connection_error.clear();
            self.connection_test_valid = false;
        } else if kind == "sqlite" || kind == "ssh-key" {
            self.connection_error = "File selection was cancelled".to_owned();
            self.connection_test_valid = false;
        } else if kind == "workspace-folder" {
            self.runtime_message = "Workspace folder selection was cancelled".to_owned();
        }
    }

    /// DDL applied; re-introspect so the tree and table view pick up the change.
    fn on_ddl_completed(&mut self, request_id: RequestId, affected_rows: u64) {
        if self.ddl_execution_request == Some(request_id) {
            self.ddl_execution_request = None;
            self.ddl_execute_confirmation = false;
            self.table_ddl_error = None;
            self.refresh_table_info_after_schema = self.selected_table.is_some();
            self.runtime_message = format!("DDL applied · {affected_rows} affected rows");
            if let Some(connection_id) = self.active_connection_id.clone() {
                self.request_schema_introspection(connection_id, true);
            }
        }
    }

    /// Generic completion for connection, table-row and query operations.
    fn on_operation_completed(&mut self, request_id: RequestId, operation: String) {
        let pending_connection_request = self.pending_connection_request == Some(request_id);
        if matches!(
            operation.as_str(),
            "connection.created" | "connection.updated" | "connection.deleted" | "connection.tested"
        ) && !pending_connection_request
        {
            return;
        }
        self.runtime_message = operation.clone();
        if pending_connection_request {
            self.pending_connection_request = None;
        }
        if matches!(
            operation.as_str(),
            "connection.created" | "connection.updated" | "connection.deleted"
        ) {
            self.connections_requested = false;
            self.request_connections_once();
        }
        if operation == "connection.tested" && pending_connection_request {
            self.connection_error.clear();
            if self.connection_test_draft.as_ref() == Some(&self.connection_draft) {
                self.connection_test_valid = true;
                self.runtime_message = "Connection test succeeded".to_owned();
            } else {
                self.connection_test_valid = false;
                self.runtime_message = "Connection changed · test again before saving".to_owned();
            }
        }
        if operation.starts_with("table-row.") || operation == "table-changes.applied" {
            self.on_table_row_operation_completed(request_id);
        }
        if operation == "connection.created" || operation == "connection.updated" {
            self.connection_dialog_open = false;
            self.editing_connection_id = None;
        }
        if operation.starts_with("query") || operation.starts_with("query-folder") {
            self.request_saved_queries_refresh();
        }
        if operation == "connection.deleted" {
            self.active_connection_id = None;
            self.connected = false;
        }
    }

    /// Row mutation follow-up: finalise a staged apply, or refresh the grid.
    fn on_table_row_operation_completed(&mut self, request_id: RequestId) {
        self.data_editing_cell = None;
        self.data_edit_value.clear();
        self.data_edit_error = None;
        self.data_delete_confirmation = false;
        if self.staged_apply_request == Some(request_id) {
            self.staged_apply_completed();
        } else if self.table_mutation_request == Some(request_id) {
            self.table_mutation_request = None;
            self.table_data_result = None;
            self.table_data_total_rows = None;
            self.table_data_error = None;
            self.table_data_request = None;
            if self.active_tab == WorkspaceTab::Table {
                self.request_table_data();
            }
        }
    }

    /// Re-reads saved queries for the active connection.
    fn request_saved_queries_refresh(&mut self) {
        if let Some(connection_id) = self.active_connection_id.clone() {
            let request_id = self.task_bridge.next_request_id();
            self.dispatch_command(UiCommand::ListSavedQueries {
                request_id,
                connection_id,
            });
        }
    }

    /// Connection established: load schema, saved queries and query folders.
    fn on_connected(&mut self, request_id: RequestId, connection_id: String) {
        if self
            .pending_connection_request
            .is_some_and(|expected_request| expected_request != request_id)
        {
            return;
        }
        self.pending_connection_request = None;
        self.pending_connection_id = None;
        self.active_connection_id = Some(connection_id.clone());
        self.connected = true;
        self.connection_errors.remove(&connection_id);
        self.failed_connection_ids.remove(&connection_id);
        self.runtime_message = "Connection established".to_owned();
        if let Some(connection_id) = self.active_connection_id.clone() {
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

    fn on_query_completed(&mut self, request_id: RequestId, result: UiQueryResult) {
        let target_doc_id = self.query_document_requests.remove(&request_id);
        // A new result set replaces the rows behind the grid, so nothing the projection cache holds
        // may survive it.
        self.invalidate_grid_projection();
        let mut history = None;
        if let Some(doc_id) = &target_doc_id {
            if let Some(doc) = self.query_documents.iter_mut().find(|d| &d.id == doc_id) {
                let (started_at, duration_ms) = doc.take_execution_timing(result.duration_ms);
                history = Some((
                    doc.executing_sql.clone().unwrap_or_else(|| doc.text().to_owned()),
                    doc.connection_id.clone(),
                    doc.schema.clone(),
                    started_at,
                    duration_ms,
                ));
                doc.query_result = Some(result.clone());
                doc.query_results = vec![result.clone()];
                doc.active_result_index = 0;
                doc.execution_output = Some(UiQueryExecutionOutput {
                    statements: vec![UiStatementOutput {
                        statement_index: 0,
                        result_set: Some(result.clone()),
                        affected_rows: None,
                        duration_ms: result.duration_ms,
                        message: None,
                        error: None,
                    }],
                    total_duration_ms: duration_ms,
                });
                doc.execution_state = QueryExecutionState::Idle;
                doc.executing_range = None;
                doc.executing_sql = None;
                doc.executing_version = None;
                doc.execution_diagnostic = None;
                doc.query_messages
                    .push(format!("Query completed · {} rows", result.row_count));
            }
        }
        if let Some((sql, connection_id, schema, started_at, duration_ms)) = history {
            self.record_query_history(QueryHistoryRecord {
                sql,
                connection_id,
                schema,
                started_at,
                status: UiQueryHistoryStatus::Success,
                duration_ms,
                row_count: Some(result.row_count),
                affected_rows: None,
                error_code: None,
                error_summary: None,
            });
        }
        let is_active_doc = self
            .query_documents
            .get(self.active_query_document)
            .is_some_and(|d| target_doc_id.as_ref() == Some(&d.id));

        if is_active_doc {
            self.runtime_message = format!("Query completed · {} rows", result.row_count);
            self.grid_sort_column = None;
            self.grid_column_widths = vec![180.0; result.columns.len()];
            self.selected_cell = None;
            self.selected_row = None;
            self.selected_rows.clear();
            self.selection_anchor_row = None;
            self.selection_anchor_cell = None;
            self.copy_status.clear();
            if let Some(doc_id) = target_doc_id.as_deref() {
                self.set_query_output_tab(doc_id, OutputTab::Results);
            }
        }
    }

    fn on_query_multi_completed(&mut self, request_id: RequestId, output: UiQueryExecutionOutput) {
        let target_doc_id = self.query_document_requests.remove(&request_id);
        let Some(doc_id) = target_doc_id else {
            return;
        };
        // Same as the single-statement path: the result sets behind the grid are being replaced.
        self.invalidate_grid_projection();
        let mut history = None;
        if let Some(doc) = self.query_documents.iter_mut().find(|doc| doc.id == doc_id) {
            let (started_at, duration_ms) = doc.take_execution_timing(output.total_duration_ms);
            history = Some((
                doc.executing_sql.clone().unwrap_or_else(|| doc.text().to_owned()),
                doc.connection_id.clone(),
                doc.schema.clone(),
                started_at,
                duration_ms,
            ));
            doc.query_results = output
                .statements
                .iter()
                .filter_map(|statement| statement.result_set.clone())
                .collect();
            doc.query_result = doc.query_results.first().cloned();
            doc.active_result_index = 0;
            doc.execution_output = Some(output.clone());
            doc.execution_state = if output.statements.iter().any(|statement| statement.error.is_some()) {
                QueryExecutionState::Failed
            } else {
                QueryExecutionState::Idle
            };
            let execution_range = doc.executing_range;
            let execution_sql = doc.executing_sql.clone();
            let execution_version = doc.executing_version;
            doc.executing_range = None;
            doc.executing_sql = None;
            doc.executing_version = None;
            doc.execution_diagnostic = output.statements.iter().find_map(|statement| {
                let error = statement.error.as_ref()?;
                if execution_version != Some(doc.buffer.version()) {
                    return None;
                }
                let fallback_range = execution_range.unwrap_or((0, doc.buffer.len_bytes()));
                let (statement_sql, statement_range) =
                    doc.analysis.statements.get(statement.statement_index).map_or_else(
                        || (execution_sql.as_deref().unwrap_or_default(), fallback_range),
                        |parsed| (parsed.text.as_str(), parsed.range),
                    );
                super::query_view::database_error_diagnostic(
                    &error.message,
                    statement_sql,
                    statement_range,
                    error.position,
                    Some(error.code.as_str()),
                )
            });
            for statement in &output.statements {
                if let Some(message) = &statement.message {
                    doc.query_messages.push(message.clone());
                }
                if let Some(error) = &statement.error {
                    doc.query_messages.push(format!(
                        "Statement {} failed · {}",
                        statement.statement_index + 1,
                        error.message
                    ));
                }
            }
        }
        if let Some((sql, connection_id, schema, started_at, duration_ms)) = history {
            let failed = output.statements.iter().any(|statement| statement.error.is_some());
            self.record_query_history(QueryHistoryRecord {
                sql,
                connection_id,
                schema,
                started_at,
                status: if failed {
                    UiQueryHistoryStatus::Failed
                } else {
                    UiQueryHistoryStatus::Success
                },
                duration_ms,
                row_count: Some(
                    output
                        .statements
                        .iter()
                        .filter_map(|statement| statement.result_set.as_ref().map(|result| result.row_count))
                        .sum(),
                ),
                affected_rows: Some(
                    output
                        .statements
                        .iter()
                        .filter_map(|statement| statement.affected_rows)
                        .sum(),
                ),
                error_code: None,
                error_summary: output
                    .statements
                    .iter()
                    .find_map(|statement| statement.error.as_ref().map(|error| error.message.clone())),
            });
        }
        let is_active_doc = self
            .query_documents
            .get(self.active_query_document)
            .is_some_and(|document| document.id == doc_id);
        if is_active_doc {
            self.runtime_message = format!(
                "Script completed · {} result{} · {} ms",
                self.query_documents
                    .get(self.active_query_document)
                    .map_or(0, |document| document.query_results.len()),
                if self.active_query_result_count() == 1 { "" } else { "s" },
                output.total_duration_ms,
            );
            self.set_query_output_tab(&doc_id, OutputTab::Results);
            self.grid_sort_column = None;
            self.selected_cell = None;
            self.selected_row = None;
            self.selected_rows.clear();
        }
    }

    fn on_query_saved(&mut self, request_id: RequestId, query: UiSavedQuerySummary) {
        let document_id = self.query_save_requests.remove(&request_id);
        let mut close_index = None;
        if let Some(document_id) = document_id {
            if let Some((index, doc)) = self
                .query_documents
                .iter_mut()
                .enumerate()
                .find(|(_, doc)| doc.id == document_id)
            {
                doc.saved_query_id = Some(query.id.clone());
                doc.mark_saved();
                if self.pending_close_after_save == Some(index) {
                    close_index = Some(index);
                }
            }
        }
        self.saved_queries.retain(|saved| saved.id != query.id);
        self.saved_queries.push(query);
        self.runtime_message = "Query saved".to_owned();
        if let Some(index) = close_index {
            self.pending_close_after_save = None;
            self.close_query_document(index);
        }
    }

    fn record_query_history(&mut self, record: QueryHistoryRecord) {
        self.query_history_entries.push(UiQueryHistoryEntry {
            id: uuid::Uuid::new_v4().to_string(),
            sql: record.sql,
            connection_id: record.connection_id,
            schema: record.schema,
            started_at: record.started_at,
            duration_ms: record.duration_ms,
            status: record.status,
            row_count: record.row_count,
            affected_rows: record.affected_rows,
            error_code: record.error_code,
            error_summary: record.error_summary,
        });
        const QUERY_HISTORY_RETENTION: usize = 500;
        let excess = self.query_history_entries.len().saturating_sub(QUERY_HISTORY_RETENTION);
        if excess > 0 {
            self.query_history_entries.drain(..excess);
        }
    }

    fn on_explain_completed(&mut self, request_id: RequestId, plan: String) {
        let Some(doc_index) = self
            .query_documents
            .iter()
            .position(|doc| doc.explain_request == Some(request_id))
        else {
            return;
        };
        let doc_id = self.query_documents[doc_index].id.clone();
        self.set_query_output_tab(&doc_id, OutputTab::Explain);
        if let Some(doc) = self.query_documents.get_mut(doc_index) {
            doc.explain_request = None;
            doc.explain_plan = Some(plan);
            self.runtime_message = "Query plan ready".to_owned();
            doc.query_messages.push(self.runtime_message.clone());
        }
    }

    fn on_query_cancelled(&mut self, request_id: RequestId) {
        let target_doc_id = self.query_document_requests.remove(&request_id);
        let mut history = None;
        if let Some(doc_id) = &target_doc_id {
            if let Some(doc) = self.query_documents.iter_mut().find(|d| &d.id == doc_id) {
                let (started_at, duration_ms) = doc.take_execution_timing(0);
                history = Some((
                    doc.executing_sql.clone().unwrap_or_else(|| doc.text().to_owned()),
                    doc.connection_id.clone(),
                    doc.schema.clone(),
                    started_at,
                    duration_ms,
                ));
                doc.execution_state = QueryExecutionState::Idle;
                doc.executing_range = None;
                doc.executing_sql = None;
                doc.executing_version = None;
                doc.execution_diagnostic = None;
                doc.query_messages.push("Query cancelled".to_owned());
            }
        }
        if let Some((sql, connection_id, schema, started_at, duration_ms)) = history {
            self.record_query_history(QueryHistoryRecord {
                sql,
                connection_id,
                schema,
                started_at,
                status: UiQueryHistoryStatus::Cancelled,
                duration_ms,
                row_count: None,
                affected_rows: None,
                error_code: None,
                error_summary: Some("Query cancelled".to_owned()),
            });
        }
        if target_doc_id.as_ref().is_some_and(|doc_id| {
            self.query_documents
                .get(self.active_query_document)
                .is_some_and(|doc| &doc.id == doc_id)
        }) {
            self.runtime_message = "Query cancelled".to_owned();
        }
    }

    fn on_sql_prediction_ready(
        &mut self,
        request_id: RequestId,
        document_id: String,
        document_version: u64,
        anchor: usize,
        replacement_range: (usize, usize),
        prediction_text: String,
    ) {
        let Some(doc) = self.query_documents.iter_mut().find(|doc| doc.id == document_id) else {
            return;
        };
        if doc.pending_prediction_request != Some(request_id) {
            return;
        }
        doc.pending_prediction_request = None;
        if doc.buffer.version() != document_version || doc.cursor.offset != anchor {
            doc.prediction_stale_responses_dropped = doc.prediction_stale_responses_dropped.saturating_add(1);
            return;
        }
        let (before, after) = doc.buffer.split_at(anchor);
        let prediction_text = if replacement_range.0 == replacement_range.1 {
            crate::editor::prediction::normalize_prediction_overlap(before, after, &prediction_text)
        } else {
            prediction_text
        };
        if crate::editor::prediction::is_prediction_acceptable(&prediction_text) {
            doc.prediction = Some(crate::editor::prediction::EditPrediction::with_range_and_version(
                anchor,
                replacement_range,
                prediction_text,
                Some(request_id),
                document_version,
            ));
            if let Some(prediction) = doc.prediction.clone() {
                if let Some(fingerprint) = doc.prediction_context_fingerprint {
                    doc.cache_prediction(fingerprint, prediction, std::time::Instant::now());
                }
            }
        } else {
            doc.prediction_rejected = doc.prediction_rejected.saturating_add(1);
        }
        if let Some(started_at) = doc.prediction_request_started_at.take() {
            doc.prediction_last_latency_ms = Some(started_at.elapsed().as_millis() as u64);
        }
    }

    fn on_sql_prediction_failed(
        &mut self,
        request_id: RequestId,
        document_id: String,
        _document_version: u64,
        _anchor: usize,
        _replacement_range: (usize, usize),
        _message: String,
    ) {
        if let Some(doc) = self
            .query_documents
            .iter_mut()
            .find(|doc| doc.id == document_id && doc.pending_prediction_request == Some(request_id))
        {
            doc.pending_prediction_request = None;
            doc.prediction_request_started_at = None;
        }
    }

    /// Routes a failure to whichever request slot is waiting on it.
    fn on_query_failed(
        &mut self,
        request_id: RequestId,
        message: String,
        position: Option<usize>,
        code: Option<String>,
    ) {
        if self.pending_connection_request == Some(request_id) {
            self.pending_connection_request = None;
            let conn_id = self
                .pending_connection_id
                .take()
                .or_else(|| self.active_connection_id.clone());
            if let Some(cid) = conn_id {
                self.failed_connection_ids.insert(cid.clone());
                self.connection_errors.insert(cid, message.clone());
            }
            if !self.connection_dialog_open {
                self.connected = false;
                self.schema_request = None;
                self.schema_error = None;
            }
            self.connection_error = message.clone();
            self.runtime_message = format!("Connection failed · {message}");
        } else if self.schema_request == Some(request_id) {
            self.schema_request = None;
            self.schema_error = Some(message.clone());
            self.runtime_message = format!("Schema introspection failed · {message}");
        } else if self.staged_apply_request == Some(request_id) {
            // Older runtimes can still report the generic failure event. Keep
            // the staged changes and surface it as an unmapped mutation.
            self.staged_apply_failed(usize::MAX, "UNKNOWN", &message, false);
        } else if self.table_mutation_request == Some(request_id) {
            self.table_mutation_request = None;
            self.data_editing_cell = None;
            self.data_edit_value.clear();
            self.data_edit_error = None;
            self.data_delete_confirmation = false;
            let formatted = format!("Row mutation failed · {message}");
            self.runtime_message = formatted.clone();
            self.show_toast_error(formatted);
        } else if self.table_info_request == Some(request_id) {
            self.table_info_request = None;
            self.table_info_error = Some(message.clone());
            self.runtime_message = format!("Table structure failed · {message}");
        } else if self.table_ddl_request == Some(request_id) {
            self.table_ddl_request = None;
            self.table_ddl_error = Some(message.clone());
            self.runtime_message = format!("Table DDL failed · {message}");
        } else if self.table_row_reload_request == Some(request_id) {
            self.table_row_reload_request = None;
            self.table_row_reload_identity = None;
            self.table_mutation_retry_after_reload = false;
            self.table_mutation_retry_target = None;
            self.runtime_message = format!("Could not reload row: {message}");
        } else if self.table_data_request == Some(request_id) {
            self.table_data_request = None;
            self.table_mutation_retry_after_reload = false;
            self.table_mutation_retry_target = None;
            self.table_data_error = Some(message.clone());
            let formatted = format!("Table data failed · {message}");
            self.runtime_message = formatted.clone();
            self.show_toast_error(formatted);
        } else if self.ddl_execution_request == Some(request_id) {
            self.ddl_execution_request = None;
            self.ddl_execute_confirmation = false;
            self.table_ddl_error = Some(message.clone());
            let formatted = format!("DDL execution failed · {message}");
            self.runtime_message = formatted.clone();
            self.show_toast_error(formatted);
        } else if let Some(document_id) = self.query_save_requests.remove(&request_id) {
            if self.pending_close_after_save.is_some_and(|index| {
                self.query_documents
                    .get(index)
                    .is_some_and(|document| document.id == document_id)
            }) {
                self.pending_close_after_save = None;
            }
            self.runtime_message = format!("Save failed · {message}");
        } else if self.query_document_requests.contains_key(&request_id) {
            let target_doc_id = self.query_document_requests.remove(&request_id);
            let mut history = None;
            if let Some(doc_id) = &target_doc_id {
                if let Some(doc) = self.query_documents.iter_mut().find(|d| &d.id == doc_id) {
                    let execution_range = doc.executing_range.take();
                    let execution_sql = doc.executing_sql.take();
                    let execution_version = doc.executing_version.take();
                    let (started_at, duration_ms) = doc.take_execution_timing(0);
                    history = Some((
                        execution_sql.clone().unwrap_or_else(|| doc.text().to_owned()),
                        doc.connection_id.clone(),
                        doc.schema.clone(),
                        started_at,
                        duration_ms,
                    ));
                    doc.execution_state = QueryExecutionState::Failed;
                    doc.execution_diagnostic = if execution_version == Some(doc.buffer.version()) {
                        execution_sql.as_deref().and_then(|sql| {
                            execution_range.and_then(|range| {
                                super::query_view::database_error_diagnostic(
                                    &message,
                                    sql,
                                    range,
                                    position,
                                    code.as_deref(),
                                )
                            })
                        })
                    } else {
                        None
                    };
                    doc.query_messages.push(format!("Query failed · {message}"));
                }
            }
            if let Some((sql, connection_id, schema, started_at, duration_ms)) = history {
                self.record_query_history(QueryHistoryRecord {
                    sql,
                    connection_id,
                    schema,
                    started_at,
                    status: UiQueryHistoryStatus::Failed,
                    duration_ms,
                    row_count: None,
                    affected_rows: None,
                    error_code: code,
                    error_summary: Some(message.clone()),
                });
            }
            let is_active_doc = self
                .query_documents
                .get(self.active_query_document)
                .is_some_and(|d| target_doc_id.as_ref() == Some(&d.id));

            if is_active_doc {
                self.runtime_message = format!("Query failed · {message}");
            }
        } else if let Some(doc_index) = self
            .query_documents
            .iter()
            .position(|doc| doc.explain_request == Some(request_id))
        {
            let doc_id = self.query_documents[doc_index].id.clone();
            self.set_query_output_tab(&doc_id, OutputTab::Messages);
            let message = format!("Explain failed · {message}");
            self.runtime_message = message;
            if let Some(doc) = self.query_documents.get_mut(doc_index) {
                doc.explain_request = None;
                doc.explain_plan = None;
                doc.query_messages.push(self.runtime_message.clone());
            }
        } else {
            self.runtime_message = format!("Operation failed · {message}");
        }
    }

    fn on_table_changes_failed(
        &mut self,
        request_id: RequestId,
        code: String,
        message: String,
        statement_index: usize,
        rolled_back: bool,
    ) {
        if self.staged_apply_request == Some(request_id) {
            self.staged_apply_failed(statement_index, &code, &message, rolled_back);
        }
    }

    pub(super) fn handle_shortcuts(&mut self, ctx: &egui::Context) {
        if self.palette_mode.is_some() {
            return;
        }
        if ctx.input(|input| {
            input.key_pressed(egui::Key::S) && Self::primary_modifier_pressed(input) && input.modifiers.shift
        }) {
            self.open_save_as_dialog();
            return;
        }
        if ctx.input(|input| input.key_pressed(egui::Key::S) && Self::primary_modifier_pressed(input)) {
            self.save_query_document_at(self.active_query_document);
            return;
        }
        let text_input_has_focus = ctx.wants_keyboard_input();
        if !text_input_has_focus
            && ctx.input(|i| i.key_pressed(egui::Key::P) && Self::primary_modifier_pressed(i) && i.modifiers.shift)
        {
            self.open_palette(PaletteMode::Commands);
            return;
        }
        if !text_input_has_focus
            && (ctx.input(|i| i.key_pressed(egui::Key::K) && Self::primary_modifier_pressed(i))
                || ctx.input(|i| i.key_pressed(egui::Key::P) && Self::primary_modifier_pressed(i)))
        {
            self.open_palette(PaletteMode::QuickOpen);
            return;
        }
        if !text_input_has_focus && ctx.input(|i| i.key_pressed(egui::Key::B) && Self::primary_modifier_pressed(i)) {
            self.sidebar_open = !self.sidebar_open;
        }
        if !text_input_has_focus && ctx.input(|i| i.key_pressed(egui::Key::F) && Self::primary_modifier_pressed(i)) {
            self.editor_search_open = true;
        }
        if ctx.input(|i| {
            i.key_pressed(egui::Key::F5)
                || (self.query_editor_focused
                    && !self.agent_open
                    && i.key_pressed(egui::Key::Enter)
                    && Self::primary_modifier_pressed(i))
        }) {
            self.dispatch_query();
        }
        if ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
            if let Some(request_id) = self.active_query_running_request() {
                if self.query_capabilities().allows(|c| c.query.cancel) {
                    self.cancel_query(request_id);
                } else {
                    self.runtime_message = "Query cancellation is not supported for this provider".to_owned();
                }
            } else if self.query_tools_open {
                self.query_tools_open = false;
            } else if self.editor_search_open {
                self.editor_search_open = false;
            } else {
                self.set_agent_open(false, ctx);
            }
        }
    }

    pub(super) fn cancel_query(&mut self, request_id: crate::RequestId) {
        self.dispatch_command(UiCommand::CancelQuery { request_id });
        self.runtime_message = "Cancelling query…".to_owned();
    }

    pub(super) fn dispatch_query(&mut self) {
        if self.active_query_running_request().is_some() {
            return;
        }
        let Some(connection_id) = self
            .active_query_connection_id()
            .map(String::from)
            .or_else(|| self.active_connection().map(|connection| connection.id.clone()))
        else {
            self.runtime_message = "Create or select a connection first".to_owned();
            return;
        };
        let (sql, execution_range) = self
            .query_documents
            .get(self.active_query_document)
            .map(|doc| doc.resolve_executable_range())
            .unwrap_or_else(|| {
                (
                    self.active_query_text().trim().to_owned(),
                    (0, self.active_query_text().len()),
                )
            });
        if sql.trim().is_empty() {
            self.runtime_message = "Query is empty".to_owned();
            return;
        }
        let version = self.active_query_buffer_version();
        if self.hold_destructive_run(&sql, execution_range, version, false) {
            return;
        }
        self.send_query_run(connection_id, sql, execution_range, version, false);
    }

    pub(super) fn dispatch_query_all(&mut self) {
        if self.active_query_running_request().is_some() {
            return;
        }
        let Some(connection_id) = self
            .active_query_connection_id()
            .map(String::from)
            .or_else(|| self.active_connection().map(|connection| connection.id.clone()))
        else {
            self.runtime_message = "Create or select a connection first".to_owned();
            return;
        };
        let (sql, execution_range) = self
            .query_documents
            .get(self.active_query_document)
            .map(|doc| {
                let text = doc.text().trim().to_owned();
                let leading = doc.text().len().saturating_sub(doc.text().trim_start().len());
                (text.clone(), (leading, leading + text.len()))
            })
            .unwrap_or_else(|| {
                (
                    self.active_query_text().trim().to_owned(),
                    (0, self.active_query_text().len()),
                )
            });
        if sql.is_empty() {
            self.runtime_message = "Query is empty".to_owned();
            return;
        }
        let version = self.active_query_buffer_version();
        if self.hold_destructive_run(&sql, execution_range, version, true) {
            return;
        }
        self.send_query_run(connection_id, sql, execution_range, version, true);
    }

    /// Buffer version of the active query document, so an execution stays bound to the
    /// text it was started from.
    fn active_query_buffer_version(&self) -> u64 {
        self.query_documents
            .get(self.active_query_document)
            .map(|doc| doc.buffer.version())
            .unwrap_or(0)
    }

    /// Hold a statement or script the classifier rates `Destructive` until the user
    /// confirms the exact text, and report whether the run was held.
    ///
    /// The query editor is the documented path for arbitrary SQL (including DDL), so
    /// nothing is refused here: the text is kept in `pending_destructive_run` and sent by
    /// `confirm_pending_destructive_run`. Reads, writes and plain DDL dispatch exactly as
    /// before, and the backend policy still runs on whatever is dispatched.
    fn hold_destructive_run(
        &mut self,
        sql: &str,
        execution_range: (usize, usize),
        version: u64,
        all_statements: bool,
    ) -> bool {
        if db_pro_core::domain::safety::classify_script_safety(sql)
            != Some(db_pro_core::domain::safety::StatementSafety::Destructive)
        {
            return false;
        }
        self.pending_destructive_run = Some(PendingDestructiveRun {
            sql: sql.to_owned(),
            execution_range,
            version,
            all_statements,
        });
        self.runtime_message =
            "Destructive statement held for confirmation — nothing was sent to the database".to_owned();
        true
    }

    /// Send the statement the user confirmed. The text and the buffer version are the ones
    /// the prompt displayed, so a confirmation can never execute something the user did
    /// not see.
    pub(super) fn confirm_pending_destructive_run(&mut self) {
        let Some(pending) = self.pending_destructive_run.take() else {
            return;
        };
        let Some(connection_id) = self
            .active_query_connection_id()
            .map(String::from)
            .or_else(|| self.active_connection().map(|connection| connection.id.clone()))
        else {
            self.runtime_message = "Create or select a connection first".to_owned();
            return;
        };
        self.send_query_run(
            connection_id,
            pending.sql,
            pending.execution_range,
            pending.version,
            pending.all_statements,
        );
    }

    /// Drop a held destructive statement without executing it.
    pub(super) fn cancel_pending_destructive_run(&mut self) {
        if self.pending_destructive_run.take().is_some() {
            self.runtime_message = "Destructive statement cancelled — nothing was sent to the database".to_owned();
        }
    }

    fn send_query_run(
        &mut self,
        connection_id: String,
        sql: String,
        execution_range: (usize, usize),
        version: u64,
        all_statements: bool,
    ) {
        let discovered = crate::query::discover_sql_parameters(&sql);
        if all_statements && !discovered.is_empty() {
            self.runtime_message =
                "Parameterized scripts are not supported yet — run a single statement with bindings".to_owned();
            return;
        }

        let style = if self.active_driver().eq_ignore_ascii_case("postgresql")
            || self.active_driver().eq_ignore_ascii_case("postgres")
        {
            crate::query::PlaceholderStyle::NumberedDollar
        } else {
            crate::query::PlaceholderStyle::QuestionMark
        };
        let values = self
            .query_documents
            .get(self.active_query_document)
            .map(|doc| doc.parameter_values.clone())
            .unwrap_or_default();
        let (sql, params) = if discovered.is_empty() {
            (sql, Vec::new())
        } else {
            match crate::query::prepare_bound_sql(&sql, &values, style) {
                Ok(prepared) => (prepared.sql, prepared.values),
                Err(missing) => {
                    self.runtime_message = format!("Fill parameter {missing} before running");
                    return;
                }
            }
        };

        if !self.query_history.iter().any(|query| query == &sql) {
            self.query_history.push(sql.clone());
            if self.query_history.len() > 20 {
                self.query_history.remove(0);
            }
        }
        let request_id = self.task_bridge.next_request_id();
        if let Some(doc) = self.query_documents.get_mut(self.active_query_document) {
            doc.execution_state = QueryExecutionState::Running(request_id);
            doc.execution_started_at = Some(Instant::now());
            doc.execution_started_wall_time = Some(chrono::Utc::now().to_rfc3339());
            doc.executing_range = Some(execution_range);
            doc.executing_sql = Some(sql.clone());
            doc.executing_version = Some(version);
            doc.last_executed_range = Some(execution_range);
            doc.execution_diagnostic = None;
            self.query_document_requests.insert(request_id, doc.id.clone());
        }
        self.runtime_message = if all_statements {
            "Sending full script to runtime…".to_owned()
        } else {
            "Sending query to runtime…".to_owned()
        };
        self.dispatch_command(if all_statements {
            UiCommand::RunQueryMulti {
                request_id,
                connection_id,
                sql,
            }
        } else {
            UiCommand::RunQuery {
                request_id,
                connection_id,
                sql,
                params,
            }
        });
    }
}

#[cfg(test)]
mod tests {
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
        }
    }

    #[test]
    fn row_reload_merges_server_values_by_original_identity() {
        let mut app = row_reload_app();
        app.on_table_data_loaded(RequestId(9), row_result("fresh server value"), 1);

        let result = app.table_data_result.expect("table result should remain visible");
        assert_eq!(result.rows[0][1], UiCell::Text("fresh server value".to_owned()));
        assert!(app.table_row_reload_request.is_none());
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

        assert_eq!(app.runtime_message, "Row was deleted");
        assert!(app.table_data_result.is_some());
    }
}
