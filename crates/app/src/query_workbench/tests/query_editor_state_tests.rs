//! Tests for `../query_editor_state.rs` (moved out of the source file).
    use super::*;

    #[test]
    fn default_query_editor_state_starts_with_safe_editor_defaults() {
        let state = QueryEditorState::default();

        assert_eq!(state.query_cursor_line, 1);
        assert_eq!(state.query_cursor_column, 1);
        assert_eq!(state.editor_font_size, 14.0);
        assert_eq!(state.visual_builder.where_op, "=");
        assert_eq!(state.visual_builder.limit, "100");
        assert!(state.diagnostics.is_empty());
    }