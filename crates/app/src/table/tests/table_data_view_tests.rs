//! Tests for `../table_data_view.rs` (moved out of the source file).
    use super::can_mutate_table_toolbar;

    #[test]
    fn allows_toolbar_mutations_for_a_writable_idle_connection() {
        assert!(can_mutate_table_toolbar(true, false));
    }

    #[test]
    fn disables_mutations_without_write_access_or_while_query_runs() {
        assert!(!can_mutate_table_toolbar(false, false));
        assert!(!can_mutate_table_toolbar(true, true));
    }