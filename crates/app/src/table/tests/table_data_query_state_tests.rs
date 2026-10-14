//! Tests for `../table_data_query_state.rs` (moved out of the source file).
    use super::*;
    use crate::{RequestId, UiTableColumn};

    #[test]
    fn keeps_last_successful_inline_sql_until_another_query_succeeds_or_table_resets() {
        let mut state = TableDataQueryState::default();
        let first_sql = "SELECT * FROM customers WHERE id = 1".to_owned();
        state.begin_inline_query(RequestId(1), first_sql.clone());
        assert!(state.complete_inline_query(RequestId(1)));
        assert_eq!(state.active_inline_query_sql.as_deref(), Some(first_sql.as_str()));

        state.begin_inline_query(RequestId(2), "SELECT * FROM customers WHERE id = 2".to_owned());
        assert!(!state.complete_inline_query(RequestId(3)));
        assert!(state.abandon_inline_query(RequestId(2)));
        assert_eq!(state.active_inline_query_sql.as_deref(), Some(first_sql.as_str()));

        state.reset_for_table();
        assert!(state.active_inline_query_sql.is_none());
        assert!(state.pending_inline_query_sql.is_none());
    }

    fn table_info(data_type: &str) -> UiTableInfo {
        UiTableInfo {
            schema: "public".to_owned(),
            name: "orders".to_owned(),
            row_count: None,
            columns: vec![UiTableColumn {
                name: "amount".to_owned(),
                data_type: data_type.to_owned(),
                ..Default::default()
            }],
            primary_key: None,
            indexes: Vec::new(),
            foreign_keys: Vec::new(),
            check_constraints: Vec::new(),
            dependencies: Vec::new(),
        }
    }

    #[test]
    fn commit_filter_draft_validates_typed_values_before_mutating_filters() {
        let mut state = TableDataQueryState {
            filter_column: "amount".to_owned(),
            filter_operator: UiTableFilterOperator::GreaterThan,
            filter_value: "10.00".to_owned(),
            ..Default::default()
        };

        assert_eq!(state.commit_filter_draft(Some(&table_info("numeric(12,2)"))), Ok(true));
        assert_eq!(state.filters.len(), 1);
        assert_eq!(state.filters[0].value, "10.00");

        state.filter_value = "not-a-number".to_owned();
        assert!(state.commit_filter_draft(Some(&table_info("numeric(12,2)"))).is_err());
        assert_eq!(state.filters[0].value, "10.00");
    }

    #[test]
    fn commit_filter_draft_replaces_the_selected_filter_and_normalizes_null_values() {
        let mut state = TableDataQueryState {
            filter_column: "amount".to_owned(),
            filter_operator: UiTableFilterOperator::Equals,
            filter_value: "10".to_owned(),
            ..Default::default()
        };
        assert_eq!(state.commit_filter_draft(Some(&table_info("integer"))), Ok(true));

        state.filter_editing = Some(0);
        state.filter_operator = UiTableFilterOperator::IsNull;
        state.filter_value = "ignored".to_owned();
        assert_eq!(state.commit_filter_draft(Some(&table_info("integer"))), Ok(true));
        assert_eq!(state.filters.len(), 1);
        assert_eq!(state.filters[0].value, "");
        assert_eq!(state.filters[0].operator, UiTableFilterOperator::IsNull);
    }

    #[test]
    fn removing_filters_keeps_edit_index_consistent() {
        let mut state = TableDataQueryState {
            filters: vec![
                UiTableDataFilter {
                    column: "a".to_owned(),
                    data_type: "text".to_owned(),
                    operator: UiTableFilterOperator::Equals,
                    value: "one".to_owned(),
                },
                UiTableDataFilter {
                    column: "b".to_owned(),
                    data_type: "text".to_owned(),
                    operator: UiTableFilterOperator::Equals,
                    value: "two".to_owned(),
                },
            ],
            filter_editing: Some(1),
            ..Default::default()
        };

        assert!(state.remove_filter(0));
        assert_eq!(state.filter_editing, Some(0));
        assert!(!state.remove_filter(9));
    }

    #[test]
    fn sort_transitions_preserve_single_and_additive_cycle_semantics() {
        let mut state = TableDataQueryState::default();

        state.set_sort(Some("amount".to_owned()), Some(false));
        state.cycle_sort("amount".to_owned(), false);
        assert!(state.sorts[0].descending);
        state.cycle_sort("amount".to_owned(), false);
        assert!(state.sorts.is_empty());

        state.cycle_sort("amount".to_owned(), true);
        state.cycle_sort("created_at".to_owned(), true);
        assert_eq!(state.sorts.len(), 2);
        state.cycle_sort("amount".to_owned(), true);
        assert!(state.sorts[0].descending);
    }