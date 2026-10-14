use std::collections::{HashMap, HashSet};

use db_pro_core::domain::agent::{
    AgentTool, AgentToolCall, AgentToolInput, AgentToolOutput, AgentToolRequest, MAX_AGENT_TOOL_STEPS,
};
use db_pro_core::domain::agent_context::AgentContext;
use db_pro_core::domain::agent_workflow::{
    AgentConfirmationKind, AgentExecutionContext, AgentToolDisposition, AgentToolError, AgentWorkflow,
};

pub use db_pro_core::domain::agent_workflow::AgentWorkflowEvent;

use crate::agent::{AgentProvider, AgentProviderEvent, AgentProviderMessage, AgentProviderRequest};
use crate::agent_executor::AgentToolRunner;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CachedToolOutcome {
    Success(AgentToolOutput),
    Failed(AgentToolError),
    Rejected(AgentConfirmationKind),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CachedToolExecution {
    pub tool: AgentTool,
    pub input_fingerprint: String,
    pub outcome: CachedToolOutcome,
}

struct PendingToolCall {
    call_id: String,
    confirmation: db_pro_core::domain::agent_workflow::PendingAgentConfirmation,
}

pub struct AgentRunOrchestrator {
    provider: Box<dyn AgentProvider>,
    tool_runner: Box<dyn AgentToolRunner>,
    workflow: AgentWorkflow,
    execution_context: AgentExecutionContext,
    provider_request: AgentProviderRequest,
    seen_call_ids: HashSet<String>,
    completed_tool_calls: HashMap<String, CachedToolExecution>,
    tool_steps: usize,
    pending_tool: Option<PendingToolCall>,
}

impl AgentRunOrchestrator {
    pub fn new(
        provider: Box<dyn AgentProvider>,
        tool_runner: Box<dyn AgentToolRunner>,
        mut workflow: AgentWorkflow,
        mut execution_context: AgentExecutionContext,
        prompt: String,
        context: AgentContext,
    ) -> Result<Self, AgentToolError> {
        let document_version = execution_context
            .document
            .as_ref()
            .map(|document| document.document_version)
            .ok_or(AgentToolError::DocumentNotFound)?;
        let run_id = workflow.start_run(document_version)?;
        execution_context.session.active_run = workflow.session().active_run.clone();
        execution_context.mode = workflow.mode();
        execution_context.allow_read_only_auto_run = workflow.allow_read_only_auto_run();
        debug_assert_eq!(
            Some(run_id),
            execution_context.session.active_run.as_ref().map(|run| run.id)
        );
        Ok(Self {
            provider,
            tool_runner,
            workflow,
            execution_context,
            provider_request: AgentProviderRequest {
                prompt,
                context,
                messages: Vec::new(),
            },
            seen_call_ids: HashSet::new(),
            completed_tool_calls: HashMap::new(),
            tool_steps: 0,
            pending_tool: None,
        })
    }

    pub fn run_id(&self) -> Result<db_pro_core::domain::agent::AgentRunId, AgentToolError> {
        self.workflow.active_run_id()
    }

    pub fn workflow(&self) -> &AgentWorkflow {
        &self.workflow
    }

    pub fn has_pending_confirmation(&self) -> bool {
        self.pending_tool.is_some()
    }

    /// Record a user edit without advancing the run baseline. Strict tools will
    /// then return `StaleDocument` until the provider refreshes via
    /// `GetCurrentQuery`.
    pub fn update_document_snapshot(&mut self, document: db_pro_core::domain::agent::AgentDocumentSnapshot) {
        self.execution_context.document = Some(document);
    }

    pub async fn run(&mut self) -> Vec<AgentWorkflowEvent> {
        self.drive().await
    }

    pub fn cancel(&mut self) -> Result<AgentWorkflowEvent, AgentToolError> {
        let run_id = self.run_id()?;
        self.pending_tool = None;
        self.workflow.cancel(run_id)?;
        Ok(AgentWorkflowEvent::Cancelled {
            run_id,
            session_id: self.workflow.session().id,
            document_id: self.workflow.session().document_id.clone(),
        })
    }

    pub async fn resume_confirmation(
        &mut self,
        approved: bool,
        current_document: Option<db_pro_core::domain::agent::AgentDocumentSnapshot>,
        applied_patch: Option<AgentToolOutput>,
    ) -> Result<Vec<AgentWorkflowEvent>, AgentToolError> {
        let pending = self.pending_tool.take().ok_or(AgentToolError::RunNotActive)?;
        let pending_call_id = pending.call_id.clone();
        let pending_tool = pending.confirmation.tool;
        let pending_kind = pending.confirmation.kind;
        let fingerprint = pending.confirmation.input_fingerprint.clone();
        if let Some(document) = current_document {
            self.execution_context.document = Some(document);
        }
        let run_id = self.run_id()?;
        let current_version = self.current_document_version()?;
        if !approved {
            self.workflow.reject(run_id)?;
            self.completed_tool_calls.insert(
                pending_call_id.clone(),
                CachedToolExecution {
                    tool: pending_tool,
                    input_fingerprint: fingerprint,
                    outcome: CachedToolOutcome::Rejected(pending_kind),
                },
            );
            self.push_tool_error(
                pending_call_id.clone(),
                pending_tool,
                AgentToolError::ConfirmationRejected { kind: pending_kind },
            );
            return Ok(self.drive().await);
        }

        let confirmed = self.workflow.confirm(run_id, current_version)?;
        let request = match confirmed {
            db_pro_core::domain::agent_workflow::AgentConfirmationResult::Approved(request) => request,
            db_pro_core::domain::agent_workflow::AgentConfirmationResult::Rejected => {
                unreachable!("approved confirmation")
            }
        };

        // SAFETY RECHECK ON APPROVAL:
        if pending_tool == AgentTool::RunQuery {
            let AgentToolInput::Query { sql } = &request.input else {
                return Err(AgentToolError::InvalidInput {
                    tool: AgentTool::RunQuery,
                });
            };
            let current_safety = db_pro_core::domain::agent::AgentSqlSafety::classify(sql);
            if let Some(initial_safety) = pending.confirmation.safety {
                let safety_increased = matches!(
                    (initial_safety, current_safety),
                    (
                        db_pro_core::domain::agent::AgentSqlSafety::ReadOnly,
                        db_pro_core::domain::agent::AgentSqlSafety::Mutating
                            | db_pro_core::domain::agent::AgentSqlSafety::Destructive,
                    ) | (
                        db_pro_core::domain::agent::AgentSqlSafety::Mutating,
                        db_pro_core::domain::agent::AgentSqlSafety::Destructive,
                    )
                );
                if safety_increased {
                    return Err(AgentToolError::ConfirmationRequired {
                        kind: match current_safety {
                            db_pro_core::domain::agent::AgentSqlSafety::Destructive => {
                                AgentConfirmationKind::RunDestructive
                            }
                            db_pro_core::domain::agent::AgentSqlSafety::Mutating => AgentConfirmationKind::RunMutation,
                            _ => AgentConfirmationKind::RunUnknown,
                        },
                    });
                }
            }
        }

        let output = if pending.confirmation.kind == AgentConfirmationKind::ApplyPatch {
            let Some(AgentToolOutput::PatchApplied {
                document_id,
                new_version,
                range,
            }) = applied_patch
            else {
                return Err(AgentToolError::InvalidInput {
                    tool: AgentTool::PatchQuery,
                });
            };
            if document_id != request.document_id {
                return Err(AgentToolError::DocumentMismatch);
            }
            let AgentToolInput::Patch { patch } = &request.input else {
                return Err(AgentToolError::InvalidInput {
                    tool: AgentTool::PatchQuery,
                });
            };
            if new_version <= request.document_version {
                return Err(AgentToolError::InvalidInput {
                    tool: AgentTool::PatchQuery,
                });
            }
            let new_sql = self
                .execution_context
                .document
                .as_ref()
                .ok_or(AgentToolError::DocumentNotFound)
                .and_then(|document| {
                    patch
                        .apply_to(&document.document_id, document.document_version, &document.sql)
                        .map_err(AgentToolError::InvalidPatch)
                })?;
            self.workflow.refresh_document_version(run_id, new_version)?;
            if let Some(document) = self.execution_context.document.as_mut() {
                document.document_version = new_version;
                document.sql = new_sql.clone();
            }
            self.provider_request.context.document_version = new_version;
            self.provider_request.context.current_sql = new_sql;
            AgentToolOutput::PatchApplied {
                document_id,
                new_version,
                range,
            }
        } else {
            self.execution_context.confirmed = true;
            let result = self.tool_runner.execute(&request, &self.execution_context).await;
            self.execution_context.confirmed = false;
            match result {
                Ok(result) => result.output,
                Err(error) => {
                    let tool = request.tool;
                    self.completed_tool_calls.insert(
                        pending_call_id.clone(),
                        CachedToolExecution {
                            tool,
                            input_fingerprint: fingerprint,
                            outcome: CachedToolOutcome::Failed(error.clone()),
                        },
                    );
                    self.push_tool_error(pending_call_id.clone(), tool, error.clone());
                    let mut emitted = vec![AgentWorkflowEvent::ToolFailed {
                        run_id,
                        session_id: self.workflow.session().id,
                        document_id: pending.confirmation.document_id.clone(),
                        call_id: pending_call_id,
                        tool,
                        error,
                    }];
                    emitted.extend(self.drive().await);
                    return Ok(emitted);
                }
            }
        };
        self.completed_tool_calls.insert(
            pending_call_id.clone(),
            CachedToolExecution {
                tool: request.tool,
                input_fingerprint: fingerprint,
                outcome: CachedToolOutcome::Success(output.clone()),
            },
        );
        self.push_tool_result(pending_call_id, request.tool, output.clone());
        Ok(self.drive().await)
    }

    // cc-scan:allow DEEP_NESTING,LONG_FUNCTION — nesting mirrors structure depth
    async fn drive(&mut self) -> Vec<AgentWorkflowEvent> {
        let mut emitted = Vec::new();
        loop {
            let Ok(run_id) = self.run_id() else {
                return emitted;
            };
            let provider_events = match self.provider.complete(self.provider_request.clone()).await {
                Ok(events) => events,
                Err(error) => {
                    emitted.push(self.fail_run(run_id, error.to_string()));
                    return emitted;
                }
            };
            let mut handled_tool = false;
            for provider_event in provider_events {
                match provider_event {
                    AgentProviderEvent::TextDelta { delta } => emitted.push(AgentWorkflowEvent::TextDelta {
                        run_id,
                        session_id: self.workflow.session().id,
                        document_id: self.workflow.session().document_id.clone(),
                        delta,
                    }),
                    AgentProviderEvent::ToolCall(call) => {
                        handled_tool = true;
                        emitted.push(AgentWorkflowEvent::ToolRequested {
                            run_id,
                            session_id: self.workflow.session().id,
                            document_id: self.workflow.session().document_id.clone(),
                            call: call.clone(),
                        });
                        match self.handle_tool_call(call).await {
                            Ok(Some(event)) => {
                                emitted.push(event);
                                if self.pending_tool.is_some() {
                                    return emitted;
                                }
                            }
                            Ok(None) => {}
                            Err(error) => {
                                emitted.push(self.fail_run(run_id, error.to_string()));
                                return emitted;
                            }
                        }
                    }
                    AgentProviderEvent::Completed => {}
                }
            }
            if !handled_tool {
                emitted.push(self.complete_run(run_id));
                return emitted;
            }
        }
    }

    // cc-scan:allow COMPLEXITY — classifier/dispatch ladder — one case per branch
    async fn handle_tool_call(&mut self, call: AgentToolCall) -> Result<Option<AgentWorkflowEvent>, AgentToolError> {
        let fingerprint = call.input.fingerprint();
        if let Some(event) = self.pending_confirmation_event(&call, &fingerprint)? {
            return Ok(Some(event));
        }
        if let Some(event) = self.cached_call_event(&call, &fingerprint)? {
            return Ok(Some(event));
        }

        if !self.seen_call_ids.insert(call.call_id.clone()) {
            return Err(AgentToolError::InvalidInput { tool: call.tool });
        }
        self.tool_steps += 1;
        if self.tool_steps > MAX_AGENT_TOOL_STEPS {
            return Err(AgentToolError::MaxStepsExceeded);
        }
        let run_id = self.run_id()?;
        self.provider_request
            .messages
            .push(AgentProviderMessage::ToolCall(call.clone()));
        let request = self.request_for_call(&call)?;
        let current_version = self.current_document_version()?;
        let disposition = match self.workflow.request_tool(
            request,
            current_version,
            self.execution_context
                .document
                .as_ref()
                .map(|document| document.sql.as_str()),
        ) {
            Ok(disposition) => disposition,
            Err(error) => {
                return Ok(Some(self.cache_tool_failure(&call, fingerprint, run_id, error)));
            }
        };
        match disposition {
            AgentToolDisposition::Execute(request) => self.execute_tool_call(&call, fingerprint, run_id, request).await,
            AgentToolDisposition::PatchPreview { preview, confirmation } => {
                Ok(Some(self.require_confirmation(&call, confirmation, Some(preview))?))
            }
            AgentToolDisposition::ConfirmationRequired(confirmation) => {
                Ok(Some(self.require_confirmation(&call, confirmation, None)?))
            }
        }
    }

    /// A repeated call_id that matches the pending confirmation replays the
    /// confirmation event; the same id with a different tool or input is a
    /// provider protocol error.
    fn pending_confirmation_event(
        &self,
        call: &AgentToolCall,
        fingerprint: &str,
    ) -> Result<Option<AgentWorkflowEvent>, AgentToolError> {
        let Some(pending) = &self.pending_tool else {
            return Ok(None);
        };
        if pending.call_id != call.call_id {
            return Ok(None);
        }
        if pending.confirmation.tool != call.tool || pending.confirmation.input_fingerprint != fingerprint {
            return Err(AgentToolError::ProviderProtocolError(format!(
                "Pending confirmation collision for call_id '{}': mismatched tool ({:?} vs {:?}) or input",
                call.call_id, pending.confirmation.tool, call.tool
            )));
        }
        let preview = match &pending.confirmation.kind {
            AgentConfirmationKind::ApplyPatch => match &pending.confirmation.request.input {
                AgentToolInput::Patch { patch } => {
                    let current_text = self
                        .execution_context
                        .document
                        .as_ref()
                        .map(|d| d.sql.as_str())
                        .unwrap_or("");
                    let (start, end) = patch.range;
                    let original = current_text.get(start..end).unwrap_or("").to_owned();
                    let proposed = patch
                        .apply_to(
                            &pending.confirmation.document_id,
                            pending.confirmation.document_version,
                            current_text,
                        )
                        .unwrap_or_default();
                    Some(AgentToolOutput::PatchPreview {
                        patch: patch.clone(),
                        original,
                        proposed,
                    })
                }
                _ => None,
            },
            _ => None,
        };
        Ok(Some(AgentWorkflowEvent::ConfirmationRequired {
            run_id: self.run_id()?,
            session_id: self.workflow.session().id,
            document_id: self.workflow.session().document_id.clone(),
            call_id: call.call_id.clone(),
            kind: pending.confirmation.kind,
            preview,
        }))
    }

    /// A call_id already present in `completed_tool_calls` replays its cached
    /// outcome as the matching event without re-executing the tool.
    fn cached_call_event(
        &self,
        call: &AgentToolCall,
        fingerprint: &str,
    ) -> Result<Option<AgentWorkflowEvent>, AgentToolError> {
        let Some(cached) = self.completed_tool_calls.get(&call.call_id) else {
            return Ok(None);
        };
        if cached.tool != call.tool || cached.input_fingerprint != fingerprint {
            return Err(AgentToolError::ProviderProtocolError(format!(
                "Tool call collision for ID '{}': mismatched tool ({:?} vs {:?}) or input",
                call.call_id, cached.tool, call.tool
            )));
        }
        let run_id = self.run_id()?;
        match &cached.outcome {
            CachedToolOutcome::Success(output) => {
                let result = db_pro_core::domain::agent::AgentToolResult {
                    tool: call.tool,
                    output: output.clone(),
                };
                Ok(Some(AgentWorkflowEvent::ToolCompleted {
                    run_id,
                    session_id: self.workflow.session().id,
                    document_id: self.workflow.session().document_id.clone(),
                    call_id: call.call_id.clone(),
                    result,
                }))
            }
            CachedToolOutcome::Failed(error) => Ok(Some(AgentWorkflowEvent::ToolFailed {
                run_id,
                session_id: self.workflow.session().id,
                document_id: self.workflow.session().document_id.clone(),
                call_id: call.call_id.clone(),
                tool: call.tool,
                error: error.clone(),
            })),
            CachedToolOutcome::Rejected(kind) => Ok(Some(AgentWorkflowEvent::ToolFailed {
                run_id,
                session_id: self.workflow.session().id,
                document_id: self.workflow.session().document_id.clone(),
                call_id: call.call_id.clone(),
                tool: call.tool,
                error: AgentToolError::ConfirmationRejected { kind: *kind },
            })),
        }
    }

    /// Records a tool failure in the completed-call cache and the provider
    /// transcript, then builds the ToolFailed event for it.
    fn cache_tool_failure(
        &mut self,
        call: &AgentToolCall,
        fingerprint: String,
        run_id: db_pro_core::domain::agent::AgentRunId,
        error: AgentToolError,
    ) -> AgentWorkflowEvent {
        self.completed_tool_calls.insert(
            call.call_id.clone(),
            CachedToolExecution {
                tool: call.tool,
                input_fingerprint: fingerprint,
                outcome: CachedToolOutcome::Failed(error.clone()),
            },
        );
        self.push_tool_error(call.call_id.clone(), call.tool, error.clone());
        AgentWorkflowEvent::ToolFailed {
            run_id,
            session_id: self.workflow.session().id,
            document_id: self.workflow.session().document_id.clone(),
            call_id: call.call_id.clone(),
            tool: call.tool,
            error,
        }
    }

    async fn execute_tool_call(
        &mut self,
        call: &AgentToolCall,
        fingerprint: String,
        run_id: db_pro_core::domain::agent::AgentRunId,
        request: AgentToolRequest,
    ) -> Result<Option<AgentWorkflowEvent>, AgentToolError> {
        match self.tool_runner.execute(&request, &self.execution_context).await {
            Ok(result) => {
                let output = result.output.clone();
                self.refresh_context_from_output(&output, run_id)?;
                self.completed_tool_calls.insert(
                    call.call_id.clone(),
                    CachedToolExecution {
                        tool: call.tool,
                        input_fingerprint: fingerprint,
                        outcome: CachedToolOutcome::Success(output.clone()),
                    },
                );
                self.push_tool_result(call.call_id.clone(), call.tool, output);
                Ok(Some(AgentWorkflowEvent::ToolCompleted {
                    run_id,
                    session_id: self.workflow.session().id,
                    document_id: self.workflow.session().document_id.clone(),
                    call_id: call.call_id.clone(),
                    result,
                }))
            }
            Err(error) => Ok(Some(self.cache_tool_failure(call, fingerprint, run_id, error))),
        }
    }

    /// Registers the pending confirmation and builds its ConfirmationRequired event.
    fn require_confirmation(
        &mut self,
        call: &AgentToolCall,
        confirmation: db_pro_core::domain::agent_workflow::PendingAgentConfirmation,
        preview: Option<AgentToolOutput>,
    ) -> Result<AgentWorkflowEvent, AgentToolError> {
        self.pending_tool = Some(PendingToolCall {
            call_id: call.call_id.clone(),
            confirmation: confirmation.clone(),
        });
        Ok(AgentWorkflowEvent::ConfirmationRequired {
            run_id: self.run_id()?,
            session_id: self.workflow.session().id,
            document_id: self.workflow.session().document_id.clone(),
            call_id: call.call_id.clone(),
            kind: confirmation.kind,
            preview,
        })
    }

    fn request_for_call(&self, call: &AgentToolCall) -> Result<AgentToolRequest, AgentToolError> {
        let document = self
            .execution_context
            .document
            .as_ref()
            .ok_or(AgentToolError::DocumentNotFound)?;
        Ok(AgentToolRequest {
            session_id: self.workflow.session().id,
            run_id: self.run_id()?,
            document_id: document.document_id.clone(),
            document_version: document.document_version,
            tool: call.tool,
            input: call.input.clone(),
        })
    }

    fn current_document_version(&self) -> Result<u64, AgentToolError> {
        self.execution_context
            .document
            .as_ref()
            .map(|document| document.document_version)
            .ok_or(AgentToolError::DocumentNotFound)
    }

    fn push_tool_result(&mut self, call_id: String, tool: AgentTool, output: AgentToolOutput) {
        if let AgentToolOutput::QueryResult {
            result_count, summary, ..
        } = &output
        {
            self.execution_context.latest_result = Some(summary.clone());
            self.execution_context.result_count = *result_count;
        }
        self.provider_request.messages.push(AgentProviderMessage::ToolResult {
            call_id,
            tool,
            output: Ok(output),
        });
        if self.provider_request.messages.len() > db_pro_core::domain::agent::MAX_AGENT_HISTORY_MESSAGES {
            self.provider_request.messages.drain(
                0..(self.provider_request.messages.len() - db_pro_core::domain::agent::MAX_AGENT_HISTORY_MESSAGES),
            );
        }
    }

    fn refresh_context_from_output(
        &mut self,
        output: &AgentToolOutput,
        run_id: db_pro_core::domain::agent::AgentRunId,
    ) -> Result<(), AgentToolError> {
        let AgentToolOutput::CurrentQuery {
            document_id,
            document_version,
            sql,
            cursor_offset,
            selection,
            current_statement,
        } = output
        else {
            return Ok(());
        };
        if self
            .execution_context
            .document
            .as_ref()
            .map(|document| document.document_id.as_str())
            != Some(document_id)
        {
            return Err(AgentToolError::DocumentMismatch);
        }
        self.workflow.refresh_document_version(run_id, *document_version)?;
        if let Some(document) = self.execution_context.document.as_mut() {
            document.document_version = *document_version;
            document.sql = sql.clone();
            document.cursor_offset = *cursor_offset;
            document.selection = *selection;
            document.current_statement = current_statement.clone();
        }
        self.provider_request.context.document_version = *document_version;
        self.provider_request.context.current_sql = sql.clone();
        self.provider_request.context.selected_range = *selection;
        Ok(())
    }

    fn push_tool_error(&mut self, call_id: String, tool: AgentTool, error: AgentToolError) {
        self.provider_request.messages.push(AgentProviderMessage::ToolResult {
            call_id,
            tool,
            output: Err(error),
        });
        if self.provider_request.messages.len() > db_pro_core::domain::agent::MAX_AGENT_HISTORY_MESSAGES {
            self.provider_request.messages.drain(
                0..(self.provider_request.messages.len() - db_pro_core::domain::agent::MAX_AGENT_HISTORY_MESSAGES),
            );
        }
    }

    fn complete_run(&mut self, run_id: db_pro_core::domain::agent::AgentRunId) -> AgentWorkflowEvent {
        if let Err(error) = self.workflow.complete(run_id) {
            tracing::warn!(%error, "agent workflow completion state was already invalid");
        }
        AgentWorkflowEvent::Completed {
            run_id,
            session_id: self.workflow.session().id,
            document_id: self.workflow.session().document_id.clone(),
        }
    }

    fn fail_run(&mut self, run_id: db_pro_core::domain::agent::AgentRunId, message: String) -> AgentWorkflowEvent {
        if let Err(error) = self.workflow.fail(run_id) {
            tracing::warn!(%error, "agent workflow failure state was already invalid");
        }
        AgentWorkflowEvent::Failed {
            run_id,
            session_id: self.workflow.session().id,
            document_id: self.workflow.session().document_id.clone(),
            message,
        }
    }
}

impl std::fmt::Debug for AgentRunOrchestrator {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("AgentRunOrchestrator")
            .field("run_id", &self.workflow.active_run_id().ok())
            .field("tool_steps", &self.tool_steps)
            .field("pending_confirmation", &self.pending_tool.is_some())
            .finish_non_exhaustive()
    }
}

#[cfg(test)]
#[path = "tests/agent_orchestrator_tests.rs"]
mod tests;
