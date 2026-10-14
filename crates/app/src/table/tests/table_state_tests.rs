//! Tests for `../table_state.rs` (moved out of the source file).
    use super::*;

    #[test]
    fn default_table_state_starts_at_structure_without_requests() {
        let state = TableState::default();

        assert_eq!(state.table_view, TableView::Structure);
        assert!(state.table_info.is_none());
        assert!(state.table_info_request.is_none());
    }

    #[test]
    fn execute_ddl_requires_non_empty_loaded_sql() {
        let state = TableState::default();

        assert_eq!(
            state.ddl_sql(),
            Err("Load the table DDL before executing it".to_owned())
        );
    }