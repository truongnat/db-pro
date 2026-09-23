//! Query-history reducers over the editor-owned state.

use super::events::QueryHistoryRecord;
use super::*;

const QUERY_HISTORY_RETENTION: usize = 500;

pub(super) fn record_query_history(query_editor: &mut QueryEditorState, record: QueryHistoryRecord) {
    query_editor.query_history_entries.push(UiQueryHistoryEntry {
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
    let excess = query_editor
        .query_history_entries
        .len()
        .saturating_sub(QUERY_HISTORY_RETENTION);
    if excess > 0 {
        query_editor.query_history_entries.drain(..excess);
    }
}
