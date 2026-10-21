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

/// #545 — the idle repaint storm: any enabled schedule used to pin
/// `request_repaint_after(50 ms)` even when the next run was an hour away.
/// These tests pin the pure scheduling decision (no wall-clock timing).
mod scheduler_repaint_delay_tests {
    use super::super::{scheduler_repaint_delay, SCHEDULER_REPAINT_CAP, SCHEDULER_REPAINT_FLOOR};
    use super::SavedTaskState;
    use chrono::{Duration, Utc};
    use std::time::Duration as StdDuration;

    fn deadline(secs: i64) -> Option<chrono::DateTime<Utc>> {
        Some(Utc::now() + Duration::seconds(secs))
    }

    #[test]
    fn no_enabled_schedule_means_no_scheduler_repaint() {
        assert_eq!(scheduler_repaint_delay(&[], Utc::now()), None);
        assert!(SavedTaskState::default().enabled_schedule_deadlines().is_empty());
    }

    #[test]
    fn far_deadline_sleeps_to_the_cap_not_to_50ms() {
        // One hourly task: minutes-scale sleep, not the old flat 50 ms.
        let delay = scheduler_repaint_delay(&[deadline(3_600)], Utc::now()).expect("scheduled task keeps waking");
        assert_eq!(delay, SCHEDULER_REPAINT_CAP);
        assert!(delay >= StdDuration::from_secs(60), "far deadline must yield a multi-minute delay: {delay:?}");
        assert_ne!(delay, StdDuration::from_millis(50));
    }

    #[test]
    fn medium_deadline_sleeps_exactly_to_the_deadline() {
        let now = Utc::now();
        let next = now + Duration::seconds(90);
        let delay = scheduler_repaint_delay(&[Some(next)], now).expect("deadline");
        assert_eq!(delay, StdDuration::from_secs(90));
    }

    #[test]
    fn imminent_and_overdue_deadlines_keep_the_50ms_floor() {
        let now = Utc::now();
        // 10 ms away: below the floor → floor, so the tick still fires promptly.
        assert_eq!(
            scheduler_repaint_delay(&[Some(now + Duration::milliseconds(10))], now),
            Some(SCHEDULER_REPAINT_FLOOR)
        );
        // Overdue (a slot the store kept because nothing ticked yet): floor.
        assert_eq!(
            scheduler_repaint_delay(&[Some(now - Duration::seconds(5))], now),
            Some(SCHEDULER_REPAINT_FLOOR)
        );
        // Unassigned slot: due on the next tick.
        assert_eq!(scheduler_repaint_delay(&[None], now), Some(SCHEDULER_REPAINT_FLOOR));
    }

    #[test]
    fn earliest_deadline_wins_across_schedules() {
        let now = Utc::now();
        let far = now + Duration::hours(2);
        let near = now + Duration::seconds(7);
        assert_eq!(scheduler_repaint_delay(&[Some(far), Some(near)], now), Some(StdDuration::from_secs(7)));
        // An overdue slot drags the cadence back to the floor regardless of the far one.
        let overdue = now - Duration::seconds(1);
        assert_eq!(scheduler_repaint_delay(&[Some(far), Some(overdue)], now), Some(SCHEDULER_REPAINT_FLOOR));
    }

    #[test]
    fn deadline_collector_only_tracks_enabled_schedules() {
        let now = Utc::now();
        let mut state = SavedTaskState::default();
        state.begin_sql_draft("connection-1".to_owned(), "select 1".to_owned(), now);
        state.commit_draft(now).expect("draft saves");
        let task_id = state.store.tasks[0].id;
        state.enable_schedule(task_id, 3_600, now).expect("schedule enables");

        let deadlines = state.enabled_schedule_deadlines();
        assert_eq!(deadlines.len(), 1);
        assert!(deadlines[0].is_some(), "enable_schedule assigns the first slot");

        state.disable_schedule(task_id).expect("schedule disables");
        assert!(state.enabled_schedule_deadlines().is_empty());
    }
}