use serde::{Deserialize, Serialize};
use thiserror::Error;

use super::agent::{
    allows_stale_document_version, execution_decision, AgentExecutionDecision, AgentMode, AgentPatchError, AgentRunId,
    AgentSession, AgentSessionError, AgentSessionId, AgentSessionState, AgentSqlSafety, AgentTool, AgentToolCall,
    AgentToolInput, AgentToolOutput, AgentToolRequest, AgentToolResult,
};
use super::agent_context::AgentResultSummary;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AgentExecutionContext {
    pub session: AgentSession,
    pub document: Option<super::agent::AgentDocumentSnapshot>,
    pub latest_result: Option<AgentResultSummary>,
    pub result_count: usize,
    pub mode: AgentMode,
    pub allow_read_only_auto_run: bool,
    pub confirmed: bool,
}

impl AgentExecutionContext {
    pub fn new(session: AgentSession, document: super::agent::AgentDocumentSnapshot, mode: AgentMode) -> Self {
        Self {
            session,
            document: Some(document),
            latest_result: None,
            result_count: 0,
            mode,
            allow_read_only_auto_run: false,
            confirmed: false,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PendingAgentConfirmation {
    pub run_id: AgentRunId,
    pub session_id: AgentSessionId,
    pub document_id: String,
    pub document_version: u64,
    pub tool: AgentTool,
    pub input_fingerprint: String,
    pub sql: Option<String>,
    pub safety: Option<AgentSqlSafety>,
    pub kind: AgentConfirmationKind,
    pub reason: String,
    pub request: AgentToolRequest,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AgentConfirmationKind {
    ApplyPatch,
    RunReadOnly,
    RunMutation,
    RunDestructive,
    RunUnknown,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AgentWorkflowEvent {
    TextDelta {
        run_id: AgentRunId,
        session_id: AgentSessionId,
        document_id: String,
        delta: String,
    },
    ToolRequested {
        run_id: AgentRunId,
        session_id: AgentSessionId,
        document_id: String,
        call: AgentToolCall,
    },
    ToolCompleted {
        run_id: AgentRunId,
        session_id: AgentSessionId,
        document_id: String,
        call_id: String,
        result: AgentToolResult,
    },
    ToolFailed {
        run_id: AgentRunId,
        session_id: AgentSessionId,
        document_id: String,
        call_id: String,
        tool: AgentTool,
        error: AgentToolError,
    },
    ConfirmationRequired {
        run_id: AgentRunId,
        session_id: AgentSessionId,
        document_id: String,
        call_id: String,
        kind: AgentConfirmationKind,
        preview: Option<AgentToolOutput>,
    },
    Completed {
        run_id: AgentRunId,
        session_id: AgentSessionId,
        document_id: String,
    },
    Failed {
        run_id: AgentRunId,
        session_id: AgentSessionId,
        document_id: String,
        message: String,
    },
    Cancelled {
        run_id: AgentRunId,
        session_id: AgentSessionId,
        document_id: String,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AgentToolDisposition {
    Execute(AgentToolRequest),
    PatchPreview {
        preview: AgentToolOutput,
        confirmation: PendingAgentConfirmation,
    },
    ConfirmationRequired(PendingAgentConfirmation),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AgentConfirmationResult {
    Approved(AgentToolRequest),
    Rejected,
}

#[derive(Debug, Error, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AgentToolError {
    #[error("agent session does not match the requested session")]
    SessionMismatch,
    #[error("agent request targets another document")]
    DocumentMismatch,
    #[error("agent document was not found")]
    DocumentNotFound,
    #[error("agent request is stale: expected version {expected}, current version {actual}")]
    StaleDocument { expected: u64, actual: u64 },
    #[error("agent run is not active")]
    RunNotActive,
    #[error("an agent run is already active")]
    RunAlreadyActive,
    #[error("agent run does not match the active run")]
    RunMismatch,
    #[error("tool {tool:?} is not allowed in {mode:?} mode")]
    PermissionDenied { tool: AgentTool, mode: AgentMode },
    #[error("tool {tool:?} received an invalid input")]
    InvalidInput { tool: AgentTool },
    #[error("the requested database connection is unavailable")]
    ConnectionUnavailable,
    #[error("schema object was not found: {name}")]
    SchemaObjectNotFound { name: String },
    #[error("agent query result is not available")]
    ResultUnavailable,
    #[error("agent action requires confirmation: {kind:?}")]
    ConfirmationRequired { kind: AgentConfirmationKind },
    #[error("agent confirmation was rejected: {kind:?}")]
    ConfirmationRejected { kind: AgentConfirmationKind },
    #[error("agent query failed: {code}: {message}")]
    QueryFailed {
        code: String,
        message: String,
        position: Option<usize>,
        detail: Option<String>,
        hint: Option<String>,
    },
    #[error("agent run was cancelled")]
    Cancelled,
    #[error("agent stopped after too many tool steps")]
    MaxStepsExceeded,
    #[error("agent SQL patch is stale or invalid: {0}")]
    InvalidPatch(#[source] AgentPatchError),
    #[error("provider protocol error: {0}")]
    ProviderProtocolError(String),
}

impl AgentToolError {
    pub fn format_user_error(&self) -> String {
        match self {
            AgentToolError::QueryFailed {
                message, detail, hint, ..
            } => format_query_failed(message, detail.as_deref(), hint.as_deref()),
            AgentToolError::MaxStepsExceeded => "Agent stopped after too many tool steps (limit reached).".to_owned(),
            AgentToolError::ConfirmationRejected { kind } => {
                format!("{} was cancelled by user.", rejected_confirmation_label(kind))
            }
            AgentToolError::ConfirmationRequired { kind } => {
                format!("{} requires user confirmation.", required_confirmation_label(kind))
            }
            AgentToolError::StaleDocument { expected, actual } => {
                format!(
                    "The query editor changed (expected version {expected}, current version {actual}). Please retry."
                )
            }
            AgentToolError::PermissionDenied { tool, mode } => format_permission_denied(tool, mode),
            AgentToolError::SchemaObjectNotFound { name } => {
                format!("Database object '{name}' was not found.")
            }
            AgentToolError::SessionMismatch => "Agent session mismatch. Please try again.".to_owned(),
            AgentToolError::DocumentMismatch => "Target query tab is no longer active.".to_owned(),
            AgentToolError::DocumentNotFound => "Query document not found.".to_owned(),
            AgentToolError::RunNotActive => "Agent run is not active.".to_owned(),
            AgentToolError::RunAlreadyActive => "An agent run is already in progress.".to_owned(),
            AgentToolError::RunMismatch => "Agent run mismatch.".to_owned(),
            AgentToolError::InvalidInput { .. } => "Agent received invalid input parameters.".to_owned(),
            AgentToolError::ConnectionUnavailable => "No active database connection.".to_owned(),
            AgentToolError::ResultUnavailable => "No query result is available to inspect.".to_owned(),
            AgentToolError::Cancelled => "Agent run was stopped.".to_owned(),
            AgentToolError::InvalidPatch(err) => format!("Could not apply SQL patch: {err}"),
            AgentToolError::ProviderProtocolError(msg) => format!("AI Provider error: {msg}"),
        }
    }
}

fn format_query_failed(message: &str, detail: Option<&str>, hint: Option<&str>) -> String {
    let mut output = message.to_owned();
    if let Some(detail) = detail {
        output.push_str(&format!("\nDetail: {detail}"));
    }
    if let Some(hint) = hint {
        output.push_str(&format!("\nHint: {hint}"));
    }
    output
}

fn rejected_confirmation_label(kind: &AgentConfirmationKind) -> &'static str {
    match kind {
        AgentConfirmationKind::ApplyPatch => "Patch application",
        AgentConfirmationKind::RunReadOnly => "Read-only query execution",
        AgentConfirmationKind::RunMutation => "Mutation execution",
        AgentConfirmationKind::RunDestructive => "Destructive query execution",
        AgentConfirmationKind::RunUnknown => "Query execution",
    }
}

fn required_confirmation_label(kind: &AgentConfirmationKind) -> &'static str {
    match kind {
        AgentConfirmationKind::ApplyPatch => "Applying patch",
        AgentConfirmationKind::RunReadOnly => "Running read-only query",
        AgentConfirmationKind::RunMutation => "Running mutation",
        AgentConfirmationKind::RunDestructive => "Running destructive query",
        AgentConfirmationKind::RunUnknown => "Running query",
    }
}

fn format_permission_denied(tool: &AgentTool, mode: &AgentMode) -> String {
    format!(
        "{} is not permitted in {} mode.",
        permission_tool_label(tool),
        mode_label(mode)
    )
}

fn permission_tool_label(tool: &AgentTool) -> &'static str {
    match tool {
        AgentTool::PatchQuery => "Modifying query text",
        AgentTool::RunQuery => "Executing queries",
        AgentTool::ExplainQuery => "Explaining queries",
        AgentTool::SuggestIndexes => "Suggesting indexes",
        AgentTool::MonitoringRead => "Reading monitoring snapshot",
        AgentTool::InspectQueryResult => "Inspecting previous results",
        _ => "This operation",
    }
}

fn mode_label(mode: &AgentMode) -> &'static str {
    match mode {
        AgentMode::Ask => "Ask",
        AgentMode::Edit => "Edit",
        AgentMode::Agent => "Agent",
    }
}

#[derive(Debug)]
pub struct AgentWorkflow {
    session: AgentSession,
    mode: AgentMode,
    allow_read_only_auto_run: bool,
    pending_confirmation: Option<PendingAgentConfirmation>,
}

impl AgentWorkflow {
    pub fn new(session: AgentSession, mode: AgentMode, allow_read_only_auto_run: bool) -> Self {
        Self {
            session,
            mode,
            allow_read_only_auto_run,
            pending_confirmation: None,
        }
    }

    pub fn session(&self) -> &AgentSession {
        &self.session
    }

    pub fn mode(&self) -> AgentMode {
        self.mode
    }

    pub fn allow_read_only_auto_run(&self) -> bool {
        self.allow_read_only_auto_run
    }

    pub fn active_run_id(&self) -> Result<AgentRunId, AgentToolError> {
        self.session
            .active_run
            .as_ref()
            .map(|run| run.id)
            .ok_or(AgentToolError::RunNotActive)
    }

    pub fn refresh_document_version(
        &mut self,
        run_id: AgentRunId,
        document_version: u64,
    ) -> Result<(), AgentToolError> {
        let Some(run) = self.session.active_run.as_mut() else {
            return Err(AgentToolError::RunNotActive);
        };
        if run.id != run_id {
            return Err(AgentToolError::RunMismatch);
        }
        run.document_version = document_version;
        Ok(())
    }

    pub fn pending_confirmation(&self) -> Option<&PendingAgentConfirmation> {
        self.pending_confirmation.as_ref()
    }

    pub fn start_run(&mut self, document_version: u64) -> Result<AgentRunId, AgentToolError> {
        self.session.start_run(document_version).map_err(map_session_error)
    }

    pub fn request_tool(
        &mut self,
        request: AgentToolRequest,
        current_document_version: u64,
        current_text: Option<&str>,
    ) -> Result<AgentToolDisposition, AgentToolError> {
        self.validate_request(&request)?;
        if request.document_version != current_document_version && !allows_stale_document_version(request.tool) {
            return Err(AgentToolError::StaleDocument {
                expected: request.document_version,
                actual: current_document_version,
            });
        }
        if !is_tool_allowed(self.mode, request.tool) {
            return Err(AgentToolError::PermissionDenied {
                tool: request.tool,
                mode: self.mode,
            });
        }
        validate_tool_input(&request)?;

        if request.tool == AgentTool::PatchQuery {
            return self.prepare_patch_preview(
                request,
                current_text.ok_or(AgentToolError::InvalidInput {
                    tool: AgentTool::PatchQuery,
                })?,
            );
        }

        if matches!(request.tool, AgentTool::RunQuery | AgentTool::ExplainQuery) {
            let AgentToolInput::Query { sql } = &request.input else {
                return Err(AgentToolError::InvalidInput { tool: request.tool });
            };
            let safety = AgentSqlSafety::classify(sql);
            let decision = execution_decision(self.mode, safety, self.allow_read_only_auto_run);
            if decision == AgentExecutionDecision::Allowed {
                return Ok(AgentToolDisposition::Execute(request));
            }
            if decision == AgentExecutionDecision::Rejected {
                return Err(AgentToolError::PermissionDenied {
                    tool: request.tool,
                    mode: self.mode,
                });
            }
            return self.require_confirmation(request, safety);
        }

        Ok(AgentToolDisposition::Execute(request))
    }

    pub fn confirm(
        &mut self,
        run_id: AgentRunId,
        current_document_version: u64,
    ) -> Result<AgentConfirmationResult, AgentToolError> {
        let Some(pending) = self.pending_confirmation.take() else {
            return Err(AgentToolError::RunNotActive);
        };
        if pending.run_id != run_id {
            self.pending_confirmation = Some(pending);
            return Err(AgentToolError::RunMismatch);
        }
        if pending.document_version != current_document_version {
            let expected_version = pending.document_version;
            self.pending_confirmation = Some(pending);
            return Err(AgentToolError::StaleDocument {
                expected: expected_version,
                actual: current_document_version,
            });
        }
        self.session.state = AgentSessionState::Running;
        Ok(AgentConfirmationResult::Approved(pending.request))
    }

    pub fn reject(&mut self, run_id: AgentRunId) -> Result<AgentConfirmationResult, AgentToolError> {
        let Some(pending) = self.pending_confirmation.take() else {
            return Err(AgentToolError::RunNotActive);
        };
        if pending.run_id != run_id {
            self.pending_confirmation = Some(pending);
            return Err(AgentToolError::RunMismatch);
        }
        self.session.state = AgentSessionState::Running;
        Ok(AgentConfirmationResult::Rejected)
    }

    pub fn complete(&mut self, run_id: AgentRunId) -> Result<(), AgentToolError> {
        self.finish(run_id, AgentSessionState::Completed)
    }

    pub fn fail(&mut self, run_id: AgentRunId) -> Result<(), AgentToolError> {
        self.finish(run_id, AgentSessionState::Failed)
    }

    pub fn cancel(&mut self, run_id: AgentRunId) -> Result<(), AgentToolError> {
        self.finish(run_id, AgentSessionState::Cancelled)
    }

    fn validate_request(&self, request: &AgentToolRequest) -> Result<(), AgentToolError> {
        if request.session_id != self.session.id {
            return Err(AgentToolError::SessionMismatch);
        }
        if request.document_id != self.session.document_id {
            return Err(AgentToolError::DocumentMismatch);
        }
        let Some(run) = self.session.active_run.as_ref() else {
            return Err(AgentToolError::RunNotActive);
        };
        if request.run_id != run.id {
            return Err(AgentToolError::RunMismatch);
        }
        if request.document_version != run.document_version && !allows_stale_document_version(request.tool) {
            return Err(AgentToolError::StaleDocument {
                expected: run.document_version,
                actual: request.document_version,
            });
        }
        Ok(())
    }

    fn prepare_patch_preview(
        &mut self,
        request: AgentToolRequest,
        current_text: &str,
    ) -> Result<AgentToolDisposition, AgentToolError> {
        let AgentToolInput::Patch { patch } = &request.input else {
            return Err(AgentToolError::InvalidInput {
                tool: AgentTool::PatchQuery,
            });
        };
        let proposed = patch
            .apply_to(&request.document_id, request.document_version, current_text)
            .map_err(AgentToolError::InvalidPatch)?;
        let (start, end) = patch.range;
        let preview = AgentToolOutput::PatchPreview {
            patch: patch.clone(),
            original: current_text[start..end].to_owned(),
            proposed: proposed.clone(),
        };
        let confirmation = self.pending_confirmation_for(
            request,
            None,
            AgentConfirmationKind::ApplyPatch,
            "Review and apply this SQL patch.",
        )?;
        self.session.state = AgentSessionState::AwaitingConfirmation;
        self.pending_confirmation = Some(confirmation.clone());
        Ok(AgentToolDisposition::PatchPreview { preview, confirmation })
    }

    fn require_confirmation(
        &mut self,
        request: AgentToolRequest,
        safety: AgentSqlSafety,
    ) -> Result<AgentToolDisposition, AgentToolError> {
        let confirmation = self.pending_confirmation_for(
            request,
            Some(safety),
            confirmation_kind(safety),
            confirmation_reason(safety),
        )?;
        self.session.state = AgentSessionState::AwaitingConfirmation;
        self.pending_confirmation = Some(confirmation.clone());
        Ok(AgentToolDisposition::ConfirmationRequired(confirmation))
    }

    fn pending_confirmation_for(
        &self,
        request: AgentToolRequest,
        safety: Option<AgentSqlSafety>,
        kind: AgentConfirmationKind,
        reason: &str,
    ) -> Result<PendingAgentConfirmation, AgentToolError> {
        let Some(run) = self.session.active_run.as_ref() else {
            return Err(AgentToolError::RunNotActive);
        };
        let input_fingerprint = request.input.fingerprint();
        let sql = match &request.input {
            AgentToolInput::Query { sql } => Some(sql.clone()),
            AgentToolInput::Patch { patch } => Some(patch.replacement.clone()),
            _ => None,
        };
        Ok(PendingAgentConfirmation {
            run_id: run.id,
            session_id: self.session.id,
            document_id: request.document_id.clone(),
            document_version: request.document_version,
            tool: request.tool,
            input_fingerprint,
            sql,
            safety,
            kind,
            reason: reason.to_owned(),
            request,
        })
    }

    fn finish(&mut self, run_id: AgentRunId, state: AgentSessionState) -> Result<(), AgentToolError> {
        self.pending_confirmation = None;
        self.session.finish_run(run_id, state).map_err(map_session_error)
    }
}

pub fn is_tool_allowed(mode: AgentMode, tool: AgentTool) -> bool {
    match mode {
        AgentMode::Ask => matches!(
            tool,
            AgentTool::InspectSchema
                | AgentTool::InspectTable
                | AgentTool::InspectColumns
                | AgentTool::InspectForeignKeys
                | AgentTool::GetCurrentQuery
                | AgentTool::InspectQueryResult
                | AgentTool::ExplainQuery
                | AgentTool::SuggestIndexes
                | AgentTool::MonitoringRead
        ),
        AgentMode::Edit => matches!(
            tool,
            AgentTool::InspectSchema
                | AgentTool::InspectTable
                | AgentTool::InspectColumns
                | AgentTool::InspectForeignKeys
                | AgentTool::GetCurrentQuery
                | AgentTool::InspectQueryResult
                | AgentTool::ExplainQuery
                | AgentTool::SuggestIndexes
                | AgentTool::MonitoringRead
                | AgentTool::PatchQuery
        ),
        AgentMode::Agent => true,
    }
}

fn validate_tool_input(request: &AgentToolRequest) -> Result<(), AgentToolError> {
    let valid = match (request.tool, &request.input) {
        (AgentTool::InspectSchema, AgentToolInput::None | AgentToolInput::Schema { .. }) => true,
        (AgentTool::InspectTable | AgentTool::InspectColumns, AgentToolInput::Table { .. }) => true,
        (AgentTool::InspectForeignKeys, AgentToolInput::None | AgentToolInput::Table { .. }) => true,
        (AgentTool::GetCurrentQuery | AgentTool::InspectQueryResult, AgentToolInput::None) => true,
        (AgentTool::InspectQueryResult, AgentToolInput::ResultSample { max_rows, .. }) => *max_rows > 0,
        (AgentTool::PatchQuery, AgentToolInput::Patch { .. }) => true,
        (AgentTool::RunQuery | AgentTool::ExplainQuery, AgentToolInput::Query { sql }) => !sql.trim().is_empty(),
        (AgentTool::SuggestIndexes, AgentToolInput::Table { .. }) => true,
        (AgentTool::MonitoringRead, AgentToolInput::None) => true,
        _ => false,
    };
    if valid {
        Ok(())
    } else {
        Err(AgentToolError::InvalidInput { tool: request.tool })
    }
}

fn confirmation_reason(safety: AgentSqlSafety) -> &'static str {
    match safety {
        AgentSqlSafety::ReadOnly => "Read-only execution requires confirmation in the current mode.",
        AgentSqlSafety::Mutating => "This query changes database data or schema.",
        AgentSqlSafety::Destructive => "This query may irreversibly change or remove database data.",
        AgentSqlSafety::Unknown => "The query safety could not be determined.",
    }
}

fn confirmation_kind(safety: AgentSqlSafety) -> AgentConfirmationKind {
    match safety {
        AgentSqlSafety::ReadOnly => AgentConfirmationKind::RunReadOnly,
        AgentSqlSafety::Mutating => AgentConfirmationKind::RunMutation,
        AgentSqlSafety::Destructive => AgentConfirmationKind::RunDestructive,
        AgentSqlSafety::Unknown => AgentConfirmationKind::RunUnknown,
    }
}

fn map_session_error(error: AgentSessionError) -> AgentToolError {
    match error {
        AgentSessionError::RunAlreadyActive => AgentToolError::RunAlreadyActive,
        AgentSessionError::RunMismatch => AgentToolError::RunMismatch,
    }
}

#[cfg(test)]
#[path = "agent_workflow/tests.rs"]
mod tests;
