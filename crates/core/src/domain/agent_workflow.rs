use serde::{Deserialize, Serialize};
use thiserror::Error;

use super::agent::{
    execution_decision, AgentExecutionDecision, AgentMode, AgentPatchError, AgentRunId, AgentSession,
    AgentSessionError, AgentSessionId, AgentSessionState, AgentSqlSafety, AgentTool, AgentToolInput, AgentToolOutput,
    AgentToolRequest,
};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PendingAgentConfirmation {
    pub run_id: AgentRunId,
    pub session_id: AgentSessionId,
    pub document_id: String,
    pub document_version: u64,
    pub tool: AgentTool,
    pub sql: Option<String>,
    pub safety: AgentSqlSafety,
    pub reason: String,
    request: AgentToolRequest,
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

#[derive(Debug, Error, Clone, PartialEq, Eq)]
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
    #[error("agent query failed: {code}: {message}")]
    QueryFailed { code: String, message: String },
    #[error("agent run was cancelled")]
    Cancelled,
    #[error("agent SQL patch is stale or invalid: {0}")]
    InvalidPatch(#[source] AgentPatchError),
}

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
        if request.document_version != current_document_version {
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

        if request.tool == AgentTool::RunQuery {
            let AgentToolInput::Query { sql } = &request.input else {
                return Err(AgentToolError::InvalidInput {
                    tool: AgentTool::RunQuery,
                });
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
            return self.require_confirmation(request, safety, confirmation_reason(safety));
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
            self.pending_confirmation = Some(pending);
            return Err(AgentToolError::StaleDocument {
                expected: self
                    .session
                    .active_run
                    .as_ref()
                    .map_or(current_document_version, |run| run.document_version),
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
        if request.document_version != run.document_version {
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
            Some(AgentSqlSafety::Mutating),
            "Review and apply this SQL patch.",
        );
        self.session.state = AgentSessionState::AwaitingConfirmation;
        self.pending_confirmation = Some(confirmation.clone());
        Ok(AgentToolDisposition::PatchPreview { preview, confirmation })
    }

    fn require_confirmation(
        &mut self,
        request: AgentToolRequest,
        safety: AgentSqlSafety,
        reason: &str,
    ) -> Result<AgentToolDisposition, AgentToolError> {
        let confirmation = self.pending_confirmation_for(request, Some(safety), reason);
        self.session.state = AgentSessionState::AwaitingConfirmation;
        self.pending_confirmation = Some(confirmation.clone());
        Ok(AgentToolDisposition::ConfirmationRequired(confirmation))
    }

    fn pending_confirmation_for(
        &self,
        request: AgentToolRequest,
        safety: Option<AgentSqlSafety>,
        reason: &str,
    ) -> PendingAgentConfirmation {
        let run_id = self.session.active_run.as_ref().map(|run| run.id).unwrap_or_default();
        let sql = match &request.input {
            AgentToolInput::Query { sql } => Some(sql.clone()),
            AgentToolInput::Patch { patch } => Some(patch.replacement.clone()),
            _ => None,
        };
        PendingAgentConfirmation {
            run_id,
            session_id: self.session.id,
            document_id: request.document_id.clone(),
            document_version: request.document_version,
            tool: request.tool,
            sql,
            safety: safety.unwrap_or(AgentSqlSafety::ReadOnly),
            reason: reason.to_owned(),
            request,
        }
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
        ),
        AgentMode::Edit => matches!(
            tool,
            AgentTool::InspectSchema
                | AgentTool::InspectTable
                | AgentTool::InspectColumns
                | AgentTool::InspectForeignKeys
                | AgentTool::GetCurrentQuery
                | AgentTool::InspectQueryResult
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
        (
            AgentTool::GetCurrentQuery | AgentTool::InspectQueryResult | AgentTool::ExplainQuery,
            AgentToolInput::None,
        ) => true,
        (AgentTool::PatchQuery, AgentToolInput::Patch { .. }) => true,
        (AgentTool::RunQuery, AgentToolInput::Query { sql }) => !sql.trim().is_empty(),
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

fn map_session_error(error: AgentSessionError) -> AgentToolError {
    match error {
        AgentSessionError::RunAlreadyActive => AgentToolError::RunAlreadyActive,
        AgentSessionError::RunMismatch => AgentToolError::RunMismatch,
    }
}

#[cfg(test)]
mod tests {
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
    fn stale_tool_request_is_rejected() {
        let mut workflow = workflow(AgentMode::Agent, true);
        workflow.start_run(8).expect("run starts");
        let mut tool_request = request(&workflow, AgentTool::GetCurrentQuery, AgentToolInput::None);
        tool_request.document_version = 9;
        assert_eq!(
            workflow.request_tool(tool_request, 8, None),
            Err(AgentToolError::StaleDocument { expected: 8, actual: 9 })
        );
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
        assert!(matches!(disposition, AgentToolDisposition::PatchPreview { .. }));
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
}
