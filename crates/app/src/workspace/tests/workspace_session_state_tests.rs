//! Tests for `../workspace_session_state.rs` (moved out of the source file).
    use super::super::workspace_session::WorkspaceSession;
    use super::WorkspaceSessionState;

    #[test]
    fn default_state_has_no_named_session_selection_or_restore_notes() {
        let state = WorkspaceSessionState::default();

        assert!(state.store.sessions.is_empty());
        assert!(state.name_draft.is_empty());
        assert!(state.selected_id.is_none());
        assert!(state.last_restore_notes.is_empty());
    }

    #[test]
    fn store_mutations_keep_selection_owned_by_session_state() {
        let mut state = WorkspaceSessionState::default();
        let session = WorkspaceSession::new_named("Focus");
        let id = session.id.clone();

        state.upsert(session);
        assert_eq!(state.selected_id.as_deref(), Some(id.as_str()));

        let copy_id = state.duplicate(&id).expect("existing session should duplicate");
        assert_eq!(state.selected_id.as_deref(), Some(copy_id.as_str()));
        assert!(state.remove(&copy_id));
        assert!(state.selected_id.is_none());
    }