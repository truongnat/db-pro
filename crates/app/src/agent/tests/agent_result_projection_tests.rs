//! Tests for `../agent_result_projection.rs` (moved out of the source file).
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