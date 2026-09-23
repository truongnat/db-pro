use super::*;
use crate::domain::agent::AgentSqlPatch;

fn workflow(mode: AgentMode, auto_run: bool) -> AgentWorkflow {
    AgentWorkflow::new(
        AgentSession::new("doc-a", Some("conn-a".to_owned()), Some("public".to_owned())),
        mode,
        auto_run,
    )
}

fn request(workflow: &AgentWorkflow, tool: AgentTool, input: AgentToolInput) -> AgentToolRequest {
    AgentToolRequest {
        session_id: workflow.session().id,
        run_id: workflow.session().active_run.as_ref().expect("active run").id,
        document_id: "doc-a".to_owned(),
        document_version: workflow
            .session()
            .active_run
            .as_ref()
            .expect("active run")
            .document_version,
        tool,
        input,
    }
}

#[test]
fn modes_enforce_tool_permissions() {
    assert!(!is_tool_allowed(AgentMode::Ask, AgentTool::PatchQuery));
    assert!(!is_tool_allowed(AgentMode::Ask, AgentTool::RunQuery));
    assert!(is_tool_allowed(AgentMode::Ask, AgentTool::ExplainQuery));
    assert!(is_tool_allowed(AgentMode::Ask, AgentTool::SuggestIndexes));
    assert!(is_tool_allowed(AgentMode::Ask, AgentTool::MonitoringRead));
    assert!(is_tool_allowed(AgentMode::Edit, AgentTool::PatchQuery));
    assert!(!is_tool_allowed(AgentMode::Edit, AgentTool::RunQuery));
    assert!(is_tool_allowed(AgentMode::Agent, AgentTool::RunQuery));
}

#[test]
fn second_run_is_rejected_without_replacing_active_run() {
    let mut workflow = workflow(AgentMode::Agent, true);
    let first_run = workflow.start_run(4).expect("first run starts");
    assert_eq!(workflow.start_run(5), Err(AgentToolError::RunAlreadyActive));
    assert_eq!(
        workflow.session().active_run.as_ref().map(|run| run.id),
        Some(first_run)
    );
    assert_eq!(workflow.session().state, AgentSessionState::Running);
}

#[test]
fn tool_request_after_run_finishes_does_not_create_a_fake_run_id() {
    let mut workflow = workflow(AgentMode::Ask, false);
    let run_id = workflow.start_run(4).expect("run starts");
    workflow.complete(run_id).expect("run completes");
    let request = AgentToolRequest {
        session_id: workflow.session().id,
        run_id,
        document_id: "doc-a".to_owned(),
        document_version: 4,
        tool: AgentTool::GetCurrentQuery,
        input: AgentToolInput::None,
    };
    assert_eq!(
        workflow.request_tool(request, 4, None),
        Err(AgentToolError::RunNotActive)
    );
}

#[test]
fn document_strict_tool_request_is_rejected_when_stale() {
    let mut workflow = workflow(AgentMode::Agent, true);
    workflow.start_run(8).expect("run starts");
    let mut tool_request = request(
        &workflow,
        AgentTool::RunQuery,
        AgentToolInput::Query {
            sql: "SELECT 1".to_owned(),
        },
    );
    tool_request.document_version = 9;
    assert_eq!(
        workflow.request_tool(tool_request, 8, None),
        Err(AgentToolError::StaleDocument { expected: 8, actual: 9 })
    );
}

#[test]
fn current_query_can_refresh_a_stale_document_context() {
    let mut workflow = workflow(AgentMode::Ask, false);
    workflow.start_run(8).expect("run starts");
    let mut tool_request = request(&workflow, AgentTool::GetCurrentQuery, AgentToolInput::None);
    tool_request.document_version = 9;
    assert!(matches!(
        workflow.request_tool(tool_request, 9, None),
        Ok(AgentToolDisposition::Execute(_))
    ));
}

#[test]
fn patch_enters_preview_and_confirmation_state() {
    let mut workflow = workflow(AgentMode::Edit, false);
    let run_id = workflow.start_run(3).expect("run starts");
    let patch = AgentSqlPatch {
        document_id: "doc-a".to_owned(),
        expected_version: 3,
        range: (0, 6),
        replacement: "SELECT".to_owned(),
    };
    let disposition = workflow
        .request_tool(
            request(&workflow, AgentTool::PatchQuery, AgentToolInput::Patch { patch }),
            3,
            Some("select 1"),
        )
        .expect("patch preview");
    assert!(matches!(
        &disposition,
        AgentToolDisposition::PatchPreview { confirmation, .. }
            if confirmation.kind == AgentConfirmationKind::ApplyPatch && confirmation.safety.is_none()
    ));
    assert_eq!(workflow.session().state, AgentSessionState::AwaitingConfirmation);
    assert!(matches!(
        workflow.confirm(run_id, 3),
        Ok(AgentConfirmationResult::Approved(_))
    ));
    assert_eq!(workflow.session().state, AgentSessionState::Running);
}

#[test]
fn mutation_requires_confirmation_and_rejection_keeps_run_alive() {
    let mut workflow = workflow(AgentMode::Agent, true);
    let run_id = workflow.start_run(1).expect("run starts");
    let disposition = workflow
        .request_tool(
            request(
                &workflow,
                AgentTool::RunQuery,
                AgentToolInput::Query {
                    sql: "UPDATE users SET name = 'x'".to_owned(),
                },
            ),
            1,
            None,
        )
        .expect("confirmation required");
    assert!(matches!(disposition, AgentToolDisposition::ConfirmationRequired(_)));
    assert_eq!(workflow.session().state, AgentSessionState::AwaitingConfirmation);
    assert_eq!(workflow.reject(run_id), Ok(AgentConfirmationResult::Rejected));
    assert_eq!(workflow.session().state, AgentSessionState::Running);
}

#[test]
fn cancellation_clears_confirmation_and_requires_matching_run() {
    let mut workflow = workflow(AgentMode::Agent, false);
    let run_id = workflow.start_run(1).expect("run starts");
    assert_eq!(workflow.cancel(AgentRunId::new()), Err(AgentToolError::RunMismatch));
    assert_eq!(workflow.cancel(run_id), Ok(()));
    assert_eq!(workflow.session().state, AgentSessionState::Cancelled);
    assert!(workflow.pending_confirmation().is_none());
}

#[test]
fn mixed_batch_never_auto_runs_on_the_read_only_path() {
    let mut workflow = workflow(AgentMode::Agent, true);
    workflow.start_run(1).expect("run starts");
    let disposition = workflow
        .request_tool(
            request(
                &workflow,
                AgentTool::RunQuery,
                AgentToolInput::Query {
                    sql: "SELECT 1; DROP TABLE t;".to_owned(),
                },
            ),
            1,
            None,
        )
        .expect("disposition");
    assert!(
        matches!(
            &disposition,
            AgentToolDisposition::ConfirmationRequired(confirmation)
                if confirmation.safety == Some(AgentSqlSafety::Destructive)
        ),
        "a batch that ends in DROP must not auto-run: {disposition:?}"
    );
    assert_eq!(workflow.session().state, AgentSessionState::AwaitingConfirmation);
}

#[test]
fn read_only_batch_still_auto_runs_when_enabled() {
    let mut workflow = workflow(AgentMode::Agent, true);
    workflow.start_run(1).expect("run starts");
    let disposition = workflow
        .request_tool(
            request(
                &workflow,
                AgentTool::RunQuery,
                AgentToolInput::Query {
                    sql: "SELECT 1; SELECT 2;".to_owned(),
                },
            ),
            1,
            None,
        )
        .expect("disposition");
    assert!(
        matches!(disposition, AgentToolDisposition::Execute(_)),
        "a read-only batch stays auto-runnable: {disposition:?}"
    );
}
