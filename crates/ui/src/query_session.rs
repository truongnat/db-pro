//! Active query document / connection / result session helpers.
use super::*;

impl DbProApp {
    pub(crate) fn active_query_text(&self) -> &str {
        self.query_session_state.active_text()
    }

    pub(crate) fn set_active_query_text(&mut self, text: impl Into<String>) {
        self.cancel_prediction_for_document(self.query_session_state.active_document_index);
        self.query_session_state.set_active_text(text);
    }

    pub(crate) fn append_to_active_query(&mut self, text: &str) {
        self.cancel_prediction_for_document(self.query_session_state.active_document_index);
        self.query_session_state.append_active_text(text);
    }

    pub(crate) fn active_query_output_tab(&self) -> OutputTab {
        self.query_session_state
            .documents
            .get(self.query_session_state.active_document_index)
            .map(|doc| self.query_output_state.tab_for_document(&doc.id))
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
            self.query_output_state.set_active_for_document(&doc_id, tab);
        }
    }

    pub(crate) fn set_query_output_tab(&mut self, document_id: &str, tab: OutputTab) {
        self.query_output_state.set_for_document(document_id, tab);
        if self
            .query_session_state
            .documents
            .get(self.query_session_state.active_document_index)
            .is_some_and(|doc| doc.id == document_id)
        {
            self.query_output_state.active_tab = tab;
        }
    }

    pub(crate) fn switch_query_document(&mut self, index: usize) {
        if index == self.query_session_state.active_document_index || !self.query_session_state.select_document(index) {
            return;
        }
        let Some(doc) = self.query_session_state.active_document() else {
            return;
        };
        self.query_editor.query_cursor_line = doc.cursor.line + 1;
        self.query_editor.query_cursor_column = doc.cursor.col + 1;
        if !doc.selection.is_empty() {
            let (start, end) = doc.selection.normalized();
            self.query_session_state.selected_text = doc.buffer.slice(start, end).to_owned();
        } else {
            self.query_session_state.selected_text.clear();
        }
        self.feedback.runtime_message = format!("Opened {}", self.query_session_state.documents[index].title);
    }

    // Problems / diagnostics: `problems_view.rs`.

    pub(crate) fn apply_migration_preview(&mut self) {
        use db_pro_core::application::MigrationPlanner;

        let Some(plan) = self.schema_compare.migration_plan.clone() else {
            self.feedback.runtime_message = "Plan a migration before applying".into();
            return;
        };
        if !MigrationPlanner::verify_fingerprint(&plan, &self.schema_compare.migration_fingerprint_at_preview) {
            self.feedback.runtime_message = "Migration fingerprint changed — re-plan before apply".into();
            return;
        }
        if plan.has_destructive && !self.schema_compare.migration_confirm_destructive {
            self.feedback.runtime_message = "Destructive migration requires explicit confirmation checkbox".into();
            return;
        }
        let sql = if plan.has_destructive && self.schema_compare.migration_confirm_destructive {
            MigrationPlanner::preview_sql(&plan, false)
        } else {
            MigrationPlanner::non_destructive_sql(&plan)
        };
        if sql.trim().is_empty() {
            self.feedback.runtime_message = "No supported SQL operations to apply".into();
            return;
        }
        if self.table_state.ddl_execution_request.is_some() {
            return;
        }
        let Some(connection_id) = self.connection.lifecycle.active_connection_id().map(str::to_owned) else {
            return;
        };
        let request_id = self.task_bridge.next_request_id();
        self.dispatch_command(UiCommand::ExecuteDdl {
            request_id,
            connection_id,
            sql,
        });
        self.table_state.ddl_execution_request = Some(request_id);
        self.feedback.runtime_message = "Applying migration plan…".into();
    }

    pub(crate) fn handle_transaction_action(&mut self, action: crate::components::TransactionAction) {
        if let Some(sql) = self
            .query_execution
            .apply_transaction_action(action, &mut self.feedback)
        {
            self.dispatch_transaction_sql(sql);
        }
    }

    pub(super) fn dispatch_transaction_sql(&mut self, sql: &str) {
        let Some(connection_id) = self.active_query_connection_id().map(str::to_owned) else {
            self.feedback.runtime_message = "Connect before using transaction controls".into();
            return;
        };
        // Reuse the normal run path so execution state / cancel / history stay consistent.
        let version = self.query_session_state.active_buffer_version();
        self.send_query_run(connection_id, sql.to_owned(), (0, sql.len()), version, false);
    }

    pub(crate) fn active_query_result(&self) -> Option<&UiQueryResult> {
        self.query_session_state.active_result()
    }

    pub(crate) fn set_active_query_result(&mut self, index: usize) {
        if self.query_session_state.set_active_result(index) {
            self.table_data.invalidate_grid_projection();
        }
    }

    pub(crate) fn active_query_messages(&self) -> &[String] {
        self.query_session_state.active_messages()
    }

    pub(crate) fn active_query_connection_id(&self) -> Option<&str> {
        self.query_session_state
            .active_connection_id()
            .or(self.connection.lifecycle.active_connection_id())
    }

    pub(crate) fn active_query_connection(&self) -> Option<&UiConnectionSummary> {
        let conn_id = self.active_query_connection_id()?;
        self.connection.catalog.find(conn_id)
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
            .unwrap_or(self.connection.lifecycle.fallback_name())
    }

    pub(crate) fn active_query_driver(&self) -> &str {
        self.active_query_connection()
            .map(|c| c.driver.as_str())
            .unwrap_or_else(|| self.active_driver())
    }

    pub(crate) fn active_query_schema(&self) -> &str {
        self.query_session_state
            .active_schema()
            .unwrap_or_else(|| self.active_schema())
    }

    pub(crate) fn set_document_connection(&mut self, doc_index: usize, connection_id: Option<String>) {
        self.cancel_prediction_for_document(doc_index);
        self.query_session_state
            .set_document_connection(doc_index, connection_id);
    }

    pub(crate) fn set_document_schema(&mut self, doc_index: usize, schema: Option<String>) {
        self.cancel_prediction_for_document(doc_index);
        self.query_session_state.set_document_schema(doc_index, schema);
    }

    pub(crate) fn cancel_prediction_for_document(&mut self, doc_index: usize) {
        let request_id = self.query_session_state.invalidate_prediction(doc_index);
        if let Some(request_id) = request_id {
            self.dispatch_command(UiCommand::CancelSqlPrediction { request_id });
        }
    }

    // Query documents: `query_documents.rs`.
    // Workspace actions: `workspace_actions.rs`.
}
