//! Tests for `../agent_state.rs` (moved out of the source file).
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
                is_retry: false,
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
                is_retry: false,
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
                is_retry: false,
                context,
            }),
            Err(AgentRunPreparationError::AlreadyActive)
        ));
        assert_eq!(state.sessions["query-1"].messages.len(), 1);
    }