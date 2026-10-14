//! Tests for `../query_output_state.rs` (moved out of the source file).
    use super::*;

    #[test]
    fn output_state_defaults_to_results_without_document_overrides() {
        let state = QueryOutputState::default();

        assert_eq!(state.active_tab, OutputTab::Results);
        assert!(state.tabs_by_document.is_empty());
    }

    #[test]
    fn document_tab_overrides_are_owned_by_output_state() {
        let mut state = QueryOutputState::default();

        state.set_active_for_document("query-1", OutputTab::Explain);
        assert_eq!(state.tab_for_document("query-1"), OutputTab::Explain);
        assert_eq!(state.active_tab, OutputTab::Explain);

        state.set_for_document("query-2", OutputTab::Messages);
        assert_eq!(state.tab_for_document("query-2"), OutputTab::Messages);
        assert_eq!(state.active_tab, OutputTab::Explain);
    }

    #[test]
    fn tab_selection_without_document_still_switches_the_visible_pane() {
        let mut state = QueryOutputState::default();

        state.set_active_for_optional_document(None, OutputTab::History);

        assert_eq!(state.active_tab_for_document(None), OutputTab::History);
    }
