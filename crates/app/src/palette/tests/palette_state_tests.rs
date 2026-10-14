//! Tests for `../palette_state.rs` (moved out of the source file).
    use super::{PaletteMode, PaletteState, SearchScope};

    #[test]
    fn default_palette_state_is_closed_and_unselected() {
        let state = PaletteState::default();

        assert!(state.mode.is_none());
        assert!(state.query.is_empty());
        assert_eq!(state.selected, 0);
        assert!(!state.focus_requested);
    }

    #[test]
    fn opening_palette_resets_query_selection_and_focuses_the_input() {
        let mut state = PaletteState {
            query: "stale".to_owned(),
            selected: 4,
            ..Default::default()
        };
        state.open_with_scope(PaletteMode::QuickOpen, SearchScope::Schema);

        assert_eq!(state.mode, Some(PaletteMode::QuickOpen));
        assert_eq!(state.scope, SearchScope::Schema);
        assert!(state.query.is_empty());
        assert_eq!(state.selected, 0);
        assert!(state.focus_requested);
    }