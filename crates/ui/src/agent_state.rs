use super::*;

use db_pro_core::domain::agent::{AgentDocumentSnapshot, AgentObjectRef};
use db_pro_core::domain::agent_context::{
    AgentColumnContext, AgentContext as CoreAgentContext, AgentContextBuilder, AgentContextRequest,
    AgentDiagnosticContext, AgentForeignKeyContext, AgentSchemaCatalog, AgentTableContext,
};

/// Owns agent workspace state independently from the shell and query session.
#[derive(Debug)]
pub(crate) struct AgentState {
    pub(crate) pending_prompt: Option<String>,
    pub(crate) pending_context: Option<AgentContext>,
    pub(crate) provider_label: String,
    pub(crate) provider_detail: String,
    pub(crate) input: String,
    pub(crate) messages: Vec<AgentMessage>,
    pub(crate) sessions: HashMap<String, AgentUiSession>,
    pub(crate) auto_run_read_only: bool,
    pub(crate) settings_open: bool,
    pub(crate) api_key_draft: String,
    pub(crate) api_key_show_password: bool,
    pub(crate) configure_request: Option<crate::RequestId>,
}

impl Default for AgentState {
    fn default() -> Self {
        let provider_info = OfflineAgentProvider.info();
        Self {
            pending_prompt: None,
            pending_context: None,
            provider_label: provider_info.label.to_owned(),
            provider_detail: provider_info.detail.to_owned(),
            input: String::new(),
            messages: Vec::new(),
            sessions: HashMap::new(),
            auto_run_read_only: false,
            settings_open: false,
            api_key_draft: String::new(),
            api_key_show_password: false,
            configure_request: None,
        }
    }
}

impl DbProApp {
    pub(super) fn reset_agent_context(&mut self) {
        self.agent.pending_prompt = None;
        self.agent.pending_context = None;
        self.agent.input.clear();
        self.agent.messages.clear();
    }

    pub(super) fn agent_context(&self) -> AgentContext {
        let connection_name = Some(self.active_connection_name().to_owned());
        let driver = self.active_driver().to_owned();
        let selected_columns = self
            .table_state
            .table_info
            .as_ref()
            .map(|info| info.columns.iter().map(|column| column.name.clone()).collect())
            .unwrap_or_default();
        let current_sql = if self.query_session_state.selected_text.trim().is_empty() {
            self.active_query_text().to_owned()
        } else {
            self.query_session_state.selected_text.clone()
        };
        let result_summary = self
            .active_query_result()
            .or(self.table_state.table_data_result.as_ref())
            .map(|result| format!("{} rows returned in {} ms", result.row_count, result.duration_ms));
        let last_error = self.has_runtime_error().then(|| self.runtime_message.clone());

        AgentContext {
            connection_name,
            driver,
            tables: self.active_schema_table_names(),
            columns: self.active_schema_column_names(),
            schema: Some(self.active_schema().to_owned()),
            selected_table: self.selected_table.clone(),
            selected_columns,
            current_sql,
            result_summary,
            explain_plan: self.active_explain_plan().map(|p| p.to_owned()),
            last_error,
            workspace_files: self.workspace_context_items.clone(),
        }
    }

    pub(super) fn submit_agent_prompt(&mut self) {
        if !self.workspace_context_items.is_empty() && !self.ide_workspace.is_trusted() {
            self.runtime_message = "Trust the workspace before sending folder/file context to Agent".to_owned();
            return;
        }
        let prompt = self.agent.input.trim().to_owned();
        if prompt.is_empty() {
            return;
        }
        self.submit_typed_agent_prompt(prompt);
    }

    fn submit_typed_agent_prompt(&mut self, prompt: String) {
        let Some(document) = self
            .query_session_state
            .documents
            .get(self.query_session_state.active_document_index)
        else {
            self.runtime_message = "No query document is available for Agent".to_owned();
            return;
        };
        if self.agent.provider_label == "Offline draft" {
            self.runtime_message = "AI provider is not configured. Enter an API key in Agent Settings.".to_owned();
            self.show_toast_error("Configure an API key in Agent Settings to start.");
            return;
        }
        let document_id = document.id.clone();
        let connection_id = document
            .connection_id
            .clone()
            .or_else(|| self.connection_lifecycle.active_connection_id.clone());
        let schema = document
            .schema
            .clone()
            .or_else(|| Some(self.active_schema().to_owned()));
        let snapshot = self.agent_document_snapshot(document);
        let context = self.build_agent_context(&prompt, document, connection_id.as_deref(), schema.as_deref());

        let session = self
            .agent
            .sessions
            .entry(document_id.clone())
            .or_insert_with(|| AgentUiSession::for_document(&document_id, connection_id.clone(), schema.clone()));
        if session.session.is_none() {
            *session = AgentUiSession::for_document(&document_id, connection_id.clone(), schema.clone());
        }
        if session.active_run_id.is_some() || session.request_id.is_some() {
            self.runtime_message = "An Agent run is already active for this query".to_owned();
            return;
        }
        let Some(mut core_session) = session.session.clone() else {
            session.state = db_pro_core::domain::agent::AgentSessionState::Failed;
            self.runtime_message = "Agent session could not be initialized".to_owned();
            return;
        };
        core_session.connection_id = connection_id;
        core_session.schema = schema;
        session.session = Some(core_session.clone());
        let mode = session.mode;
        session.messages.push(AgentMessage {
            role: AgentRole::User,
            content: prompt.clone(),
            sql: None,
            requires_confirmation: false,
        });
        session.state = db_pro_core::domain::agent::AgentSessionState::Running;
        session.streaming_text.clear();
        session.activities.clear();
        session.pending_confirmation = None;
        session.tool_results.clear();

        let request_id = self.task_bridge.next_request_id();
        session.request_id = Some(request_id);
        self.agent.input.clear();
        self.runtime_message = format!("Sending request to {}…", self.agent.provider_label);
        if self
            .task_bridge
            .send(UiCommand::StartAgentRun {
                request_id,
                prompt,
                session: core_session,
                document: snapshot,
                mode,
                allow_read_only_auto_run: self.agent.auto_run_read_only,
                context,
            })
            .is_err()
        {
            session.request_id = None;
            session.state = db_pro_core::domain::agent::AgentSessionState::Failed;
            self.runtime_message = "Agent runtime unavailable".to_owned();
        }
    }

    fn agent_document_snapshot(&self, document: &crate::query::QueryDocument) -> AgentDocumentSnapshot {
        let selection = (!document.selection.is_empty()).then(|| document.selection.normalized());
        AgentDocumentSnapshot {
            document_id: document.id.clone(),
            document_version: document.buffer.version(),
            sql: document.text().to_owned(),
            cursor_offset: document.cursor.offset,
            selection,
            current_statement: document
                .analysis
                .current_statement_at(document.cursor.offset)
                .map(|statement| statement.text.clone()),
        }
    }

    fn build_agent_context(
        &self,
        prompt: &str,
        document: &crate::query::QueryDocument,
        connection_id: Option<&str>,
        schema: Option<&str>,
    ) -> CoreAgentContext {
        let tables = self
            .schema
            .table_details
            .iter()
            .map(|table| AgentTableContext {
                object: AgentObjectRef {
                    schema: Some(table.schema.clone()),
                    name: table.name.clone(),
                },
                columns: table
                    .columns
                    .iter()
                    .enumerate()
                    .map(|(ordinal, column)| AgentColumnContext {
                        name: column.name.clone(),
                        data_type: column.data_type.clone(),
                        nullable: column.nullable,
                        ordinal,
                        default: None,
                        is_primary_key: column.is_primary_key,
                        is_unique: false,
                        is_identity: false,
                        is_generated: false,
                    })
                    .collect(),
            })
            .collect::<Vec<_>>();
        let foreign_keys = self
            .schema
            .table_details
            .iter()
            .flat_map(|table| {
                table.foreign_keys.iter().map(|foreign_key| AgentForeignKeyContext {
                    name: foreign_key.name.clone(),
                    source: AgentObjectRef {
                        schema: Some(table.schema.clone()),
                        name: table.name.clone(),
                    },
                    source_columns: foreign_key.from_columns.clone(),
                    target: AgentObjectRef {
                        schema: Some(foreign_key.to_schema.clone()),
                        name: foreign_key.to_table.clone(),
                    },
                    target_columns: foreign_key.to_columns.clone(),
                })
            })
            .collect::<Vec<_>>();
        let catalog = AgentSchemaCatalog { tables, foreign_keys };
        let diagnostics = document
            .diagnostics
            .iter()
            .map(|diagnostic| AgentDiagnosticContext {
                message: diagnostic.message.clone(),
                range: Some(diagnostic.range),
            })
            .collect::<Vec<_>>();
        AgentContextBuilder::default().build(&AgentContextRequest {
            document_id: &document.id,
            document_version: document.buffer.version(),
            connection_id,
            schema,
            current_sql: document.text(),
            selected_range: (!document.selection.is_empty()).then(|| document.selection.normalized()),
            user_request: prompt,
            diagnostics: &diagnostics,
            result_summary: None,
            catalog: &catalog,
        })
    }

    pub(super) fn on_agent_workflow_event(&mut self, event: db_pro_core::domain::agent_workflow::AgentWorkflowEvent) {
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
        let Some(session) = self.agent.sessions.get_mut(&document_id) else {
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

        apply_agent_workflow_event(session, event, run_id);
    }

    pub(super) fn agent_confirmation_action(&mut self, approved: bool) {
        let Some(document_id) = self
            .query_session_state
            .documents
            .get(self.query_session_state.active_document_index)
            .map(|document| document.id.clone())
        else {
            return;
        };
        let Some(pending) = self
            .agent
            .sessions
            .get(&document_id)
            .and_then(|session| session.pending_confirmation.clone())
        else {
            return;
        };
        let target_doc_index = self
            .query_session_state
            .documents
            .iter()
            .position(|doc| doc.id == pending.document_id)
            .unwrap_or(self.query_session_state.active_document_index);
        let current_document = self
            .query_session_state
            .documents
            .get(target_doc_index)
            .map(|document| self.agent_document_snapshot(document));
        let mut applied_patch = None;
        if approved && pending.kind == db_pro_core::domain::agent_workflow::AgentConfirmationKind::ApplyPatch {
            let Some(db_pro_core::domain::agent::AgentToolOutput::PatchPreview { patch, .. }) = pending.preview else {
                self.runtime_message = "Agent patch preview is unavailable".to_owned();
                return;
            };
            let Some(document) = self.query_session_state.documents.get_mut(target_doc_index) else {
                return;
            };
            if document.id != patch.document_id || document.buffer.version() != patch.expected_version {
                self.runtime_message = "This query changed since the suggestion was created.".to_owned();
                self.show_toast_error("The query changed since the suggestion was created.");
                self.agent_confirmation_action(false);
                return;
            }
            let Some(current) = current_document.clone() else {
                return;
            };
            if patch
                .apply_to(&current.document_id, current.document_version, &current.sql)
                .is_err()
            {
                self.runtime_message = "Agent patch range is no longer valid".to_owned();
                self.agent_confirmation_action(false);
                return;
            }
            let (start, end) = patch.range;
            let before = crate::editor::buffer::EditorSnapshot {
                cursor_offset: document.cursor.offset,
                anchor_offset: document.selection.anchor,
            };
            let new_cursor = start + patch.replacement.len();
            let after = crate::editor::buffer::EditorSnapshot {
                cursor_offset: new_cursor,
                anchor_offset: new_cursor,
            };
            document
                .buffer
                .replace_with_snapshot(start, end, &patch.replacement, before, after);
            document.cursor.set_offset(&document.buffer, new_cursor);
            document.selection = crate::editor::selection::SelectionRange::point(new_cursor);
            document.reanalyze(crate::editor::syntax::SqlDialect::Postgres);
            document.invalidate_prediction();
            document.completion = crate::editor::completion::CompletionState::default();
            document.execution_diagnostic = None;
            document
                .diagnostics
                .retain(|diagnostic| diagnostic.source != crate::editor::diagnostics::DiagnosticSource::Database);
            document.dirty = true;
            let new_version = document.buffer.version();
            applied_patch = Some(db_pro_core::domain::agent::AgentToolOutput::PatchApplied {
                document_id: patch.document_id,
                new_version,
                range: patch.range,
            });
        }
        let request_id = self.task_bridge.next_request_id();
        if let Some(session) = self.agent.sessions.get_mut(&pending.document_id) {
            session.pending_confirmation = None;
            session.request_id = Some(request_id);
            session.state = db_pro_core::domain::agent::AgentSessionState::Running;
        }
        let _ = self.task_bridge.send(UiCommand::ContinueAgentRun {
            request_id,
            run_id: pending.run_id,
            approved,
            current_document,
            applied_patch,
        });
    }

    pub(super) fn open_agent_result_in_workspace(&mut self, call_id: &str) {
        let Some(document) = self
            .query_session_state
            .documents
            .get_mut(self.query_session_state.active_document_index)
        else {
            return;
        };
        let document_id = document.id.clone();
        let Some(session) = self.agent.sessions.get(&document_id) else {
            return;
        };
        let Some(tool_result) = session.tool_results.get(call_id) else {
            return;
        };
        if let db_pro_core::domain::agent::AgentToolOutput::QueryResult { summary, .. } = &tool_result.output {
            let columns = summary
                .columns
                .iter()
                .map(|col| crate::runtime::UiColumn {
                    name: col.name.clone(),
                    data_type: col.data_type.clone().unwrap_or_else(|| "text".to_owned()),
                    nullable: true,
                })
                .collect();
            let rows = summary
                .sample_rows
                .iter()
                .map(|row| {
                    row.iter()
                        .map(|cell| crate::runtime::UiCell::Text(cell.clone()))
                        .collect()
                })
                .collect();
            let sample_len = summary.sample_rows.len();
            let total_rows = summary.row_count.unwrap_or(sample_len as u64);
            let ui_result = crate::runtime::UiQueryResult {
                columns,
                rows,
                row_count: total_rows,
                duration_ms: tool_result.duration_ms.unwrap_or(0),
            };
            document.query_result = Some(ui_result.clone());
            document.query_results = vec![ui_result];
            document.active_result_index = 0;
            self.query_output_state.active_tab = OutputTab::Results;
            // The agent's result replaces the rows behind the grid.
            self.invalidate_grid_projection();
            if total_rows > sample_len as u64 {
                self.runtime_message = format!("Showing {sample_len} sampled rows of {total_rows} total rows.");
            } else {
                self.runtime_message = format!("Opened Agent query result ({total_rows} rows)");
            }
        }
    }

    pub(super) fn retry_agent_run(&mut self) {
        let Some(document_id) = self
            .query_session_state
            .documents
            .get(self.query_session_state.active_document_index)
            .map(|document| document.id.clone())
        else {
            return;
        };
        let prompt = self.agent.sessions.get(&document_id).and_then(|session| {
            session
                .messages
                .iter()
                .rev()
                .find(|msg| msg.role == AgentRole::User)
                .map(|msg| msg.content.clone())
        });
        if let Some(prompt) = prompt {
            self.submit_typed_agent_prompt(prompt);
        }
    }

    pub(super) fn cancel_active_agent_run(&mut self) {
        let Some(document_id) = self
            .query_session_state
            .documents
            .get(self.query_session_state.active_document_index)
            .map(|document| document.id.clone())
        else {
            return;
        };
        let Some(run_id) = self
            .agent
            .sessions
            .get(&document_id)
            .and_then(|session| session.active_run_id)
        else {
            return;
        };
        let request_id = self.task_bridge.next_request_id();
        let _ = self.task_bridge.send(UiCommand::CancelAgentRun { request_id, run_id });
    }
}

fn flush_agent_stream(session: &mut super::agent_workflow_state::AgentUiSession) {
    if session.streaming_text.is_empty() {
        return;
    }
    session.messages.push(AgentMessage {
        role: AgentRole::Assistant,
        content: std::mem::take(&mut session.streaming_text),
        sql: None,
        requires_confirmation: false,
    });
}

fn apply_agent_workflow_event(
    session: &mut super::agent_workflow_state::AgentUiSession,
    event: db_pro_core::domain::agent_workflow::AgentWorkflowEvent,
    run_id: db_pro_core::domain::agent::AgentRunId,
) {
    use super::agent_workflow_state::{
        AgentAuditEntry, AgentUiActivity, AgentUiActivityStatus, AgentUiConfirmation, AgentUiToolResult,
    };
    use db_pro_core::domain::agent_workflow::AgentWorkflowEvent;

    match event {
        AgentWorkflowEvent::TextDelta { delta, .. } => {
            session.state = db_pro_core::domain::agent::AgentSessionState::Running;
            session.streaming_text.push_str(&delta);
        }
        AgentWorkflowEvent::ToolRequested { call, .. } => {
            flush_agent_stream(session);
            session.state = db_pro_core::domain::agent::AgentSessionState::Running;
            session.activities.push(AgentUiActivity {
                call_id: Some(call.call_id),
                tool: Some(call.tool),
                label: agent_tool_label(call.tool),
                status: AgentUiActivityStatus::Running,
                duration_ms: None,
            });
        }
        AgentWorkflowEvent::ToolCompleted { call_id, result, .. } => {
            set_activity_status(session, &call_id, AgentUiActivityStatus::Success);
            session.tool_results.insert(
                call_id.clone(),
                AgentUiToolResult {
                    call_id: call_id.clone(),
                    tool: result.tool,
                    output: result.output.clone(),
                    duration_ms: None,
                    status: AgentUiActivityStatus::Success,
                },
            );
            session.audit_trail.push(AgentAuditEntry {
                run_id,
                tool: result.tool,
                duration_ms: None,
                safety: None,
                confirmed: false,
            });
            if session.audit_trail.len() > 30 {
                session.audit_trail.remove(0);
            }
            session.state = db_pro_core::domain::agent::AgentSessionState::Running;
        }
        AgentWorkflowEvent::ToolFailed {
            call_id, tool, error, ..
        } => {
            set_activity_status(session, &call_id, AgentUiActivityStatus::Failed);
            session.messages.push(AgentMessage {
                role: AgentRole::Assistant,
                content: format!("{} failed: {}", agent_tool_label(tool), error.format_user_error()),
                sql: None,
                requires_confirmation: false,
            });
            session.state = db_pro_core::domain::agent::AgentSessionState::Running;
        }
        AgentWorkflowEvent::ConfirmationRequired {
            call_id, kind, preview, ..
        } => {
            set_activity_status(session, &call_id, AgentUiActivityStatus::AwaitingConfirmation);
            let document_id = session
                .session
                .as_ref()
                .map(|s| s.document_id.clone())
                .unwrap_or_default();
            session.pending_confirmation = Some(AgentUiConfirmation {
                run_id,
                call_id,
                kind,
                preview,
                document_id,
            });
            session.state = db_pro_core::domain::agent::AgentSessionState::AwaitingConfirmation;
        }
        AgentWorkflowEvent::Completed { .. } => {
            finish_agent_session(session, db_pro_core::domain::agent::AgentSessionState::Completed);
        }
        AgentWorkflowEvent::Failed { message, .. } => {
            flush_agent_stream(session);
            session.messages.push(AgentMessage {
                role: AgentRole::Assistant,
                content: message,
                sql: None,
                requires_confirmation: false,
            });
            finish_agent_session(session, db_pro_core::domain::agent::AgentSessionState::Failed);
        }
        AgentWorkflowEvent::Cancelled { .. } => {
            for activity in &mut session.activities {
                if matches!(
                    activity.status,
                    AgentUiActivityStatus::Running | AgentUiActivityStatus::AwaitingConfirmation
                ) {
                    activity.status = AgentUiActivityStatus::Cancelled;
                }
            }
            finish_agent_session(session, db_pro_core::domain::agent::AgentSessionState::Cancelled);
        }
    }
}

fn set_activity_status(
    session: &mut super::agent_workflow_state::AgentUiSession,
    call_id: &str,
    status: super::agent_workflow_state::AgentUiActivityStatus,
) {
    if let Some(activity) = session
        .activities
        .iter_mut()
        .rev()
        .find(|activity| activity.call_id.as_deref() == Some(call_id))
    {
        activity.status = status;
    }
}

pub(super) fn finish_agent_session(
    session: &mut super::agent_workflow_state::AgentUiSession,
    state: db_pro_core::domain::agent::AgentSessionState,
) {
    let terminal_activity_status = match state {
        db_pro_core::domain::agent::AgentSessionState::Completed => {
            Some(super::agent_workflow_state::AgentUiActivityStatus::Success)
        }
        db_pro_core::domain::agent::AgentSessionState::Failed => {
            Some(super::agent_workflow_state::AgentUiActivityStatus::Failed)
        }
        db_pro_core::domain::agent::AgentSessionState::Cancelled => {
            Some(super::agent_workflow_state::AgentUiActivityStatus::Cancelled)
        }
        _ => None,
    };
    if let Some(status) = terminal_activity_status {
        for activity in &mut session.activities {
            if matches!(
                activity.status,
                super::agent_workflow_state::AgentUiActivityStatus::Running
                    | super::agent_workflow_state::AgentUiActivityStatus::AwaitingConfirmation
            ) {
                activity.status = status;
            }
        }
    }
    flush_agent_stream(session);
    session.state = state;
    session.active_run_id = None;
    session.request_id = None;
    session.pending_confirmation = None;
}

fn agent_tool_label(tool: db_pro_core::domain::agent::AgentTool) -> String {
    use db_pro_core::domain::agent::AgentTool;
    match tool {
        AgentTool::InspectSchema => "Inspecting schema".to_owned(),
        AgentTool::InspectTable => "Reading table metadata".to_owned(),
        AgentTool::InspectColumns => "Reading columns".to_owned(),
        AgentTool::InspectForeignKeys => "Reading foreign keys".to_owned(),
        AgentTool::GetCurrentQuery => "Reading current query".to_owned(),
        AgentTool::PatchQuery => "Preparing SQL change".to_owned(),
        AgentTool::RunQuery => "Running query".to_owned(),
        AgentTool::InspectQueryResult => "Inspecting query result".to_owned(),
        AgentTool::ExplainQuery => "Explaining query".to_owned(),
        AgentTool::SuggestIndexes => "Suggesting indexes".to_owned(),
        AgentTool::MonitoringRead => "Reading monitoring snapshot".to_owned(),
    }
}
