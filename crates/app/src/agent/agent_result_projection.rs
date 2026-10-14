//! Projection from Agent tool results to the query result surface.

use super::*;
use crate::runtime::UiColumn;

pub(super) fn query_result(tool_result: &super::agent_workflow_state::AgentUiToolResult) -> Option<UiQueryResult> {
    let db_pro_core::domain::agent::AgentToolOutput::QueryResult { summary, .. } = &tool_result.output else {
        return None;
    };
    let columns = summary
        .columns
        .iter()
        .map(|column| UiColumn {
            name: column.name.clone(),
            data_type: column.data_type.clone().unwrap_or_else(|| "text".to_owned()),
            nullable: true,
        })
        .collect();
    let rows = summary
        .sample_rows
        .iter()
        .map(|row| row.iter().map(|cell| UiCell::Text(cell.clone())).collect())
        .collect();
    let sample_len = summary.sample_rows.len();
    Some(UiQueryResult {
        columns,
        rows,
        row_count: summary.row_count.unwrap_or(sample_len as u64),
        duration_ms: tool_result.duration_ms.unwrap_or(0),
    })
}

pub(super) struct AgentResultWorkspaceContext<'a> {
    session: &'a mut super::QuerySessionState,
    output: &'a mut super::QueryOutputState,
    table_data: &'a mut super::TableDataState,
    feedback: &'a mut super::FeedbackState,
}

impl<'a> AgentResultWorkspaceContext<'a> {
    pub(super) fn new(
        session: &'a mut super::QuerySessionState,
        output: &'a mut super::QueryOutputState,
        table_data: &'a mut super::TableDataState,
        feedback: &'a mut super::FeedbackState,
    ) -> Self {
        Self {
            session,
            output,
            table_data,
            feedback,
        }
    }

    pub(super) fn open(
        &mut self,
        active_document_index: usize,
        tool_result: &super::agent_workflow_state::AgentUiToolResult,
    ) {
        let Some(document) = self.session.documents.get_mut(active_document_index) else {
            return;
        };
        let Some(ui_result) = query_result(tool_result) else {
            return;
        };

        let sample_len = ui_result.rows.len();
        let total_rows = ui_result.row_count;
        document.query_results = vec![ui_result];
        document.query_result = None;
        document.active_result_index = 0;
        let document_id = document.id.clone();
        let active_document_id = self
            .session
            .active_document()
            .map(|active| active.id.as_str());
        self.output.set_for_document_and_activate_if_active(
            &document_id,
            active_document_id,
            super::OutputTab::Results,
        );
        self.table_data.invalidate_grid_projection();
        self.feedback.runtime_message = if total_rows > sample_len as u64 {
            format!("Showing {sample_len} sampled rows of {total_rows} total rows.")
        } else {
            format!("Opened Agent query result ({total_rows} rows)")
        };
    }
}

#[cfg(test)]
#[path = "tests/agent_result_projection_tests.rs"]
mod tests;

