//! Tests for `../feedback_state.rs` (moved out of the source file).
    use super::FeedbackState;

    #[test]
    fn default_feedback_is_quiet() {
        let state = FeedbackState::default();

        assert!(state.runtime_message.is_empty());
        assert!(state.copy_status.is_empty());
        assert!(state.toasts.is_empty());
    }