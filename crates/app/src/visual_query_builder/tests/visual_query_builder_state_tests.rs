//! Tests for `../visual_query_builder_state.rs` (moved out of the source file).
    use super::*;

    #[test]
    fn default_state_has_safe_form_defaults() {
        let state = VisualQueryBuilderState::default();

        assert!(!state.open);
        assert_eq!(state.where_op, "=");
        assert_eq!(state.limit, "100");
        assert!(state.model.tables.is_empty());
    }

    #[test]
    fn add_table_and_preview_are_kept_in_one_transition() {
        let mut state = VisualQueryBuilderState {
            add_table: "public.users".into(),
            ..Default::default()
        };

        state.add_selected_table(BuilderDialect::Postgres);

        assert!(state.error.is_none());
        assert_eq!(state.model.tables[0].alias, "use");
        assert!(state.sql_preview.contains("FROM \"public\".\"users\""));
        assert!(state.add_table.is_empty());
    }