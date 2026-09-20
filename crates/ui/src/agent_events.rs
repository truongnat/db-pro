//! Agent runtime event reducers.

use super::*;
use crate::RequestId;

/// Applies a workflow event only to its matching document session.
pub(crate) fn on_agent_workflow_event(
    agent: &mut AgentState,
    event: db_pro_core::domain::agent_workflow::AgentWorkflowEvent,
) {
    use db_pro_core::domain::agent_workflow::AgentWorkflowEvent;

    let document_id = match &event {
        AgentWorkflowEvent::TextDelta { document_id, .. }
        | AgentWorkflowEvent::ToolRequested { document_id, .. }
        | AgentWorkflowEvent::ToolCompleted { document_id, .. }
        | AgentWorkflowEvent::ToolFailed { document_id, .. }
        | AgentWorkflowEvent::ConfirmationRequired { document_id, .. }
        | AgentWorkflowEvent::Completed { document_id, .. }
        | AgentWorkflowEvent::Failed { document_id, .. }
        | AgentWorkflowEvent::Cancelled { document_id, .. } => document_id.clone(),
    };
    let Some(session) = agent.sessions.get_mut(&document_id) else {
        return;
    };
    let (session_id, run_id) = match &event {
        AgentWorkflowEvent::TextDelta { session_id, run_id, .. }
        | AgentWorkflowEvent::ToolRequested { session_id, run_id, .. }
        | AgentWorkflowEvent::ToolCompleted { session_id, run_id, .. }
        | AgentWorkflowEvent::ToolFailed { session_id, run_id, .. }
        | AgentWorkflowEvent::ConfirmationRequired { session_id, run_id, .. }
        | AgentWorkflowEvent::Completed { session_id, run_id, .. }
        | AgentWorkflowEvent::Failed { session_id, run_id, .. }
        | AgentWorkflowEvent::Cancelled { session_id, run_id, .. } => (*session_id, *run_id),
    };
    if session.session.as_ref().map(|value| value.id) != Some(session_id)
        || session.active_run_id.is_some_and(|active| active != run_id)
    {
        return;
    }
    if matches!(
        session.state,
        db_pro_core::domain::agent::AgentSessionState::Completed
            | db_pro_core::domain::agent::AgentSessionState::Failed
            | db_pro_core::domain::agent::AgentSessionState::Cancelled
    ) {
        return;
    }
    if session.active_run_id.is_none() {
        if session.state != db_pro_core::domain::agent::AgentSessionState::Running {
            return;
        }
        session.active_run_id = Some(run_id);
    }

    super::agent_state::apply_agent_workflow_event(session, event, run_id);
}

/// Applies a provider configuration failure only to its matching request.
pub(crate) fn handle_agent_request_failure(
    agent: &mut AgentState,
    feedback: &mut FeedbackState,
    request_id: RequestId,
    message: &str,
) -> bool {
    if agent.configure_request != Some(request_id) {
        return false;
    }
    agent.configure_request = None;
    let runtime_message = format!("Agent key operation failed · {message}");
    feedback.set_runtime_message(runtime_message.clone());
    feedback.show_error_toast(runtime_message);
    true
}

pub(crate) fn on_agent_provider_ready(agent: &mut AgentState, provider: String, detail: String) {
    agent.provider_label = provider;
    agent.provider_detail = detail;
}

pub(crate) fn on_agent_failed(
    agent: &mut AgentState,
    feedback: &mut FeedbackState,
    request_id: RequestId,
    message: String,
) {
    if let Some(session) = agent
        .sessions
        .values_mut()
        .find(|session| session.request_id == Some(request_id))
    {
        if !session.streaming_text.is_empty() {
            session.messages.push(AgentMessage {
                role: AgentRole::Assistant,
                content: std::mem::take(&mut session.streaming_text),
                sql: None,
                requires_confirmation: false,
            });
        }
        session.messages.push(AgentMessage {
            role: AgentRole::Assistant,
            content: message,
            sql: None,
            requires_confirmation: false,
        });
        super::agent_state::finish_agent_session(session, db_pro_core::domain::agent::AgentSessionState::Failed);
        feedback.set_runtime_message("Agent workflow failed");
    }
}

pub(crate) fn on_agent_configured(
    agent: &mut AgentState,
    feedback: &mut FeedbackState,
    request_id: RequestId,
    provider: String,
    detail: String,
) {
    if agent.configure_request != Some(request_id) {
        return;
    }
    agent.configure_request = None;
    agent.provider_label = provider.clone();
    agent.provider_detail = detail;
    agent.settings_open = false;
    agent.api_key_draft.clear();
    agent.api_key_show_password = false;
    let message = format!("{provider} API key saved · provider active");
    feedback.set_runtime_message(message.clone());
    feedback.show_success_toast(message);
}

pub(crate) fn on_agent_forgotten(agent: &mut AgentState, feedback: &mut FeedbackState, request_id: RequestId) {
    if agent.configure_request != Some(request_id) {
        return;
    }
    agent.configure_request = None;
    agent.provider_label = "Offline draft".to_owned();
    agent.provider_detail = "AI provider not configured · local drafts stay unexecuted".to_owned();
    agent.settings_open = false;
    agent.api_key_draft.clear();
    agent.api_key_show_password = false;
    let message = "API key forgotten · provider inactive".to_owned();
    feedback.set_runtime_message(message.clone());
    feedback.show_success_toast(message);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn configuration_failure_is_scoped_to_the_pending_request() {
        let mut agent = AgentState {
            configure_request: Some(RequestId(4)),
            ..Default::default()
        };
        let mut feedback = FeedbackState::default();

        assert!(!handle_agent_request_failure(
            &mut agent,
            &mut feedback,
            RequestId(5),
            "stale"
        ));
        assert_eq!(agent.configure_request, Some(RequestId(4)));
        assert!(handle_agent_request_failure(
            &mut agent,
            &mut feedback,
            RequestId(4),
            "offline"
        ));
        assert!(agent.configure_request.is_none());
        assert_eq!(feedback.runtime_message, "Agent key operation failed · offline");
        assert_eq!(feedback.toasts.len(), 1);
    }

    #[test]
    fn provider_ready_updates_only_agent_provider_state() {
        let mut agent = AgentState::default();

        on_agent_provider_ready(&mut agent, "OpenAI".to_owned(), "configured".to_owned());

        assert_eq!(agent.provider_label, "OpenAI");
        assert_eq!(agent.provider_detail, "configured");
    }
}
