//! Active query document / connection / result session helpers.
use super::*;

impl DbProApp {
    pub(crate) fn active_query_text(&self) -> &str {
        self.query_session_state
            .documents
            .get(self.query_session_state.active_document_index)
            .map(|doc| doc.text())
            .unwrap_or("")
    }

    pub(crate) fn set_active_query_text(&mut self, text: impl Into<String>) {
        self.cancel_prediction_for_document(self.query_session_state.active_document_index);
        if let Some(doc) = self
            .query_session_state
            .documents
            .get_mut(self.query_session_state.active_document_index)
        {
            doc.set_text(text);
        }
    }

    pub(crate) fn append_to_active_query(&mut self, text: &str) {
        self.cancel_prediction_for_document(self.query_session_state.active_document_index);
        if let Some(doc) = self
            .query_session_state
            .documents
            .get_mut(self.query_session_state.active_document_index)
        {
            let mut current = doc.text().to_owned();
            if !current.trim().is_empty() {
                current.push_str("\n\n");
            }
            current.push_str(text);
            doc.set_text(current);
        }
    }

    pub(crate) fn active_explain_plan(&self) -> Option<&str> {
        self.query_session_state
            .documents
            .get(self.query_session_state.active_document_index)
            .and_then(|d| d.explain_plan.as_deref())
    }

    pub(crate) fn active_explain_request(&self) -> Option<crate::RequestId> {
        self.query_session_state
            .documents
            .get(self.query_session_state.active_document_index)
            .and_then(|d| d.explain_request)
    }

    pub(crate) fn active_query_output_tab(&self) -> OutputTab {
        self.query_session_state
            .documents
            .get(self.query_session_state.active_document_index)
            .and_then(|doc| self.query_output_state.tabs_by_document.get(&doc.id).copied())
            .unwrap_or(OutputTab::Results)
    }

    pub(crate) fn set_active_query_output_tab(&mut self, tab: OutputTab) {
        self.query_output_state.active_tab = tab;
        if let Some(doc_id) = self
            .query_session_state
            .documents
            .get(self.query_session_state.active_document_index)
            .map(|doc| doc.id.clone())
        {
            self.query_output_state.tabs_by_document.insert(doc_id, tab);
        }
    }

    pub(crate) fn set_query_output_tab(&mut self, document_id: &str, tab: OutputTab) {
        self.query_output_state
            .tabs_by_document
            .insert(document_id.to_owned(), tab);
        if self
            .query_session_state
            .documents
            .get(self.query_session_state.active_document_index)
            .is_some_and(|doc| doc.id == document_id)
        {
            self.query_output_state.active_tab = tab;
        }
    }

    pub(crate) fn active_query_running_request(&self) -> Option<crate::RequestId> {
        self.query_session_state
            .documents
            .get(self.query_session_state.active_document_index)
            .and_then(|doc| match doc.execution_state {
                QueryExecutionState::Running(request_id) => Some(request_id),
                _ => None,
            })
    }

    pub(crate) fn switch_query_document(&mut self, index: usize) {
        if index >= self.query_session_state.documents.len() || index == self.query_session_state.active_document_index
        {
            return;
        }
        self.query_session_state.active_document_index = index;
        let doc = &self.query_session_state.documents[index];
        self.query_editor.query_cursor_line = doc.cursor.line + 1;
        self.query_editor.query_cursor_column = doc.cursor.col + 1;
        if !doc.selection.is_empty() {
            let (start, end) = doc.selection.normalized();
            self.query_session_state.selected_text = doc.buffer.slice(start, end).to_owned();
        } else {
            self.query_session_state.selected_text.clear();
        }
        self.runtime_message = format!("Opened {}", self.query_session_state.documents[index].title);
    }

    // Problems / diagnostics: `problems_view.rs`.

    pub(crate) fn take_schema_snapshot(&mut self) {
        let label = format!(
            "{} @ {}",
            self.active_connection_name(),
            chrono::Utc::now().format("%H:%M:%S")
        );
        self.database_operations.schema_snapshot = Some(schema_compare::UiSchemaSnapshot::from_summary(
            label,
            &self.schema_explorer.schema,
        ));
        self.runtime_message = "Schema snapshot captured".to_owned();
    }

    pub(crate) fn diff_against_schema_snapshot(&mut self) {
        let Some(snapshot) = self.database_operations.schema_snapshot.clone() else {
            self.runtime_message = "Take a schema snapshot before comparing".to_owned();
            return;
        };
        let current = schema_compare::UiSchemaSnapshot::from_summary("current", &self.schema_explorer.schema);
        self.database_operations.schema_diff = Some(schema_compare::diff_snapshots(&snapshot, &current));
        self.database_operations.migration_plan = None;
        self.database_operations.migration_preview_sql.clear();
        self.database_operations.migration_confirm_destructive = false;
        self.database_operations.migration_fingerprint_at_preview.clear();
        self.runtime_message = "Schema diff ready".to_owned();
    }

    pub(crate) fn plan_migration_from_schema_diff(&mut self) {
        use db_pro_core::application::MigrationPlanner;

        let Some(diff) = self.database_operations.schema_diff.clone() else {
            self.runtime_message = "Diff a schema snapshot before planning a migration".into();
            return;
        };
        let core_diff = schema_compare::to_core_schema_diff(&diff);
        let driver = self.active_driver().to_owned();
        let plan = MigrationPlanner::plan_from_schema_diff(&core_diff, &driver);
        self.database_operations.migration_preview_sql = MigrationPlanner::preview_sql(&plan, true);
        self.database_operations.migration_fingerprint_at_preview = plan.fingerprint.clone();
        self.database_operations.migration_confirm_destructive = false;
        self.database_operations.migration_plan = Some(plan);
        self.runtime_message = "Migration plan ready — review SQL before apply".into();
    }

    pub(crate) fn apply_migration_preview(&mut self) {
        use db_pro_core::application::MigrationPlanner;

        let Some(plan) = self.database_operations.migration_plan.clone() else {
            self.runtime_message = "Plan a migration before applying".into();
            return;
        };
        if !MigrationPlanner::verify_fingerprint(&plan, &self.database_operations.migration_fingerprint_at_preview) {
            self.runtime_message = "Migration fingerprint changed — re-plan before apply".into();
            return;
        }
        if plan.has_destructive && !self.database_operations.migration_confirm_destructive {
            self.runtime_message = "Destructive migration requires explicit confirmation checkbox".into();
            return;
        }
        let sql = if plan.has_destructive && self.database_operations.migration_confirm_destructive {
            MigrationPlanner::preview_sql(&plan, false)
        } else {
            MigrationPlanner::non_destructive_sql(&plan)
        };
        if sql.trim().is_empty() {
            self.runtime_message = "No supported SQL operations to apply".into();
            return;
        }
        if self.table_state.ddl_execution_request.is_some() {
            return;
        }
        let Some(connection_id) = self.connection_lifecycle.active_connection_id.clone() else {
            return;
        };
        let request_id = self.task_bridge.next_request_id();
        self.dispatch_command(UiCommand::ExecuteDdl {
            request_id,
            connection_id,
            sql,
        });
        self.table_state.ddl_execution_request = Some(request_id);
        self.runtime_message = "Applying migration plan…".into();
    }

    pub(crate) fn handle_transaction_action(&mut self, action: crate::components::TransactionAction) {
        match action {
            crate::components::TransactionAction::ToggleAutoCommit(value) => {
                if self.query_execution.query_in_transaction && value {
                    self.runtime_message = "Commit or rollback the open transaction before enabling auto-commit".into();
                    return;
                }
                self.query_execution.query_auto_commit = value;
                if value {
                    self.query_execution.query_in_transaction = false;
                    self.query_execution.query_txn_pending = 0;
                }
            }
            crate::components::TransactionAction::Begin => {
                self.query_execution.query_auto_commit = false;
                self.dispatch_transaction_sql("BEGIN");
                self.query_execution.query_in_transaction = true;
                self.query_execution.query_txn_pending = 0;
            }
            crate::components::TransactionAction::Commit => {
                self.dispatch_transaction_sql("COMMIT");
                self.query_execution.query_in_transaction = false;
                self.query_execution.query_txn_pending = 0;
            }
            crate::components::TransactionAction::Rollback => {
                self.dispatch_transaction_sql("ROLLBACK");
                self.query_execution.query_in_transaction = false;
                self.query_execution.query_txn_pending = 0;
            }
        }
    }

    pub(super) fn dispatch_transaction_sql(&mut self, sql: &str) {
        let Some(connection_id) = self.active_query_connection_id().map(str::to_owned) else {
            self.runtime_message = "Connect before using transaction controls".into();
            return;
        };
        // Reuse the normal run path so execution state / cancel / history stay consistent.
        let version = self.active_query_buffer_version();
        self.send_query_run(connection_id, sql.to_owned(), (0, sql.len()), version, false);
    }

    pub(crate) fn active_query_result(&self) -> Option<&UiQueryResult> {
        self.query_session_state
            .documents
            .get(self.query_session_state.active_document_index)
            .and_then(|doc| {
                doc.query_results
                    .get(doc.active_result_index)
                    .or(doc.query_result.as_ref())
            })
    }

    pub(crate) fn active_query_result_count(&self) -> usize {
        self.query_session_state
            .documents
            .get(self.query_session_state.active_document_index)
            .map_or(0, |doc| {
                doc.query_results
                    .len()
                    .max(if doc.query_result.is_some() { 1 } else { 0 })
            })
    }

    pub(crate) fn set_active_query_result(&mut self, index: usize) {
        if let Some(doc) = self
            .query_session_state
            .documents
            .get_mut(self.query_session_state.active_document_index)
        {
            if index < doc.query_results.len() && doc.active_result_index != index {
                doc.active_result_index = index;
                self.invalidate_grid_projection();
            }
        }
    }

    /// Advance the grid's projection epoch: the displayed rows are about to change.
    ///
    /// Called wherever the row data behind the grid is replaced or edited in place — loading query
    /// results, loading table data, reloading one row, switching the active result set. Missing a
    /// call does not corrupt data, but the grid would keep drawing the previous order and filter.
    pub(crate) fn invalidate_grid_projection(&mut self) {
        self.table_data.grid_projection_epoch = self.table_data.grid_projection_epoch.wrapping_add(1);
    }

    /// Drop the per-row identity cache and the projection built from those rows.
    ///
    /// The two are invalidated together on purpose: every site that changes row data needs both, and
    /// keeping them in one call is what makes "no site was forgotten" checkable by grep.
    pub(crate) fn invalidate_grid_row_caches(&mut self) {
        self.table_data.grid_row_identity_cache.clear();
        self.table_data.grid_row_identity_cache_ready = false;
        self.invalidate_grid_projection();
    }

    /// The projection key for the result currently being drawn.
    pub(super) fn grid_projection_key(&self, result: &UiQueryResult) -> GridProjectionKey {
        GridProjectionKey {
            epoch: self.table_data.grid_projection_epoch,
            filter: self.table_data.grid_filter.clone(),
            sort_column: self.table_data.grid_sort_column,
            sort_desc: self.table_data.grid_sort_desc,
            row_count: result.row_count,
            column_count: result.columns.len(),
        }
    }

    pub(crate) fn active_query_messages(&self) -> &[String] {
        self.query_session_state
            .documents
            .get(self.query_session_state.active_document_index)
            .map(|doc| doc.query_messages.as_slice())
            .unwrap_or(&[])
    }

    pub(crate) fn active_query_connection_id(&self) -> Option<&str> {
        self.query_session_state
            .documents
            .get(self.query_session_state.active_document_index)
            .and_then(|doc| doc.connection_id.as_deref())
            .or(self.connection_lifecycle.active_connection_id.as_deref())
    }

    pub(crate) fn active_query_connection(&self) -> Option<&UiConnectionSummary> {
        let conn_id = self.active_query_connection_id()?;
        self.connection_catalog.find(conn_id)
    }

    /// Capabilities for the connection the active query document is bound to.
    ///
    /// Resolved from the bound connection, then from the active connection. It
    /// deliberately does **not** go through `active_query_driver`, whose display fallback
    /// is the literal `"PostgreSQL"`: answering with PostgreSQL's set while no connection
    /// exists is the same silent-wrong-answer this lookup replaces with a named state.
    pub(crate) fn query_capabilities(&self) -> CapabilityLookup {
        match self.active_query_connection() {
            Some(connection) => CapabilityLookup::for_driver_label(&connection.driver),
            None => match self.active_connection() {
                Some(connection) => CapabilityLookup::for_driver_label(&connection.driver),
                None => CapabilityLookup::NoActiveConnection,
            },
        }
    }

    pub(crate) fn active_query_connection_name(&self) -> &str {
        self.active_query_connection()
            .map(|c| c.name.as_str())
            .unwrap_or(self.connection_name.as_str())
    }

    pub(crate) fn active_query_driver(&self) -> &str {
        self.active_query_connection()
            .map(|c| c.driver.as_str())
            .unwrap_or_else(|| self.active_driver())
    }

    pub(crate) fn active_query_schema(&self) -> &str {
        self.query_session_state
            .documents
            .get(self.query_session_state.active_document_index)
            .and_then(|doc| doc.schema.as_deref())
            .unwrap_or_else(|| self.active_schema())
    }

    pub(crate) fn set_document_connection(&mut self, doc_index: usize, connection_id: Option<String>) {
        self.cancel_prediction_for_document(doc_index);
        if let Some(doc) = self.query_session_state.documents.get_mut(doc_index) {
            doc.connection_id = connection_id;
            doc.completion.clear();
        }
    }

    pub(crate) fn set_document_schema(&mut self, doc_index: usize, schema: Option<String>) {
        self.cancel_prediction_for_document(doc_index);
        if let Some(doc) = self.query_session_state.documents.get_mut(doc_index) {
            doc.schema = schema;
            doc.completion.clear();
        }
    }

    pub(crate) fn cancel_prediction_for_document(&mut self, doc_index: usize) {
        let request_id = self
            .query_session_state
            .documents
            .get(doc_index)
            .and_then(|doc| doc.pending_prediction_request);
        if let Some(request_id) = request_id {
            self.dispatch_command(UiCommand::CancelSqlPrediction { request_id });
        }
        if let Some(doc) = self.query_session_state.documents.get_mut(doc_index) {
            if request_id.is_some() {
                doc.prediction_requests_cancelled = doc.prediction_requests_cancelled.saturating_add(1);
            }
            doc.invalidate_prediction();
        }
    }

    // Query documents: `query_documents.rs`.
    // Workspace actions: `workspace_actions.rs`.
}
