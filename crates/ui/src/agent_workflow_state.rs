use std::collections::HashMap;

use db_pro_core::domain::agent::{
    AgentMode, AgentRunId, AgentSession, AgentSessionState, AgentSqlSafety, AgentTool, AgentToolOutput,
};
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
    pub(super) call_id: Option<String>,
    pub(super) tool: Option<AgentTool>,
    pub(super) label: String,
    pub(super) status: AgentUiActivityStatus,
    pub(super) duration_ms: Option<u64>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct AgentUiToolResult {
    pub(super) call_id: String,
    pub(super) tool: AgentTool,
    pub(super) output: AgentToolOutput,
    pub(super) duration_ms: Option<u64>,
    pub(super) status: AgentUiActivityStatus,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct AgentUiConfirmation {
    pub(super) run_id: AgentRunId,
    pub(super) call_id: String,
    pub(super) kind: AgentConfirmationKind,
    pub(super) preview: Option<AgentToolOutput>,
    pub(super) document_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct AgentAuditEntry {
    pub(super) run_id: AgentRunId,
    pub(super) tool: AgentTool,
    pub(super) duration_ms: Option<u64>,
    pub(super) safety: Option<AgentSqlSafety>,
    pub(super) confirmed: bool,
}

#[derive(Debug)]
pub(crate) struct AgentUiSession {
    pub(super) session: Option<AgentSession>,
    pub(super) active_run_id: Option<AgentRunId>,
    pub(super) mode: AgentMode,
    pub(super) state: AgentSessionState,
    pub(super) messages: Vec<AgentMessage>,
    pub(super) activities: Vec<AgentUiActivity>,
    pub(super) pending_confirmation: Option<AgentUiConfirmation>,
    pub(super) streaming_text: String,
    pub(super) tool_results: HashMap<String, AgentUiToolResult>,
    pub(super) audit_trail: Vec<AgentAuditEntry>,
    pub(super) request_id: Option<RequestId>,
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
            audit_trail: Vec::new(),
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
