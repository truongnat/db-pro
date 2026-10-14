use super::workspace_session::NamedSessionStore;
use super::workspace_session::{WorkspaceSession, NAMED_SESSIONS_STORAGE_KEY, SESSION_VERSION};
use eframe::Storage;
use uuid::Uuid;

/// UI state for named workspace sessions and restore diagnostics.
#[derive(Debug)]
pub(crate) struct WorkspaceSessionState {
    pub(super) store: NamedSessionStore,
    pub(super) name_draft: String,
    pub(super) selected_id: Option<String>,
    pub(super) last_restore_notes: Vec<String>,
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

impl WorkspaceSessionState {
    pub(super) fn upsert(&mut self, session: WorkspaceSession) {
        self.selected_id = Some(session.id.clone());
        self.store.upsert(session);
    }

    pub(super) fn remove(&mut self, id: &str) -> bool {
        let removed = self.store.remove(id);
        if removed && self.selected_id.as_deref() == Some(id) {
            self.selected_id = None;
        }
        removed
    }

    pub(super) fn duplicate(&mut self, id: &str) -> Option<String> {
        let mut session = self.store.get(id).cloned()?;
        session.id = Uuid::new_v4().to_string();
        session.name = format!("{} (copy)", session.name);
        session.updated_at = chrono::Utc::now().to_rfc3339();
        let new_id = session.id.clone();
        self.store.upsert(session);
        self.selected_id = Some(new_id.clone());
        Some(new_id)
    }

    pub(super) fn persist_named_store(&self, storage: &mut dyn Storage) {
        if let Ok(raw) = serde_json::to_string(&self.store) {
            storage.set_string(NAMED_SESSIONS_STORAGE_KEY, raw);
        }
    }

    pub(super) fn load_named_store(&mut self, storage: &dyn Storage) {
        let Some(raw) = storage.get_string(NAMED_SESSIONS_STORAGE_KEY) else {
            return;
        };
        let Ok(mut store) = serde_json::from_str::<NamedSessionStore>(&raw) else {
            return;
        };
        store.version = SESSION_VERSION;
        store.sessions = store.sessions.into_iter().map(WorkspaceSession::migrate).collect();
        self.store = store;
    }
}

#[cfg(test)]
mod tests {
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
}
