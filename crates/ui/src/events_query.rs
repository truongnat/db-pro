//! Runtime event handlers for query execution, history, explain, and prediction.
use super::events::QueryHistoryRecord;
use super::*;
use crate::RequestId;

impl DbProApp {
    pub(super) fn on_query_queued(&mut self, request_id: RequestId) {
        self.feedback.runtime_message = format!("Query queued · request {}", request_id.0);
    }

    pub(super) fn on_query_completed(&mut self, request_id: RequestId, result: UiQueryResult) {
        let target_doc_id = self.query_session_state.document_requests.remove(&request_id);
        // A new result set replaces the rows behind the grid, so nothing the projection cache holds
        // may survive it.
        self.invalidate_grid_projection();
        let mut history = None;
        if let Some(doc_id) = &target_doc_id {
            if let Some(doc) = self.query_session_state.documents.iter_mut().find(|d| &d.id == doc_id) {
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
            .query_session_state
            .documents
            .get(self.query_session_state.active_document_index)
            .is_some_and(|d| target_doc_id.as_ref() == Some(&d.id));

        if is_active_doc {
            self.feedback.runtime_message = format!("Query completed · {} rows", result.row_count);
            self.table_data.grid_sort_column = None;
            self.table_data.grid_column_widths = vec![180.0; result.columns.len()];
            self.table_data.selected_cell = None;
            self.table_data.selected_row = None;
            self.table_data.selected_rows.clear();
            self.table_data.selection_anchor_row = None;
            self.table_data.selection_anchor_cell = None;
            self.feedback.copy_status.clear();
            if let Some(doc_id) = target_doc_id.as_deref() {
                self.set_query_output_tab(doc_id, OutputTab::Results);
            }
            self.workspace.bottom_panel_open = true;
            self.query_editor.query_output_dock_maximized = false;
        }
    }

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

    pub(super) fn on_query_saved(&mut self, request_id: RequestId, query: UiSavedQuerySummary) {
        let document_id = self.query_session_state.save_requests.remove(&request_id);
        let mut close_index = None;
        if let Some(document_id) = document_id {
            if let Some((index, doc)) = self
                .query_session_state
                .documents
                .iter_mut()
                .enumerate()
                .find(|(_, doc)| doc.id == document_id)
            {
                doc.saved_query_id = Some(query.id.clone());
                doc.mark_saved();
                if self.query_session_state.pending_close_after_save == Some(index) {
                    close_index = Some(index);
                }
            }
        }
        self.query_library.saved_queries.retain(|saved| saved.id != query.id);
        self.query_library.saved_queries.push(query);
        self.feedback.runtime_message = "Query saved".to_owned();
        if let Some(index) = close_index {
            self.query_session_state.pending_close_after_save = None;
            self.close_query_document(index);
        }
    }

    pub(super) fn record_query_history(&mut self, record: QueryHistoryRecord) {
        self.query_editor.query_history_entries.push(UiQueryHistoryEntry {
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
        let excess = self
            .query_editor
            .query_history_entries
            .len()
            .saturating_sub(QUERY_HISTORY_RETENTION);
        if excess > 0 {
            self.query_editor.query_history_entries.drain(..excess);
        }
    }

    pub(super) fn on_explain_completed(&mut self, request_id: RequestId, plan: String) {
        let Some(doc_index) = self
            .query_session_state
            .documents
            .iter()
            .position(|doc| doc.explain_request == Some(request_id))
        else {
            return;
        };
        let doc_id = self.query_session_state.documents[doc_index].id.clone();
        self.set_query_output_tab(&doc_id, OutputTab::Explain);
        self.workspace.bottom_panel_open = true;
        self.query_editor.query_output_dock_maximized = false;
        if let Some(doc) = self.query_session_state.documents.get_mut(doc_index) {
            doc.explain_request = None;
            doc.explain_plan = Some(plan);
            self.feedback.runtime_message = "Query plan ready".to_owned();
            doc.query_messages.push(self.feedback.runtime_message.clone());
        }
    }

    pub(super) fn on_query_cancelled(&mut self, request_id: RequestId) {
        let target_doc_id = self.query_session_state.document_requests.remove(&request_id);
        let mut history = None;
        if let Some(doc_id) = &target_doc_id {
            if let Some(doc) = self.query_session_state.documents.iter_mut().find(|d| &d.id == doc_id) {
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
            self.query_session_state
                .documents
                .get(self.query_session_state.active_document_index)
                .is_some_and(|doc| &doc.id == doc_id)
        }) {
            self.feedback.runtime_message = "Query cancelled".to_owned();
        }
    }

    pub(super) fn on_sql_prediction_ready(
        &mut self,
        request_id: RequestId,
        document_id: String,
        document_version: u64,
        anchor: usize,
        replacement_range: (usize, usize),
        prediction_text: String,
    ) {
        let Some(doc) = self
            .query_session_state
            .documents
            .iter_mut()
            .find(|doc| doc.id == document_id)
        else {
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

    pub(super) fn on_sql_prediction_failed(
        &mut self,
        request_id: RequestId,
        document_id: String,
        _document_version: u64,
        _anchor: usize,
        _replacement_range: (usize, usize),
        _message: String,
    ) {
        if let Some(doc) = self
            .query_session_state
            .documents
            .iter_mut()
            .find(|doc| doc.id == document_id && doc.pending_prediction_request == Some(request_id))
        {
            doc.pending_prediction_request = None;
            doc.prediction_request_started_at = None;
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
