//! Single-statement query result reducer over explicit query feature state.

use super::events::QueryHistoryRecord;
use super::*;
use crate::RequestId;

pub(super) struct QueryResultContext<'a> {
    pub(super) query_session: &'a mut QuerySessionState,
    pub(super) query_editor: &'a mut QueryEditorState,
    pub(super) query_output: &'a mut QueryOutputState,
    pub(super) table_data: &'a mut TableDataState,
    pub(super) workspace: &'a mut WorkspaceShellState,
    pub(super) feedback: &'a mut FeedbackState,
}

pub(super) fn on_query_completed(context: &mut QueryResultContext<'_>, request_id: RequestId, result: UiQueryResult) {
    let QueryResultContext {
        query_session,
        query_editor,
        query_output,
        table_data,
        workspace,
        feedback,
    } = context;
    let target_doc_id = query_session.document_requests.remove(&request_id);
    // A new result set replaces the rows behind the grid, so nothing the projection cache holds
    // may survive it.
    table_data.grid_projection_epoch = table_data.grid_projection_epoch.wrapping_add(1);
    let mut history = None;
    if let Some(doc_id) = &target_doc_id {
        if let Some(doc) = query_session.documents.iter_mut().find(|doc| &doc.id == doc_id) {
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
        super::query_history_events::record_query_history(
            query_editor,
            QueryHistoryRecord {
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
            },
        );
    }
    let is_active_doc = query_session
        .documents
        .get(query_session.active_document_index)
        .is_some_and(|doc| target_doc_id.as_ref() == Some(&doc.id));

    if is_active_doc {
        feedback.set_runtime_message(format!("Query completed · {} rows", result.row_count));
        table_data.grid_sort_column = None;
        table_data.grid_column_widths = vec![180.0; result.columns.len()];
        table_data.selected_cell = None;
        table_data.selected_row = None;
        table_data.selected_rows.clear();
        table_data.selection_anchor_row = None;
        table_data.selection_anchor_cell = None;
        feedback.copy_status.clear();
        if let Some(doc_id) = target_doc_id {
            query_output.tabs_by_document.insert(doc_id.clone(), OutputTab::Results);
            if query_session
                .documents
                .get(query_session.active_document_index)
                .is_some_and(|doc| doc.id == doc_id)
            {
                query_output.active_tab = OutputTab::Results;
            }
        }
        workspace.bottom_panel_open = true;
        query_editor.query_output_dock_maximized = false;
    }
}
