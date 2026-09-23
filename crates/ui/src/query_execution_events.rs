//! Query explain/cancellation reducers over explicit query feature state.

use super::events::QueryHistoryRecord;
use super::*;
use crate::RequestId;

pub(super) fn on_explain_completed(
    query_session: &mut QuerySessionState,
    query_output: &mut QueryOutputState,
    workspace: &mut WorkspaceShellState,
    query_editor: &mut QueryEditorState,
    feedback: &mut FeedbackState,
    request_id: RequestId,
    plan: String,
) {
    let Some(doc_index) = query_session
        .documents
        .iter()
        .position(|doc| doc.explain_request == Some(request_id))
    else {
        return;
    };
    let doc_id = query_session.documents[doc_index].id.clone();
    query_output.tabs_by_document.insert(doc_id.clone(), OutputTab::Explain);
    if query_session
        .documents
        .get(query_session.active_document_index)
        .is_some_and(|doc| doc.id == doc_id)
    {
        query_output.active_tab = OutputTab::Explain;
    }
    workspace.bottom_panel_open = true;
    query_editor.query_output_dock_maximized = false;
    if let Some(doc) = query_session.documents.get_mut(doc_index) {
        doc.explain_request = None;
        doc.explain_plan = Some(plan);
        feedback.set_runtime_message("Query plan ready");
        doc.query_messages.push(feedback.runtime_message.clone());
    }
}

pub(super) fn on_query_cancelled(
    query_session: &mut QuerySessionState,
    query_editor: &mut QueryEditorState,
    feedback: &mut FeedbackState,
    request_id: RequestId,
) {
    let target_doc_id = query_session.document_requests.remove(&request_id);
    let mut history = None;
    if let Some(doc_id) = &target_doc_id {
        if let Some(doc) = query_session.documents.iter_mut().find(|doc| &doc.id == doc_id) {
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
        super::query_history_events::record_query_history(
            query_editor,
            QueryHistoryRecord {
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
            },
        );
    }
    if target_doc_id.as_ref().is_some_and(|doc_id| {
        query_session
            .documents
            .get(query_session.active_document_index)
            .is_some_and(|doc| &doc.id == doc_id)
    }) {
        feedback.set_runtime_message("Query cancelled");
    }
}
