//! Tests for `../executor.rs` (moved out of the source file).
use super::*;
use db_pro_core::domain::agent::{AgentDocumentSnapshot, AgentRun, AgentRunId, AgentSession, AgentSessionId};
use db_pro_core::domain::query::{CellValue, ColumnMeta, QueryResult, Row};

fn context() -> (AgentToolRequest, AgentExecutionContext) {
    let session_id = AgentSessionId::new();
    let run_id = AgentRunId::new();
    let session = AgentSession {
        id: session_id,
        document_id: "doc-a".to_owned(),
        connection_id: None,
        schema: Some("public".to_owned()),
        messages: Vec::new(),
        state: db_pro_core::domain::agent::AgentSessionState::Running,
        active_run: Some(AgentRun {
            id: run_id,
            document_version: 7,
        }),
    };
    let document = AgentDocumentSnapshot {
        document_id: "doc-a".to_owned(),
        document_version: 7,
        sql: "SELECT café".to_owned(),
        cursor_offset: 12,
        selection: None,
        current_statement: Some("SELECT café".to_owned()),
    };
    let request = AgentToolRequest {
        session_id,
        run_id,
        document_id: "doc-a".to_owned(),
        document_version: 7,
        tool: AgentTool::GetCurrentQuery,
        input: AgentToolInput::None,
    };
    (request, AgentExecutionContext::new(session, document, AgentMode::Ask))
}

#[test]
fn request_validation_rejects_late_run() {
    let (mut request, mut context) = context();
    context.session.active_run = None;
    assert_eq!(validate_request(&request, &context), Err(AgentToolError::RunNotActive));
    request.document_version = 8;
    assert_eq!(validate_request(&request, &context), Err(AgentToolError::RunNotActive));
}

#[test]
fn truncate_chars_preserves_utf8() {
    assert_eq!(truncate_chars("café 🐘", 5), "café ");
}

#[test]
fn current_query_uses_document_snapshot() {
    let (request, context) = context();
    let output = current_query(&request, &context).expect("snapshot");
    assert!(matches!(
        output,
        AgentToolOutput::CurrentQuery { cursor_offset: 12, .. }
    ));
}

#[test]
fn multi_result_summary_uses_latest_result_set_not_trailing_command() {
    let command = QueryResult {
        columns: Vec::new(),
        rows: Vec::new(),
        row_count: 3,
        duration_ms: 1,
    };
    let result_set = QueryResult {
        columns: vec![ColumnMeta {
            name: "name".to_owned(),
            data_type: "text".to_owned(),
            nullable: false,
        }],
        rows: vec![Row(vec![CellValue::Text("users".to_owned())])],
        row_count: 1,
        duration_ms: 1,
    };
    let output = map_multi_query_output(MultiQueryResult {
        results: vec![command, result_set],
        result_kinds: vec![StatementResultKind::Command, StatementResultKind::ResultSet],
        total_duration_ms: 2,
        error: None,
    })
    .expect("summary");
    assert!(matches!(
        output,
        AgentToolOutput::QueryResult {
            statement_index: Some(1),
            result_count: 1,
            summary: AgentResultSummary { row_count: Some(1), .. }
        }
    ));
}

#[test]
fn read_only_execution_respects_explicit_auto_run_setting() {
    let (_, mut context) = context();
    context.mode = AgentMode::Agent;
    assert_eq!(
        ensure_execution_confirmation(&context, "SELECT 1"),
        Err(AgentToolError::ConfirmationRequired {
            kind: AgentConfirmationKind::RunReadOnly
        })
    );
    context.allow_read_only_auto_run = true;
    assert_eq!(ensure_execution_confirmation(&context, "SELECT 1"), Ok(()));
}

#[test]
fn mixed_batch_is_never_auto_run_before_execute_multi() {
    let (_, mut context) = context();
    context.mode = AgentMode::Agent;
    context.allow_read_only_auto_run = true;
    assert_eq!(
        ensure_execution_confirmation(&context, "SELECT 1; DROP TABLE t;"),
        Err(AgentToolError::ConfirmationRequired {
            kind: AgentConfirmationKind::RunDestructive
        })
    );
    assert_eq!(ensure_execution_confirmation(&context, "SELECT 1; SELECT 2"), Ok(()));
    // An explicit confirmation still executes the batch; the guard decides
    // whether consent is needed, not whether the batch is legal.
    context.confirmed = true;
    assert_eq!(
        ensure_execution_confirmation(&context, "SELECT 1; DROP TABLE t;"),
        Ok(())
    );
}
