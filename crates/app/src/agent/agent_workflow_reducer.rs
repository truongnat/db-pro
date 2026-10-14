//! Pure reducers for the Agent workflow session model.

use super::*;

pub(super) fn apply_event(
    session: &mut super::agent_workflow_state::AgentUiSession,
    event: db_pro_core::domain::agent_workflow::AgentWorkflowEvent,
    run_id: db_pro_core::domain::agent::AgentRunId,
) {
    use super::agent_workflow_state::{
        AgentAuditEntry, AgentUiActivity, AgentUiActivityStatus, AgentUiConfirmation, AgentUiToolResult,
    };
    use db_pro_core::domain::agent_workflow::AgentWorkflowEvent;

    match event {
        AgentWorkflowEvent::TextDelta { delta, .. } => {
            session.state = db_pro_core::domain::agent::AgentSessionState::Running;
            session.streaming_text.push_str(&delta);
        }
        AgentWorkflowEvent::ToolRequested { call, .. } => {
            flush_stream(session);
            session.state = db_pro_core::domain::agent::AgentSessionState::Running;
            session.activities.push(AgentUiActivity {
                call_id: Some(call.call_id),
                tool: Some(call.tool),
                label: tool_label(call.tool),
                status: AgentUiActivityStatus::Running,
                duration_ms: None,
            });
        }
        AgentWorkflowEvent::ToolCompleted { call_id, result, .. } => {
            set_activity_status(session, &call_id, AgentUiActivityStatus::Success);
            session.tool_results.insert(
                call_id.clone(),
                AgentUiToolResult {
                    call_id: call_id.clone(),
                    tool: result.tool,
                    output: result.output.clone(),
                    duration_ms: None,
                    status: AgentUiActivityStatus::Success,
                },
            );
            session.audit_trail.push(AgentAuditEntry {
                run_id,
                tool: result.tool,
                duration_ms: None,
                safety: None,
                confirmed: false,
            });
            if session.audit_trail.len() > 30 {
                session.audit_trail.remove(0);
            }
            session.state = db_pro_core::domain::agent::AgentSessionState::Running;
        }
        AgentWorkflowEvent::ToolFailed {
            call_id, tool, error, ..
        } => {
            set_activity_status(session, &call_id, AgentUiActivityStatus::Failed);
            session.messages.push(AgentMessage {
                role: AgentRole::Assistant,
                content: format!("{} failed: {}", tool_label(tool), error.format_user_error()),
                sql: None,
                requires_confirmation: false,
            });
            session.state = db_pro_core::domain::agent::AgentSessionState::Running;
        }
        AgentWorkflowEvent::ConfirmationRequired {
            call_id, kind, preview, ..
        } => {
            set_activity_status(session, &call_id, AgentUiActivityStatus::AwaitingConfirmation);
            let document_id = session
                .session
                .as_ref()
                .map(|value| value.document_id.clone())
                .unwrap_or_default();
            session.pending_confirmation = Some(AgentUiConfirmation {
                run_id,
                call_id,
                kind,
                preview,
                document_id,
            });
            session.state = db_pro_core::domain::agent::AgentSessionState::AwaitingConfirmation;
        }
        AgentWorkflowEvent::Completed { .. } => {
            finish_session(session, db_pro_core::domain::agent::AgentSessionState::Completed);
        }
        AgentWorkflowEvent::Failed { message, .. } => {
            flush_stream(session);
            session.messages.push(AgentMessage {
                role: AgentRole::Assistant,
                content: message,
                sql: None,
                requires_confirmation: false,
            });
            finish_session(session, db_pro_core::domain::agent::AgentSessionState::Failed);
        }
        AgentWorkflowEvent::Cancelled { .. } => {
            for activity in &mut session.activities {
                if matches!(
                    activity.status,
                    AgentUiActivityStatus::Running | AgentUiActivityStatus::AwaitingConfirmation
                ) {
                    activity.status = AgentUiActivityStatus::Cancelled;
                }
            }
            finish_session(session, db_pro_core::domain::agent::AgentSessionState::Cancelled);
        }
    }
}

fn flush_stream(session: &mut super::agent_workflow_state::AgentUiSession) {
    if session.streaming_text.is_empty() {
        return;
    }
    session.messages.push(AgentMessage {
        role: AgentRole::Assistant,
        content: std::mem::take(&mut session.streaming_text),
        sql: None,
        requires_confirmation: false,
    });
}

fn set_activity_status(
    session: &mut super::agent_workflow_state::AgentUiSession,
    call_id: &str,
    status: super::agent_workflow_state::AgentUiActivityStatus,
) {
    if let Some(activity) = session
        .activities
        .iter_mut()
        .rev()
        .find(|activity| activity.call_id.as_deref() == Some(call_id))
    {
        activity.status = status;
    }
}

pub(super) fn finish_session(
    session: &mut super::agent_workflow_state::AgentUiSession,
    state: db_pro_core::domain::agent::AgentSessionState,
) {
    let terminal_activity_status = match state {
        db_pro_core::domain::agent::AgentSessionState::Completed => {
            Some(super::agent_workflow_state::AgentUiActivityStatus::Success)
        }
        db_pro_core::domain::agent::AgentSessionState::Failed => {
            Some(super::agent_workflow_state::AgentUiActivityStatus::Failed)
        }
        db_pro_core::domain::agent::AgentSessionState::Cancelled => {
            Some(super::agent_workflow_state::AgentUiActivityStatus::Cancelled)
        }
        _ => None,
    };
    if let Some(status) = terminal_activity_status {
        for activity in &mut session.activities {
            if matches!(
                activity.status,
                super::agent_workflow_state::AgentUiActivityStatus::Running
                    | super::agent_workflow_state::AgentUiActivityStatus::AwaitingConfirmation
            ) {
                activity.status = status;
            }
        }
    }
    flush_stream(session);
    session.state = state;
    session.active_run_id = None;
    session.request_id = None;
    session.pending_confirmation = None;
}

fn tool_label(tool: db_pro_core::domain::agent::AgentTool) -> String {
    use db_pro_core::domain::agent::AgentTool;
    match tool {
        AgentTool::InspectSchema => "Inspecting schema".to_owned(),
        AgentTool::InspectTable => "Reading table metadata".to_owned(),
        AgentTool::InspectColumns => "Reading columns".to_owned(),
        AgentTool::InspectForeignKeys => "Reading foreign keys".to_owned(),
        AgentTool::GetCurrentQuery => "Reading current query".to_owned(),
        AgentTool::PatchQuery => "Preparing SQL change".to_owned(),
        AgentTool::RunQuery => "Running query".to_owned(),
        AgentTool::InspectQueryResult => "Inspecting query result".to_owned(),
        AgentTool::ExplainQuery => "Explaining query".to_owned(),
        AgentTool::SuggestIndexes => "Suggesting indexes".to_owned(),
        AgentTool::MonitoringRead => "Reading monitoring snapshot".to_owned(),
    }
}
