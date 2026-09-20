use super::*;

/// Owns agent workspace state independently from the shell and query session.
#[derive(Debug)]
pub(crate) struct AgentState {
    pub(super) provider_label: String,
    pub(super) provider_detail: String,
    pub(super) input: String,
    pub(super) sessions: HashMap<String, AgentUiSession>,
    pub(super) auto_run_read_only: bool,
    pub(super) settings_open: bool,
    pub(super) api_key_draft: String,
    pub(super) api_key_show_password: bool,
    pub(super) configure_request: Option<crate::RequestId>,
}

impl Default for AgentState {
    fn default() -> Self {
        let provider_info = OfflineAgentProvider.info();
        Self {
            provider_label: provider_info.label.to_owned(),
            provider_detail: provider_info.detail.to_owned(),
            input: String::new(),
            sessions: HashMap::new(),
            auto_run_read_only: false,
            settings_open: false,
            api_key_draft: String::new(),
            api_key_show_password: false,
            configure_request: None,
        }
    }
}

#[derive(Debug)]
pub(super) struct PreparedAgentRun {
    pub(super) document_id: String,
    pub(super) request_id: crate::RequestId,
    pub(super) prompt: String,
    pub(super) session: db_pro_core::domain::agent::AgentSession,
    pub(super) document: db_pro_core::domain::agent::AgentDocumentSnapshot,
    pub(super) mode: db_pro_core::domain::agent::AgentMode,
    pub(super) allow_read_only_auto_run: bool,
    pub(super) context: db_pro_core::domain::agent_context::AgentContext,
}

#[derive(Debug)]
pub(super) struct AgentRunPreparation {
    pub(super) request_id: crate::RequestId,
    pub(super) prompt: String,
    pub(super) document: db_pro_core::domain::agent::AgentDocumentSnapshot,
    pub(super) connection_id: Option<String>,
    pub(super) schema: Option<String>,
    pub(super) mode: db_pro_core::domain::agent::AgentMode,
    pub(super) context: db_pro_core::domain::agent_context::AgentContext,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum AgentRunPreparationError {
    AlreadyActive,
    SessionUnavailable,
}

#[derive(Debug)]
pub(super) struct PreparedAgentContinuation {
    pub(super) request_id: crate::RequestId,
    pub(super) run_id: db_pro_core::domain::agent::AgentRunId,
    pub(super) approved: bool,
    pub(super) current_document: Option<db_pro_core::domain::agent::AgentDocumentSnapshot>,
    pub(super) applied_patch: Option<db_pro_core::domain::agent::AgentToolOutput>,
}

impl AgentState {
    pub(super) fn clear_session(&mut self, document_id: &str) {
        if let Some(session) = self.sessions.get_mut(document_id) {
            session.messages.clear();
            session.activities.clear();
            session.streaming_text.clear();
            session.tool_results.clear();
            session.state = db_pro_core::domain::agent::AgentSessionState::Idle;
            session.active_run_id = None;
            session.request_id = None;
            session.pending_confirmation = None;
        }
    }

    pub(super) fn open_settings(&mut self) {
        self.settings_open = true;
        self.api_key_draft.clear();
        self.api_key_show_password = false;
    }

    pub(super) fn close_settings(&mut self) {
        self.settings_open = false;
        self.api_key_draft.clear();
        self.api_key_show_password = false;
    }

    pub(super) fn toggle_settings(&mut self) {
        if self.settings_open {
            self.close_settings();
        } else {
            self.open_settings();
        }
    }

    pub(super) fn prepare_run(
        &mut self,
        preparation: AgentRunPreparation,
    ) -> Result<PreparedAgentRun, AgentRunPreparationError> {
        let AgentRunPreparation {
            request_id,
            prompt,
            document,
            connection_id,
            schema,
            mode,
            context,
        } = preparation;
        let document_id = document.document_id.clone();
        let session = self
            .sessions
            .entry(document_id.clone())
            .or_insert_with(|| AgentUiSession::for_document(&document_id, connection_id.clone(), schema.clone()));
        if session.session.is_none() {
            *session = AgentUiSession::for_document(&document_id, connection_id.clone(), schema.clone());
        }
        if session.active_run_id.is_some() || session.request_id.is_some() {
            return Err(AgentRunPreparationError::AlreadyActive);
        }
        let Some(mut core_session) = session.session.clone() else {
            return Err(AgentRunPreparationError::SessionUnavailable);
        };

        core_session.connection_id = connection_id;
        core_session.schema = schema;
        session.session = Some(core_session.clone());
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
        session.request_id = Some(request_id);
        self.input.clear();

        Ok(PreparedAgentRun {
            document_id,
            request_id,
            prompt,
            session: core_session,
            document,
            mode,
            allow_read_only_auto_run: self.auto_run_read_only,
            context,
        })
    }

    pub(super) fn mark_run_failed(&mut self, document_id: &str) {
        if let Some(session) = self.sessions.get_mut(document_id) {
            session.request_id = None;
            session.state = db_pro_core::domain::agent::AgentSessionState::Failed;
        }
    }

    pub(super) fn prepare_continuation(
        &mut self,
        pending: &super::agent_workflow_state::AgentUiConfirmation,
        request_id: crate::RequestId,
        approved: bool,
        current_document: Option<db_pro_core::domain::agent::AgentDocumentSnapshot>,
        applied_patch: Option<db_pro_core::domain::agent::AgentToolOutput>,
    ) -> PreparedAgentContinuation {
        if let Some(session) = self.sessions.get_mut(&pending.document_id) {
            session.pending_confirmation = None;
            session.request_id = Some(request_id);
            session.state = db_pro_core::domain::agent::AgentSessionState::Running;
        }
        PreparedAgentContinuation {
            request_id,
            run_id: pending.run_id,
            approved,
            current_document,
            applied_patch,
        }
    }
}

impl DbProApp {
    pub(super) fn reset_agent_context(&mut self) {
        self.agent.input.clear();
    }

    pub(super) fn agent_context(&self) -> AgentContext {
        let connection_name = Some(self.active_connection_name().to_owned());
        let driver = self.active_driver().to_owned();
        let selected_columns = self
            .table
            .state
            .table_info
            .as_ref()
            .map(|info| info.columns.iter().map(|column| column.name.clone()).collect())
            .unwrap_or_default();
        let current_sql = if self.query.session.selected_text.trim().is_empty() {
            self.query.session.active_text().to_owned()
        } else {
            self.query.session.selected_text.clone()
        };
        let result_summary = self
            .query
            .session
            .active_result()
            .or(self.table.data_query.result.as_ref())
            .map(|result| format!("{} rows returned in {} ms", result.row_count, result.duration_ms));
        let last_error = self.has_runtime_error().then(|| self.feedback.runtime_message.clone());

        AgentContext {
            connection_name,
            driver,
            tables: self.active_schema_table_names(),
            columns: self.active_schema_column_names(),
            schema: Some(self.active_schema().to_owned()),
            selected_table: self.schema.explorer.selected_table.clone(),
            selected_columns,
            current_sql,
            result_summary,
            explain_plan: self.query.session.active_explain_plan().map(str::to_owned),
            last_error,
            workspace_files: self.workspace.files.workspace_context_items.clone(),
        }
    }

    pub(super) fn submit_agent_prompt(&mut self) {
        if !self.workspace.files.workspace_context_items.is_empty() && !self.workspace.files.ide_workspace.is_trusted()
        {
            self.feedback.runtime_message =
                "Trust the workspace before sending folder/file context to Agent".to_owned();
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
            .query
            .session
            .documents
            .get(self.query.session.active_document_index)
        else {
            self.feedback.runtime_message = "No query document is available for Agent".to_owned();
            return;
        };
        if self.agent.provider_label == "Offline draft" {
            self.feedback.runtime_message =
                "AI provider is not configured. Enter an API key in Agent Settings.".to_owned();
            self.feedback
                .show_error_toast("Configure an API key in Agent Settings to start.");
            return;
        }
        let document_id = document.id.clone();
        let connection_id = document
            .connection_id
            .clone()
            .or_else(|| self.connection.lifecycle.active_connection_id().map(str::to_owned));
        let schema = document
            .schema
            .clone()
            .or_else(|| Some(self.active_schema().to_owned()));
        let snapshot = agent_context::document_snapshot(document);
        let context = agent_context::build_context(
            &prompt,
            document,
            connection_id.as_deref(),
            schema.as_deref(),
            &self.schema.explorer.schema.table_details,
        );

        let request_id = self.task_bridge.next_request_id();
        let mode = self
            .agent
            .sessions
            .get(&document_id)
            .map(|session| session.mode)
            .unwrap_or(db_pro_core::domain::agent::AgentMode::Ask);
        let prepared = match self.agent.prepare_run(AgentRunPreparation {
            request_id,
            prompt,
            document: snapshot,
            connection_id,
            schema,
            mode,
            context,
        }) {
            Ok(prepared) => prepared,
            Err(AgentRunPreparationError::AlreadyActive) => {
                self.feedback.runtime_message = "An Agent run is already active for this query".to_owned();
                return;
            }
            Err(AgentRunPreparationError::SessionUnavailable) => {
                self.feedback.runtime_message = "Agent session could not be initialized".to_owned();
                return;
            }
        };
        self.feedback.runtime_message = format!("Sending request to {}…", self.agent.provider_label);
        if self
            .task_bridge
            .send(UiCommand::StartAgentRun {
                request_id: prepared.request_id,
                prompt: prepared.prompt,
                session: prepared.session,
                document: prepared.document,
                mode: prepared.mode,
                allow_read_only_auto_run: prepared.allow_read_only_auto_run,
                context: prepared.context,
            })
            .is_err()
        {
            self.agent.mark_run_failed(&prepared.document_id);
            self.feedback.runtime_message = "Agent runtime unavailable".to_owned();
        }
    }

    pub(super) fn on_agent_workflow_event(&mut self, event: db_pro_core::domain::agent_workflow::AgentWorkflowEvent) {
        agent_events::on_agent_workflow_event(&mut self.agent, event);
    }

    pub(super) fn agent_confirmation_action(&mut self, approved: bool) {
        let Some(document_id) = self
            .query
            .session
            .documents
            .get(self.query.session.active_document_index)
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
        let target_doc_index = agent_confirmation::target_document_index(
            &self.query.session.documents,
            self.query.session.active_document_index,
            &pending.document_id,
        );
        let current_document = self
            .query
            .session
            .documents
            .get(target_doc_index)
            .map(agent_context::document_snapshot);
        let mut applied_patch = None;
        if approved && pending.kind == db_pro_core::domain::agent_workflow::AgentConfirmationKind::ApplyPatch {
            match agent_confirmation::apply_approved_patch(
                &mut self.query.session.documents,
                target_doc_index,
                &pending,
            ) {
                Ok(output) => applied_patch = Some(output),
                Err(agent_confirmation::AgentConfirmationError::PreviewUnavailable) => {
                    self.feedback.runtime_message = "Agent patch preview is unavailable".to_owned();
                    return;
                }
                Err(agent_confirmation::AgentConfirmationError::DocumentUnavailable) => return,
                Err(agent_confirmation::AgentConfirmationError::DocumentChanged) => {
                    self.feedback.runtime_message = "This query changed since the suggestion was created.".to_owned();
                    self.feedback
                        .show_error_toast("The query changed since the suggestion was created.");
                    self.agent_confirmation_action(false);
                    return;
                }
                Err(agent_confirmation::AgentConfirmationError::InvalidRange) => {
                    self.feedback.runtime_message = "Agent patch range is no longer valid".to_owned();
                    self.agent_confirmation_action(false);
                    return;
                }
            }
        }
        let request_id = self.task_bridge.next_request_id();
        let continuation =
            self.agent
                .prepare_continuation(&pending, request_id, approved, current_document, applied_patch);
        self.send_command_best_effort(UiCommand::ContinueAgentRun {
            request_id: continuation.request_id,
            run_id: continuation.run_id,
            approved: continuation.approved,
            current_document: continuation.current_document,
            applied_patch: continuation.applied_patch,
        });
    }

    pub(super) fn open_agent_result_in_workspace(&mut self, call_id: &str) {
        let Some(document) = self
            .query
            .session
            .documents
            .get_mut(self.query.session.active_document_index)
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
        if let Some(ui_result) = agent_result_projection::query_result(tool_result) {
            let sample_len = ui_result.rows.len();
            let total_rows = ui_result.row_count;
            document.query_result = Some(ui_result.clone());
            document.query_results = vec![ui_result];
            document.active_result_index = 0;
            self.query.output.active_tab = OutputTab::Results;
            // The agent's result replaces the rows behind the grid.
            self.table.data.invalidate_grid_projection();
            if total_rows > sample_len as u64 {
                self.feedback.runtime_message =
                    format!("Showing {sample_len} sampled rows of {total_rows} total rows.");
            } else {
                self.feedback.runtime_message = format!("Opened Agent query result ({total_rows} rows)");
            }
        }
    }

    pub(super) fn retry_agent_run(&mut self) {
        let Some(document_id) = self
            .query
            .session
            .documents
            .get(self.query.session.active_document_index)
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
            .query
            .session
            .documents
            .get(self.query.session.active_document_index)
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
        self.send_command_best_effort(UiCommand::CancelAgentRun { request_id, run_id });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_agent_state_starts_in_offline_draft_mode() {
        let state = AgentState::default();

        assert_eq!(state.provider_label, "Offline draft");
        assert!(state.provider_detail.contains("unexecuted"));
        assert!(state.sessions.is_empty());
        assert!(state.input.is_empty());
        assert!(!state.settings_open);
        assert!(state.configure_request.is_none());
    }

    #[test]
    fn prepare_run_owns_session_transition_and_request_identity() {
        let mut state = AgentState::default();
        let document = db_pro_core::domain::agent::AgentDocumentSnapshot {
            document_id: "query-1".to_owned(),
            document_version: 3,
            sql: "select 1".to_owned(),
            cursor_offset: 0,
            selection: None,
            current_statement: Some("select 1".to_owned()),
        };
        let context = db_pro_core::domain::agent_context::AgentContext {
            document_id: "query-1".to_owned(),
            document_version: 3,
            connection_id: None,
            schema: None,
            current_sql: "select 1".to_owned(),
            user_request: "inspect".to_owned(),
            selected_range: None,
            referenced_tables: Vec::new(),
            foreign_keys: Vec::new(),
            diagnostics: Vec::new(),
            result_summary: None,
        };

        let prepared = state
            .prepare_run(AgentRunPreparation {
                request_id: crate::RequestId(7),
                prompt: "inspect".to_owned(),
                document,
                connection_id: None,
                schema: None,
                mode: db_pro_core::domain::agent::AgentMode::Ask,
                context,
            })
            .expect("first run should be prepared");

        assert_eq!(prepared.request_id, crate::RequestId(7));
        let session = state.sessions.get("query-1").expect("session should exist");
        assert_eq!(session.request_id, Some(crate::RequestId(7)));
        assert_eq!(session.state, db_pro_core::domain::agent::AgentSessionState::Running);
        assert_eq!(session.messages.len(), 1);
    }

    #[test]
    fn prepare_run_rejects_duplicate_without_adding_a_message() {
        let mut state = AgentState::default();
        let document = db_pro_core::domain::agent::AgentDocumentSnapshot {
            document_id: "query-1".to_owned(),
            document_version: 0,
            sql: String::new(),
            cursor_offset: 0,
            selection: None,
            current_statement: None,
        };
        let context = db_pro_core::domain::agent_context::AgentContext {
            document_id: "query-1".to_owned(),
            document_version: 0,
            connection_id: None,
            schema: None,
            current_sql: String::new(),
            user_request: "inspect".to_owned(),
            selected_range: None,
            referenced_tables: Vec::new(),
            foreign_keys: Vec::new(),
            diagnostics: Vec::new(),
            result_summary: None,
        };
        state
            .prepare_run(AgentRunPreparation {
                request_id: crate::RequestId(1),
                prompt: "first".to_owned(),
                document: document.clone(),
                connection_id: None,
                schema: None,
                mode: db_pro_core::domain::agent::AgentMode::Ask,
                context: context.clone(),
            })
            .expect("first run should be prepared");

        assert!(matches!(
            state.prepare_run(AgentRunPreparation {
                request_id: crate::RequestId(2),
                prompt: "second".to_owned(),
                document,
                connection_id: None,
                schema: None,
                mode: db_pro_core::domain::agent::AgentMode::Ask,
                context,
            }),
            Err(AgentRunPreparationError::AlreadyActive)
        ));
        assert_eq!(state.sessions["query-1"].messages.len(), 1);
    }
}
