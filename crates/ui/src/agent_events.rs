//! Runtime events owned by the feature slice.

use super::*;
use crate::RequestId;

impl DbProApp {
    pub(super) fn on_agent_completed(&mut self, _request_id: RequestId, provider: String, message: AgentMessage) {
        let provider_detail = format!("{provider} Responses API · SQL drafts stay unexecuted");
        self.agent.provider_label = provider;
        self.agent.provider_detail = provider_detail;
        self.agent.messages.push(message);
        self.feedback.runtime_message = "Agent response received".to_owned();
    }

    pub(super) fn on_agent_failed(&mut self, request_id: RequestId, message: String) {
        if let Some(session) = self
            .agent
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
            self.feedback.runtime_message = "Agent workflow failed".to_owned();
        }
    }

    pub(super) fn on_agent_configured(&mut self, request_id: RequestId, provider: String, detail: String) {
        if self.agent.configure_request != Some(request_id) {
            return;
        }
        self.agent.configure_request = None;
        self.agent.provider_label = provider.clone();
        self.agent.provider_detail = detail;
        self.agent.settings_open = false;
        self.agent.api_key_draft.clear();
        self.agent.api_key_show_password = false;
        let message = format!("{provider} API key saved · provider active");
        self.feedback.runtime_message = message.clone();
        self.show_toast_success(message);
    }

    pub(super) fn on_agent_forgotten(&mut self, request_id: RequestId) {
        if self.agent.configure_request != Some(request_id) {
            return;
        }
        self.agent.configure_request = None;
        self.agent.provider_label = "Offline draft".to_owned();
        self.agent.provider_detail = "AI provider not configured · local drafts stay unexecuted".to_owned();
        self.agent.settings_open = false;
        self.agent.api_key_draft.clear();
        self.agent.api_key_show_password = false;
        let message = "API key forgotten · provider inactive".to_owned();
        self.feedback.runtime_message = message.clone();
        self.show_toast_success(message);
    }
}
