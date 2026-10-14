//! Tests for `../saved_task_state.rs` (moved out of the source file).
    use super::{SavedTaskRunPreparation, SavedTaskState};
    use chrono::Utc;
    use db_pro_core::domain::saved_task::SavedTaskRunTrigger;

    #[test]
    fn default_state_is_empty_and_requires_no_confirmation() {
        let state = SavedTaskState::default();

        assert!(state.store.tasks.is_empty());
        assert!(state.draft.is_none());
        assert!(!state.dirty);
        assert!(!state.confirm_destructive);
        assert!(state.pending_destructive_task_id.is_none());
    }

    #[test]
    fn draft_lifecycle_is_owned_by_saved_task_state() {
        let now = Utc::now();
        let mut state = SavedTaskState::default();

        state.begin_sql_draft("connection-1".to_owned(), "select 1".to_owned(), now);
        assert!(state.draft.is_some());
        state.commit_draft(now).expect("valid draft should save");

        assert!(state.draft.is_none());
        assert!(state.dirty);
        assert_eq!(state.store.tasks.len(), 1);
    }

    #[test]
    fn destructive_tasks_cannot_be_scheduled_by_state_policy() {
        let now = Utc::now();
        let mut state = SavedTaskState::default();
        state.begin_sql_draft("connection-1".to_owned(), "drop table users".to_owned(), now);
        state.commit_draft(now).expect("valid draft should save");
        let task_id = state.store.tasks[0].id;

        let error = state
            .enable_schedule(task_id, 60, now)
            .expect_err("destructive task must be blocked");
        assert!(error.contains("Destructive tasks"));
    }

    #[test]
    fn run_policy_requires_manual_confirmation_and_blocks_scheduled_destructive_work() {
        let now = Utc::now();
        let mut state = SavedTaskState::default();
        state.begin_sql_draft("connection-1".to_owned(), "drop table users".to_owned(), now);
        state.commit_draft(now).expect("valid draft should save");
        let task_id = state.store.tasks[0].id;

        assert!(matches!(
            state.prepare_run(task_id, SavedTaskRunTrigger::Manual, true),
            SavedTaskRunPreparation::NeedsConfirmation
        ));
        assert_eq!(state.pending_destructive_task_id, Some(task_id));
        assert!(matches!(
            state.prepare_run(task_id, SavedTaskRunTrigger::Scheduled, true),
            SavedTaskRunPreparation::Blocked(_)
        ));
    }