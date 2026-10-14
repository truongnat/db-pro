//! Tests for `../saved_tasks_surface_view.rs` (moved out of the source file).
    use super::*;

    #[test]
    fn surface_action_keeps_task_identity_for_root_reduction() {
        let id = Uuid::new_v4();
        assert_eq!(
            SavedTasksSurfaceAction::RunTask(id),
            SavedTasksSurfaceAction::RunTask(id)
        );
    }