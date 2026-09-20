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

#[cfg(test)]
mod tests {
    use super::super::agent_workflow_state::{AgentUiActivityStatus, AgentUiToolResult};
    use super::*;
    use db_pro_core::domain::agent::{AgentTool, AgentToolOutput};
    use db_pro_core::domain::agent_context::{AgentResultColumn, AgentResultSummary};

    #[test]
    fn projects_sampled_query_result_with_total_row_count() {
        let result = AgentUiToolResult {
            call_id: "query-1".to_owned(),
            tool: AgentTool::RunQuery,
            output: AgentToolOutput::QueryResult {
                statement_index: None,
                result_count: 1,
                summary: AgentResultSummary {
                    columns: vec![AgentResultColumn {
                        name: "id".to_owned(),
                        data_type: Some("integer".to_owned()),
                    }],
                    sample_rows: vec![vec!["1".to_owned()]],
                    row_count: Some(42),
                    affected_rows: None,
                    truncated: true,
                },
            },
            duration_ms: Some(12),
            status: AgentUiActivityStatus::Success,
        };

        let projected = query_result(&result).expect("query result should project");

        assert_eq!(projected.row_count, 42);
        assert_eq!(projected.duration_ms, 12);
        assert_eq!(projected.rows.len(), 1);
    }
}
