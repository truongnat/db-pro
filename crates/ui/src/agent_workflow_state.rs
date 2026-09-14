use std::collections::HashMap;

use db_pro_core::domain::agent::{AgentMode, AgentRunId, AgentSession, AgentSessionState, AgentTool, AgentToolOutput};
use db_pro_core::domain::agent_workflow::AgentConfirmationKind;

use crate::{AgentMessage, RequestId};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum AgentUiActivityStatus {
    Running,
    AwaitingConfirmation,
    Success,
    Failed,
    Cancelled,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct AgentUiActivity {
    pub call_id: Option<String>,
    pub tool: Option<AgentTool>,
    pub label: String,
    pub status: AgentUiActivityStatus,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct AgentUiConfirmation {
    pub run_id: AgentRunId,
    pub call_id: String,
    pub kind: AgentConfirmationKind,
    pub preview: Option<AgentToolOutput>,
}

#[derive(Debug)]
pub(crate) struct AgentUiSession {
    pub session: Option<AgentSession>,
    pub active_run_id: Option<AgentRunId>,
    pub mode: AgentMode,
    pub state: AgentSessionState,
    pub messages: Vec<AgentMessage>,
    pub activities: Vec<AgentUiActivity>,
    pub pending_confirmation: Option<AgentUiConfirmation>,
    pub streaming_text: String,
    pub tool_results: HashMap<String, AgentToolOutput>,
    pub request_id: Option<RequestId>,
}

impl Default for AgentUiSession {
    fn default() -> Self {
        Self {
            session: None,
            active_run_id: None,
            mode: AgentMode::Ask,
            state: AgentSessionState::Idle,
            messages: Vec::new(),
            activities: Vec::new(),
            pending_confirmation: None,
            streaming_text: String::new(),
            tool_results: HashMap::new(),
            request_id: None,
        }
    }
}

impl AgentUiSession {
    pub fn for_document(document_id: &str, connection_id: Option<String>, schema: Option<String>) -> Self {
        let session = AgentSession::new(document_id, connection_id, schema);
        Self {
            mode: AgentMode::Ask,
            state: AgentSessionState::Idle,
            session: Some(session),
            ..Self::default()
        }
    }
}
