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
    pub(super) fn clear_input(&mut self) {
        self.input.clear();
    }

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
