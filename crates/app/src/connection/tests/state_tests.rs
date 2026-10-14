//! Tests for `../state.rs` (moved out of the source file).
    use super::*;

    #[test]
    fn new_dialog_starts_with_a_fresh_draft_and_focus() {
        let mut state = ConnectionDialogState {
            error: "stale error".to_owned(),
            test_valid: true,
            ..Default::default()
        };

        state.transition(ConnectionDialogAction::OpenNew);

        assert!(state.open);
        assert!(state.focus_name_on_open);
        assert!(state.editing_connection_id.is_none());
        assert!(state.error.is_empty());
        assert!(!state.test_valid);
        assert!(state.test_draft.is_none());
    }

    #[test]
    fn closing_dialog_clears_test_state() {
        let mut state = ConnectionDialogState {
            open: true,
            test_valid: true,
            ..Default::default()
        };
        state.test_draft = Some(state.draft.clone());

        state.transition(ConnectionDialogAction::Close);

        assert!(!state.open);
        assert!(!state.test_valid);
        assert!(state.test_draft.is_none());
    }