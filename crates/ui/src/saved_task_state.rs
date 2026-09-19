use db_pro_core::domain::saved_task::{SavedTask, SavedTaskStore};
use uuid::Uuid;

/// UI state for saved-task editing, scheduling, and confirmation flows.
#[derive(Debug)]
pub(crate) struct SavedTaskState {
    pub store: SavedTaskStore,
    pub draft: Option<SavedTask>,
    pub dirty: bool,
    pub confirm_destructive: bool,
    pub pending_destructive_task_id: Option<Uuid>,
}

impl Default for SavedTaskState {
    fn default() -> Self {
        Self {
            store: SavedTaskStore::new(),
            draft: None,
            dirty: false,
            confirm_destructive: false,
            pending_destructive_task_id: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::SavedTaskState;

    #[test]
    fn default_state_is_empty_and_requires_no_confirmation() {
        let state = SavedTaskState::default();

        assert!(state.store.tasks.is_empty());
        assert!(state.draft.is_none());
        assert!(!state.dirty);
        assert!(!state.confirm_destructive);
        assert!(state.pending_destructive_task_id.is_none());
    }
}
