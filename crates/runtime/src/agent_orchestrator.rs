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

    async fn handle_tool_call(&mut self, call: AgentToolCall) -> Result<Option<AgentWorkflowEvent>, AgentToolError> {
        let fingerprint = call.input.fingerprint();
        if let Some(pending) = &self.pending_tool {
            if pending.call_id == call.call_id {
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
                return Ok(Some(AgentWorkflowEvent::ConfirmationRequired {
                    run_id: self.run_id()?,
                    session_id: self.workflow.session().id,
                    document_id: self.workflow.session().document_id.clone(),
                    call_id: call.call_id,
                    kind: pending.confirmation.kind,
                    preview,
                }));
            }
        }

        if let Some(cached) = self.completed_tool_calls.get(&call.call_id) {
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
                    return Ok(Some(AgentWorkflowEvent::ToolCompleted {
                        run_id,
                        session_id: self.workflow.session().id,
                        document_id: self.workflow.session().document_id.clone(),
                        call_id: call.call_id,
                        result,
                    }));
                }
                CachedToolOutcome::Failed(error) => {
                    return Ok(Some(AgentWorkflowEvent::ToolFailed {
                        run_id,
                        session_id: self.workflow.session().id,
                        document_id: self.workflow.session().document_id.clone(),
                        call_id: call.call_id,
                        tool: call.tool,
                        error: error.clone(),
                    }));
                }
                CachedToolOutcome::Rejected(kind) => {
                    return Ok(Some(AgentWorkflowEvent::ToolFailed {
                        run_id,
                        session_id: self.workflow.session().id,
                        document_id: self.workflow.session().document_id.clone(),
                        call_id: call.call_id,
                        tool: call.tool,
                        error: AgentToolError::ConfirmationRejected { kind: *kind },
                    }));
                }
            }
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
                self.completed_tool_calls.insert(
                    call.call_id.clone(),
                    CachedToolExecution {
                        tool: call.tool,
                        input_fingerprint: fingerprint,
                        outcome: CachedToolOutcome::Failed(error.clone()),
                    },
                );
                self.push_tool_error(call.call_id.clone(), call.tool, error.clone());
                return Ok(Some(AgentWorkflowEvent::ToolFailed {
                    run_id,
                    session_id: self.workflow.session().id,
                    document_id: self.workflow.session().document_id.clone(),
                    call_id: call.call_id,
                    tool: call.tool,
                    error,
                }));
            }
        };
        match disposition {
            AgentToolDisposition::Execute(request) => {
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
                            call_id: call.call_id,
                            result,
                        }))
                    }
                    Err(error) => {
                        self.completed_tool_calls.insert(
                            call.call_id.clone(),
                            CachedToolExecution {
                                tool: call.tool,
                                input_fingerprint: fingerprint,
                                outcome: CachedToolOutcome::Failed(error.clone()),
                            },
                        );
                        self.push_tool_error(call.call_id.clone(), call.tool, error.clone());
                        Ok(Some(AgentWorkflowEvent::ToolFailed {
                            run_id,
                            session_id: self.workflow.session().id,
                            document_id: self.workflow.session().document_id.clone(),
                            call_id: call.call_id,
                            tool: call.tool,
                            error,
                        }))
                    }
                }
            }
            AgentToolDisposition::PatchPreview { preview, confirmation } => {
                self.pending_tool = Some(PendingToolCall {
                    call_id: call.call_id.clone(),
                    confirmation: confirmation.clone(),
                });
                Ok(Some(AgentWorkflowEvent::ConfirmationRequired {
                    run_id,
                    session_id: self.workflow.session().id,
                    document_id: self.workflow.session().document_id.clone(),
                    call_id: call.call_id,
                    kind: confirmation.kind,
                    preview: Some(preview),
                }))
            }
            AgentToolDisposition::ConfirmationRequired(confirmation) => {
                self.pending_tool = Some(PendingToolCall {
                    call_id: call.call_id.clone(),
                    confirmation: confirmation.clone(),
                });
                Ok(Some(AgentWorkflowEvent::ConfirmationRequired {
                    run_id,
                    session_id: self.workflow.session().id,
                    document_id: self.workflow.session().document_id.clone(),
                    call_id: call.call_id,
                    kind: confirmation.kind,
                    preview: None,
                }))
            }
        }
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
mod tests {
    use std::collections::VecDeque;
    use std::sync::Mutex;

    use async_trait::async_trait;
    use db_pro_core::domain::agent::{AgentDocumentSnapshot, AgentMode, AgentSession, AgentToolInput, AgentToolResult};
    use db_pro_core::domain::agent_context::AgentContext;

    use super::*;

    struct FakeProvider {
        responses: Mutex<VecDeque<Vec<AgentProviderEvent>>>,
    }

    #[async_trait]
    impl AgentProvider for FakeProvider {
        async fn complete(
            &self,
            _request: AgentProviderRequest,
        ) -> Result<Vec<AgentProviderEvent>, crate::CodexProviderError> {
            self.responses
                .lock()
                .expect("provider lock")
                .pop_front()
                .ok_or(crate::CodexProviderError::EmptyResponse)
        }
    }

    struct FakeToolRunner;

    #[async_trait]
    impl AgentToolRunner for FakeToolRunner {
        async fn execute(
            &self,
            request: &AgentToolRequest,
            context: &AgentExecutionContext,
        ) -> Result<AgentToolResult, AgentToolError> {
            let document = context.document.as_ref().ok_or(AgentToolError::DocumentNotFound)?;
            match request.tool {
                AgentTool::GetCurrentQuery => Ok(AgentToolResult {
                    tool: request.tool,
                    output: AgentToolOutput::CurrentQuery {
                        document_id: document.document_id.clone(),
                        document_version: document.document_version,
                        sql: document.sql.clone(),
                        cursor_offset: document.cursor_offset,
                        selection: document.selection,
                        current_statement: document.current_statement.clone(),
                    },
                }),
                _ => Err(AgentToolError::InvalidInput { tool: request.tool }),
            }
        }
    }

    fn document(version: u64) -> AgentDocumentSnapshot {
        AgentDocumentSnapshot {
            document_id: "doc-a".to_owned(),
            document_version: version,
            sql: "SELECT 1".to_owned(),
            cursor_offset: 8,
            selection: None,
            current_statement: Some("SELECT 1".to_owned()),
        }
    }

    fn context(version: u64) -> AgentContext {
        AgentContext {
            document_id: "doc-a".to_owned(),
            document_version: version,
            connection_id: None,
            schema: Some("public".to_owned()),
            current_sql: "SELECT 1".to_owned(),
            user_request: "inspect the query".to_owned(),
            selected_range: None,
            referenced_tables: Vec::new(),
            foreign_keys: Vec::new(),
            diagnostics: Vec::new(),
            result_summary: None,
        }
    }

    fn orchestrator_with_mode(responses: Vec<Vec<AgentProviderEvent>>, mode: AgentMode) -> AgentRunOrchestrator {
        let session = AgentSession::new("doc-a", None, Some("public".to_owned()));
        let execution_context = AgentExecutionContext::new(session.clone(), document(1), mode);
        AgentRunOrchestrator::new(
            Box::new(FakeProvider {
                responses: Mutex::new(responses.into_iter().collect()),
            }),
            Box::new(FakeToolRunner),
            AgentWorkflow::new(session, mode, false),
            execution_context,
            "inspect".to_owned(),
            context(1),
        )
        .expect("orchestrator should start")
    }

    fn orchestrator(responses: Vec<Vec<AgentProviderEvent>>) -> AgentRunOrchestrator {
        orchestrator_with_mode(responses, AgentMode::Ask)
    }

    #[tokio::test]
    async fn routes_typed_tool_result_into_provider_continuation() {
        let mut orchestrator = orchestrator(vec![
            vec![
                AgentProviderEvent::ToolCall(AgentToolCall {
                    call_id: "call-1".to_owned(),
                    tool: AgentTool::GetCurrentQuery,
                    input: AgentToolInput::None,
                }),
                AgentProviderEvent::Completed,
            ],
            vec![
                AgentProviderEvent::TextDelta {
                    delta: "The query is read-only.".to_owned(),
                },
                AgentProviderEvent::Completed,
            ],
        ]);

        let events = orchestrator.run().await;
        assert!(events
            .iter()
            .any(|event| matches!(event, AgentWorkflowEvent::ToolCompleted { call_id, .. } if call_id == "call-1")));
        assert!(events
            .iter()
            .any(|event| matches!(event, AgentWorkflowEvent::TextDelta { delta, .. } if delta.contains("read-only"))));
        assert!(events
            .iter()
            .any(|event| matches!(event, AgentWorkflowEvent::Completed { .. })));
        assert!(!orchestrator.has_pending_confirmation());
    }

    #[tokio::test]
    async fn patch_confirmation_updates_run_version_before_continuation() {
        let mut orchestrator = orchestrator_with_mode(
            vec![
                vec![AgentProviderEvent::ToolCall(AgentToolCall {
                    call_id: "patch-1".to_owned(),
                    tool: AgentTool::PatchQuery,
                    input: AgentToolInput::Patch {
                        patch: db_pro_core::domain::agent::AgentSqlPatch {
                            document_id: "doc-a".to_owned(),
                            expected_version: 1,
                            range: (0, 8),
                            replacement: "SELECT 2".to_owned(),
                        },
                    },
                })],
                vec![AgentProviderEvent::Completed],
            ],
            AgentMode::Edit,
        );

        let first = orchestrator.run().await;
        assert!(first.iter().any(|event| matches!(
            event,
            AgentWorkflowEvent::ConfirmationRequired {
                kind: AgentConfirmationKind::ApplyPatch,
                ..
            }
        )));
        assert!(orchestrator.has_pending_confirmation());

        let second = orchestrator
            .resume_confirmation(
                true,
                Some(document(1)),
                Some(AgentToolOutput::PatchApplied {
                    document_id: "doc-a".to_owned(),
                    new_version: 2,
                    range: (0, 8),
                }),
            )
            .await
            .expect("patch confirmation should continue");
        assert!(second
            .iter()
            .any(|event| matches!(event, AgentWorkflowEvent::Completed { .. })));
    }

    #[tokio::test]
    async fn stale_document_can_recover_through_current_query() {
        let mut orchestrator = orchestrator(vec![
            vec![AgentProviderEvent::ToolCall(AgentToolCall {
                call_id: "current-1".to_owned(),
                tool: AgentTool::GetCurrentQuery,
                input: AgentToolInput::None,
            })],
            vec![AgentProviderEvent::Completed],
        ]);
        orchestrator.update_document_snapshot(AgentDocumentSnapshot {
            sql: "SELECT 2".to_owned(),
            document_version: 2,
            ..document(1)
        });

        let events = orchestrator.run().await;
        assert!(events
            .iter()
            .any(|event| matches!(event, AgentWorkflowEvent::ToolCompleted { .. })));
        assert_eq!(
            orchestrator
                .workflow()
                .session()
                .active_run
                .as_ref()
                .map(|run| run.document_version),
            None
        );
    }

    #[tokio::test]
    async fn duplicate_tool_call_replays_cached_output_without_re_execution() {
        let mut orchestrator = orchestrator(vec![
            vec![AgentProviderEvent::ToolCall(AgentToolCall {
                call_id: "dedup-1".to_owned(),
                tool: AgentTool::GetCurrentQuery,
                input: AgentToolInput::None,
            })],
            vec![AgentProviderEvent::ToolCall(AgentToolCall {
                call_id: "dedup-1".to_owned(),
                tool: AgentTool::GetCurrentQuery,
                input: AgentToolInput::None,
            })],
            vec![AgentProviderEvent::Completed],
        ]);

        let events = orchestrator.run().await;
        let completed_count = events
            .iter()
            .filter(|event| matches!(event, AgentWorkflowEvent::ToolCompleted { call_id, .. } if call_id == "dedup-1"))
            .count();
        assert_eq!(completed_count, 2);
    }

    #[tokio::test]
    async fn duplicate_pending_confirmation_reuses_pending_state() {
        let patch = db_pro_core::domain::agent::AgentSqlPatch {
            document_id: "doc-a".to_owned(),
            expected_version: 1,
            range: (0, 8),
            replacement: "SELECT 2".to_owned(),
        };
        let mut orchestrator = orchestrator_with_mode(
            vec![
                vec![AgentProviderEvent::ToolCall(AgentToolCall {
                    call_id: "patch-dup".to_owned(),
                    tool: AgentTool::PatchQuery,
                    input: AgentToolInput::Patch { patch: patch.clone() },
                })],
                vec![AgentProviderEvent::ToolCall(AgentToolCall {
                    call_id: "patch-dup".to_owned(),
                    tool: AgentTool::PatchQuery,
                    input: AgentToolInput::Patch { patch },
                })],
            ],
            AgentMode::Edit,
        );

        let events = orchestrator.run().await;
        let confirmations = events
            .iter()
            .filter(|event| matches!(event, AgentWorkflowEvent::ConfirmationRequired { call_id, .. } if call_id == "patch-dup"))
            .count();
        assert!(confirmations >= 1);
        assert!(orchestrator.has_pending_confirmation());
    }

    #[tokio::test]
    async fn tool_call_collision_with_different_input_fails_with_protocol_error() {
        let mut orchestrator = orchestrator(vec![
            vec![AgentProviderEvent::ToolCall(AgentToolCall {
                call_id: "coll-1".to_owned(),
                tool: AgentTool::GetCurrentQuery,
                input: AgentToolInput::None,
            })],
            vec![AgentProviderEvent::ToolCall(AgentToolCall {
                call_id: "coll-1".to_owned(),
                tool: AgentTool::InspectSchema,
                input: AgentToolInput::Schema {
                    schema: Some("public".to_owned()),
                },
            })],
            vec![AgentProviderEvent::Completed],
        ]);

        let events = orchestrator.run().await;
        assert!(events
            .iter()
            .any(|event| matches!(event, AgentWorkflowEvent::Failed { message, .. } if message.contains("collision"))));
    }

    struct CountingToolRunner {
        count: std::sync::atomic::AtomicUsize,
    }

    #[async_trait::async_trait]
    impl AgentToolRunner for CountingToolRunner {
        async fn execute(
            &self,
            request: &AgentToolRequest,
            _context: &AgentExecutionContext,
        ) -> Result<AgentToolResult, AgentToolError> {
            self.count.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
            match request.tool {
                AgentTool::RunQuery => Ok(AgentToolResult {
                    tool: request.tool,
                    output: AgentToolOutput::QueryResult {
                        statement_index: Some(0),
                        result_count: 1,
                        summary: db_pro_core::domain::agent_context::AgentResultSummary {
                            columns: vec![],
                            sample_rows: vec![],
                            row_count: Some(1),
                            affected_rows: Some(1),
                            truncated: false,
                        },
                    },
                }),
                _ => Err(AgentToolError::InvalidInput { tool: request.tool }),
            }
        }
    }

    #[tokio::test]
    async fn mutation_run_query_executes_once_and_repeats_replay_without_database_re_execution() {
        let runner = std::sync::Arc::new(CountingToolRunner {
            count: std::sync::atomic::AtomicUsize::new(0),
        });
        struct ArcRunner(std::sync::Arc<CountingToolRunner>);
        #[async_trait::async_trait]
        impl AgentToolRunner for ArcRunner {
            async fn execute(
                &self,
                request: &AgentToolRequest,
                context: &AgentExecutionContext,
            ) -> Result<AgentToolResult, AgentToolError> {
                self.0.execute(request, context).await
            }
        }

        let session = AgentSession::new("doc-a", Some("conn-1".to_owned()), Some("public".to_owned()));
        let execution_context = AgentExecutionContext::new(session.clone(), document(1), AgentMode::Agent);
        let mut orchestrator = AgentRunOrchestrator::new(
            Box::new(FakeProvider {
                responses: Mutex::new(
                    vec![
                        vec![AgentProviderEvent::ToolCall(AgentToolCall {
                            call_id: "mut-1".to_owned(),
                            tool: AgentTool::RunQuery,
                            input: AgentToolInput::Query {
                                sql: "UPDATE users SET active = true".to_owned(),
                            },
                        })],
                        vec![AgentProviderEvent::ToolCall(AgentToolCall {
                            call_id: "mut-1".to_owned(),
                            tool: AgentTool::RunQuery,
                            input: AgentToolInput::Query {
                                sql: "UPDATE users SET active = true".to_owned(),
                            },
                        })],
                        vec![AgentProviderEvent::Completed],
                    ]
                    .into_iter()
                    .collect(),
                ),
            }),
            Box::new(ArcRunner(runner.clone())),
            AgentWorkflow::new(session, AgentMode::Agent, false),
            execution_context,
            "update".to_owned(),
            context(1),
        )
        .expect("orchestrator should start");

        let first = orchestrator.run().await;
        assert!(first.iter().any(|e| matches!(
            e,
            AgentWorkflowEvent::ConfirmationRequired {
                kind: AgentConfirmationKind::RunMutation,
                ..
            }
        )));

        let second = orchestrator
            .resume_confirmation(true, Some(document(1)), None)
            .await
            .expect("confirmation approved");
        assert!(second.iter().any(|e| matches!(e, AgentWorkflowEvent::Completed { .. })));

        // Verification: Even though provider called tool "mut-1" twice, the database runner executed exactly ONCE!
        assert_eq!(runner.count.load(std::sync::atomic::Ordering::SeqCst), 1);
    }

    #[tokio::test]
    async fn rejected_confirmation_is_cached_and_replayed_on_repeated_call_id() {
        let runner = std::sync::Arc::new(CountingToolRunner {
            count: std::sync::atomic::AtomicUsize::new(0),
        });
        struct ArcRunner(std::sync::Arc<CountingToolRunner>);
        #[async_trait::async_trait]
        impl AgentToolRunner for ArcRunner {
            async fn execute(
                &self,
                request: &AgentToolRequest,
                context: &AgentExecutionContext,
            ) -> Result<AgentToolResult, AgentToolError> {
                self.0.execute(request, context).await
            }
        }

        let session = AgentSession::new("doc-a", Some("conn-1".to_owned()), Some("public".to_owned()));
        let execution_context = AgentExecutionContext::new(session.clone(), document(1), AgentMode::Agent);
        let mut orchestrator = AgentRunOrchestrator::new(
            Box::new(FakeProvider {
                responses: Mutex::new(
                    vec![
                        vec![AgentProviderEvent::ToolCall(AgentToolCall {
                            call_id: "mut-rej".to_owned(),
                            tool: AgentTool::RunQuery,
                            input: AgentToolInput::Query {
                                sql: "DELETE FROM users".to_owned(),
                            },
                        })],
                        vec![AgentProviderEvent::ToolCall(AgentToolCall {
                            call_id: "mut-rej".to_owned(),
                            tool: AgentTool::RunQuery,
                            input: AgentToolInput::Query {
                                sql: "DELETE FROM users".to_owned(),
                            },
                        })],
                        vec![AgentProviderEvent::Completed],
                    ]
                    .into_iter()
                    .collect(),
                ),
            }),
            Box::new(ArcRunner(runner.clone())),
            AgentWorkflow::new(session, AgentMode::Agent, false),
            execution_context,
            "delete".to_owned(),
            context(1),
        )
        .expect("orchestrator should start");

        let first = orchestrator.run().await;
        assert!(first.iter().any(|e| matches!(
            e,
            AgentWorkflowEvent::ConfirmationRequired {
                kind: AgentConfirmationKind::RunDestructive,
                ..
            }
        )));

        // User rejects confirmation
        let second = orchestrator
            .resume_confirmation(false, Some(document(1)), None)
            .await
            .expect("confirmation rejected handled");
        assert!(second.iter().any(|e| matches!(e, AgentWorkflowEvent::Completed { .. })));

        // Verification: Database was never touched, count is 0!
        assert_eq!(runner.count.load(std::sync::atomic::Ordering::SeqCst), 0);
    }

    #[tokio::test]
    async fn safety_escalation_requires_destructive_confirmation_for_dangerous_mutation() {
        let runner = std::sync::Arc::new(CountingToolRunner {
            count: std::sync::atomic::AtomicUsize::new(0),
        });
        struct ArcRunner(std::sync::Arc<CountingToolRunner>);
        #[async_trait::async_trait]
        impl AgentToolRunner for ArcRunner {
            async fn execute(
                &self,
                request: &AgentToolRequest,
                context: &AgentExecutionContext,
            ) -> Result<AgentToolResult, AgentToolError> {
                self.0.execute(request, context).await
            }
        }

        let session = AgentSession::new("doc-a", Some("conn-1".to_owned()), Some("public".to_owned()));
        let execution_context = AgentExecutionContext::new(session.clone(), document(1), AgentMode::Agent);
        let mut orchestrator = AgentRunOrchestrator::new(
            Box::new(FakeProvider {
                responses: Mutex::new(
                    vec![
                        vec![AgentProviderEvent::ToolCall(AgentToolCall {
                            call_id: "drop-table-1".to_owned(),
                            tool: AgentTool::RunQuery,
                            input: AgentToolInput::Query {
                                sql: "DROP TABLE users CASCADE".to_owned(),
                            },
                        })],
                        vec![AgentProviderEvent::Completed],
                    ]
                    .into_iter()
                    .collect(),
                ),
            }),
            Box::new(ArcRunner(runner.clone())),
            AgentWorkflow::new(session, AgentMode::Agent, false),
            execution_context,
            "drop table".to_owned(),
            context(1),
        )
        .expect("orchestrator should start");

        let events = orchestrator.run().await;
        // Verify it demands RunDestructive, NOT RunMutation or RunUnknown
        assert!(events.iter().any(|e| matches!(
            e,
            AgentWorkflowEvent::ConfirmationRequired {
                kind: AgentConfirmationKind::RunDestructive,
                call_id,
                ..
            } if call_id == "drop-table-1"
        )));

        // Reject confirmation and verify database count remains 0
        let finish_events = orchestrator
            .resume_confirmation(false, Some(document(1)), None)
            .await
            .expect("resume rejected");
        assert!(finish_events
            .iter()
            .any(|e| matches!(e, AgentWorkflowEvent::Completed { .. })));
        assert_eq!(runner.count.load(std::sync::atomic::Ordering::SeqCst), 0);
    }

    #[tokio::test]
    async fn malformed_provider_missing_arguments_reports_tool_failed_gracefully() {
        let mut orchestrator = orchestrator_with_mode(
            vec![
                vec![AgentProviderEvent::ToolCall(AgentToolCall {
                    call_id: "bad-1".to_owned(),
                    tool: AgentTool::RunQuery,
                    input: AgentToolInput::None,
                })],
                vec![AgentProviderEvent::Completed],
            ],
            AgentMode::Agent,
        );

        let events = orchestrator.run().await;
        assert!(events.iter().any(|e| matches!(
            e,
            AgentWorkflowEvent::ToolFailed {
                call_id,
                error: AgentToolError::InvalidInput { .. },
                ..
            } if call_id == "bad-1"
        )));
        assert!(events.iter().any(|e| matches!(e, AgentWorkflowEvent::Completed { .. })));
    }
}
