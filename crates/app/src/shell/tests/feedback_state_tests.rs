//! Tests for `../feedback_state.rs` (moved out of the source file).
    use super::FeedbackState;

    #[test]
    fn default_feedback_is_quiet() {
        let state = FeedbackState::default();

        assert!(state.runtime_message.is_empty());
    }

    #[test]
    fn activity_scoped_message_hides_on_other_activities() {
        use super::super::app_types::Activity;
        let mut state = FeedbackState::default();
        state.set_activity_runtime_message(Activity::Security, "Security · 3 role(s)");
        assert_eq!(
            state.runtime_message_for_display(Activity::Security),
            "Security · 3 role(s)"
        );
        assert!(state.runtime_message_for_display(Activity::Explorer).is_empty());
        assert!(state.copy_status.is_empty());
        assert!(state.toasts.is_empty());
    }