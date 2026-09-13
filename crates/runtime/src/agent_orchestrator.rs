use std::collections::HashSet;

use db_pro_core::domain::agent::{
    AgentTool, AgentToolCall, AgentToolInput, AgentToolOutput, AgentToolRequest, AgentToolResult, MAX_AGENT_TOOL_STEPS,
};
use db_pro_core::domain::agent_context::AgentContext;
use db_pro_core::domain::agent_workflow::{
    AgentConfirmationKind, AgentExecutionContext, AgentToolDisposition, AgentToolError, AgentWorkflow,
};

use crate::agent::{AgentProvider, AgentProviderEvent, AgentProviderMessage, AgentProviderRequest};
use crate::agent_executor::AgentToolRunner;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AgentWorkflowEvent {
    TextDelta {
        run_id: db_pro_core::domain::agent::AgentRunId,
        delta: String,
    },
    ToolRequested {
        run_id: db_pro_core::domain::agent::AgentRunId,
        call: AgentToolCall,
    },
    ToolCompleted {
        run_id: db_pro_core::domain::agent::AgentRunId,
        call_id: String,
        result: AgentToolResult,
    },
    ToolFailed {
        run_id: db_pro_core::domain::agent::AgentRunId,
        call_id: String,
        tool: AgentTool,
        error: AgentToolError,
    },
    ConfirmationRequired {
        run_id: db_pro_core::domain::agent::AgentRunId,
        call_id: String,
        kind: AgentConfirmationKind,
        preview: Option<AgentToolOutput>,
    },
    Completed {
        run_id: db_pro_core::domain::agent::AgentRunId,
    },
    Cancelled {
        run_id: db_pro_core::domain::agent::AgentRunId,
    },
    Failed {
        run_id: db_pro_core::domain::agent::AgentRunId,
        message: String,
    },
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
        Ok(AgentWorkflowEvent::Cancelled { run_id })
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
        if let Some(document) = current_document {
            self.execution_context.document = Some(document);
        }
        let run_id = self.run_id()?;
        let current_version = self.current_document_version()?;
        if !approved {
            self.workflow.reject(run_id)?;
            self.push_tool_error(
                pending_call_id.clone(),
                pending_tool,
                AgentToolError::ConfirmationRejected {
                    kind: pending.confirmation.kind,
                },
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
                    self.push_tool_error(pending_call_id.clone(), tool, error.clone());
                    let mut emitted = vec![AgentWorkflowEvent::ToolFailed {
                        run_id,
                        call_id: pending_call_id,
                        tool,
                        error,
                    }];
                    emitted.extend(self.drive().await);
                    return Ok(emitted);
                }
            }
        };
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
                    AgentProviderEvent::TextDelta { delta } => {
                        emitted.push(AgentWorkflowEvent::TextDelta { run_id, delta })
                    }
                    AgentProviderEvent::ToolCall(call) => {
                        handled_tool = true;
                        emitted.push(AgentWorkflowEvent::ToolRequested {
                            run_id,
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
        if !self.seen_call_ids.insert(call.call_id.clone()) {
            return Err(AgentToolError::InvalidInput { tool: call.tool });
        }
        self.tool_steps += 1;
        if self.tool_steps > MAX_AGENT_TOOL_STEPS {
            return Err(AgentToolError::InvalidInput { tool: call.tool });
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
                self.push_tool_error(call.call_id.clone(), call.tool, error.clone());
                return Ok(Some(AgentWorkflowEvent::ToolFailed {
                    run_id,
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
                        self.push_tool_result(call.call_id.clone(), call.tool, output);
                        Ok(Some(AgentWorkflowEvent::ToolCompleted {
                            run_id,
                            call_id: call.call_id,
                            result,
                        }))
                    }
                    Err(error) => {
                        self.push_tool_error(call.call_id.clone(), call.tool, error.clone());
                        Ok(Some(AgentWorkflowEvent::ToolFailed {
                            run_id,
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
    }

    fn complete_run(&mut self, run_id: db_pro_core::domain::agent::AgentRunId) -> AgentWorkflowEvent {
        if let Err(error) = self.workflow.complete(run_id) {
            tracing::warn!(%error, "agent workflow completion state was already invalid");
        }
        AgentWorkflowEvent::Completed { run_id }
    }

    fn fail_run(&mut self, run_id: db_pro_core::domain::agent::AgentRunId, message: String) -> AgentWorkflowEvent {
        if let Err(error) = self.workflow.fail(run_id) {
            tracing::warn!(%error, "agent workflow failure state was already invalid");
        }
        AgentWorkflowEvent::Failed { run_id, message }
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
    use db_pro_core::domain::agent::{AgentDocumentSnapshot, AgentMode, AgentSession, AgentToolInput};
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
}
