use super::workspace_session::NamedSessionStore;

/// UI state for named workspace sessions and restore diagnostics.
#[derive(Debug)]
pub(crate) struct WorkspaceSessionState {
    pub store: NamedSessionStore,
    pub name_draft: String,
    pub selected_id: Option<String>,
    pub last_restore_notes: Vec<String>,
}

impl Default for WorkspaceSessionState {
    fn default() -> Self {
        Self {
            store: NamedSessionStore::new(),
            name_draft: String::new(),
            selected_id: None,
            last_restore_notes: Vec::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::WorkspaceSessionState;

    #[test]
    fn default_state_has_no_named_session_selection_or_restore_notes() {
        let state = WorkspaceSessionState::default();

        assert!(state.store.sessions.is_empty());
        assert!(state.name_draft.is_empty());
        assert!(state.selected_id.is_none());
        assert!(state.last_restore_notes.is_empty());
    }
}
