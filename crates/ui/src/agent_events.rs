//! Agent runtime event reducers.

use super::*;
use crate::RequestId;

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
