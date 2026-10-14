//! Tests for `../preferences_state.rs` (moved out of the source file).
    use super::PreferencesState;

    #[test]
    fn default_preferences_are_light_and_non_predictive() {
        let state = PreferencesState::default();

        assert!(!state.dark_mode);
        assert!(!state.reduce_motion);
        assert!(state.keybindings_filter.is_empty());
        assert!(state.keybinding_edit_id.is_none());
        assert_eq!(state.prediction_mode, crate::editor::PredictionMode::Off);
    }