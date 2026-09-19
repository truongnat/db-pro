//! Runtime event handlers for query execution, history, explain, and prediction.
use super::events::QueryHistoryRecord;
use super::*;
use crate::RequestId;

impl DbProApp {
    pub(super) fn on_query_multi_completed(&mut self, request_id: RequestId, output: UiQueryExecutionOutput) {
        let target_doc_id = self.query_session_state.document_requests.remove(&request_id);
        let Some(doc_id) = target_doc_id else {
            return;
        };
        // Same as the single-statement path: the result sets behind the grid are being replaced.
        self.invalidate_grid_projection();
        let mut history = None;
        if let Some(doc) = self
            .query_session_state
            .documents
            .iter_mut()
            .find(|doc| doc.id == doc_id)
        {
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
            .query_session_state
            .documents
            .get(self.query_session_state.active_document_index)
            .is_some_and(|document| document.id == doc_id);
        if is_active_doc {
            self.feedback.runtime_message = format!(
                "Script completed · {} result{} · {} ms",
                self.query_session_state
                    .documents
                    .get(self.query_session_state.active_document_index)
                    .map_or(0, |document| document.query_results.len()),
                if self.active_query_result_count() == 1 { "" } else { "s" },
                output.total_duration_ms,
            );
            self.set_query_output_tab(&doc_id, OutputTab::Results);
            self.table_data.grid_sort_column = None;
            self.table_data.selected_cell = None;
            self.table_data.selected_row = None;
            self.table_data.selected_rows.clear();
            self.workspace.bottom_panel_open = true;
            self.query_editor.query_output_dock_maximized = false;
        }
    }

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
                .is_some_and(|d| target_doc_id.as_ref() == Some(&d.id));

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
