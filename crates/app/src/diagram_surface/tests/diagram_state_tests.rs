//! Tests for `../diagram_state.rs` (moved out of the source file).
    use super::DiagramState;

    #[test]
    fn default_diagram_state_starts_at_safe_viewport_defaults() {
        let state = DiagramState::default();

        assert_eq!(state.zoom, 1.0);
        assert_eq!(state.neighborhood_depth, 1);
        assert_eq!(state.schema_version, 1);
        assert!(state.graph.nodes.is_empty());
        assert!(state.search.is_empty());
    }