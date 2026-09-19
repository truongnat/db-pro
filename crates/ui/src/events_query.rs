//! Runtime query-failure routing.

use super::events::QueryHistoryRecord;
use super::*;
use crate::RequestId;

impl DbProApp {
    /// Routes a failure to whichever request slot is waiting on it.
    pub(super) fn on_query_failed(
        &mut self,
        request_id: RequestId,
        message: String,
        position: Option<usize>,
        code: Option<String>,
    ) {
        if self.handle_agent_request_failure(request_id, &message)
            || self.handle_connection_request_failure(request_id, &message)
            || self.handle_schema_request_failure(request_id, &message)
            || self.handle_table_request_failure(request_id, &message)
        {
            return;
        }

        if let Some(document_id) = self.query_session_state.save_requests.remove(&request_id) {
            if self.query_session_state.pending_close_after_save.is_some_and(|index| {
                self.query_session_state
                    .documents
                    .get(index)
                    .is_some_and(|document| document.id == document_id)
            }) {
                self.query_session_state.pending_close_after_save = None;
            }
            self.feedback.runtime_message = format!("Save failed · {message}");
        } else if self.query_session_state.document_requests.contains_key(&request_id) {
            let target_doc_id = self.query_session_state.document_requests.remove(&request_id);
            let mut history = None;
            if let Some(doc_id) = &target_doc_id {
                if let Some(doc) = self.query_session_state.documents.iter_mut().find(|d| &d.id == doc_id) {
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
                .query_session_state
                .documents
                .get(self.query_session_state.active_document_index)
                .is_some_and(|doc| target_doc_id.as_ref() == Some(&doc.id));

            if is_active_doc {
                self.feedback.runtime_message = format!("Query failed · {message}");
                self.workspace.bottom_panel_open = true;
                self.query_editor.query_output_dock_maximized = false;
                if let Some(doc_id) = target_doc_id.as_deref() {
                    self.set_query_output_tab(doc_id, OutputTab::Messages);
                }
            }
        } else if let Some(doc_index) = self
            .query_session_state
            .documents
            .iter()
            .position(|doc| doc.explain_request == Some(request_id))
        {
            let doc_id = self.query_session_state.documents[doc_index].id.clone();
            self.set_query_output_tab(&doc_id, OutputTab::Messages);
            self.workspace.bottom_panel_open = true;
            self.query_editor.query_output_dock_maximized = false;
            let message = format!("Explain failed · {message}");
            self.feedback.runtime_message = message;
            if let Some(doc) = self.query_session_state.documents.get_mut(doc_index) {
                doc.explain_request = None;
                doc.explain_plan = None;
                doc.query_messages.push(self.feedback.runtime_message.clone());
            }
        } else {
            self.feedback.runtime_message = format!("Operation failed · {message}");
        }
    }
}
