//! Tests for `../table_data_toolbar_surface_view.rs` (moved out of the source file).
    use super::{
        condition_column_candidates, condition_identifier, condition_should_suggest_automatically,
        condition_token_range, select_prefix, select_with_condition,
    };
    use crate::UiTableColumn;

    fn columns() -> Vec<UiTableColumn> {
        vec![
            UiTableColumn {
                name: "first_name".into(),
                data_type: "TEXT".into(),
                ..Default::default()
            },
            UiTableColumn {
                name: "id".into(),
                data_type: "INTEGER".into(),
                is_primary_key: true,
                ..Default::default()
            },
        ]
    }

    #[test]
    fn select_prefix_quotes_schema_and_table_identifiers() {
        assert_eq!(
            select_prefix(Some("main"), "customers"),
            "SELECT * FROM \"main\".\"customers\""
        );
        assert_eq!(
            select_prefix(None, "customer\"data"),
            "SELECT * FROM \"customer\"\"data\""
        );
    }

    #[test]
    fn empty_condition_selects_all_without_a_where_clause() {
        let prefix = select_prefix(Some("main"), "customers");
        for condition in ["", " ", "\t\n"] {
            assert_eq!(select_with_condition(&prefix, condition), "SELECT * FROM \"main\".\"customers\"");
        }
        assert_eq!(select_with_condition(&prefix, " id = 2 "), "SELECT * FROM \"main\".\"customers\" WHERE id = 2");
    }

    #[test]
    fn condition_suggestions_are_scoped_to_current_table_and_prefix() {
        let columns = columns();
        let suggestions = condition_column_candidates("first", 5, &columns);
        assert_eq!(
            suggestions
                .iter()
                .map(|column| column.name.as_str())
                .collect::<Vec<_>>(),
            ["first_name"]
        );
        assert_eq!(condition_column_candidates("", 0, &columns).len(), 2);
        assert!(condition_column_candidates("id", 2, &columns).is_empty());
    }

    #[test]
    fn condition_suggestions_wait_for_a_prefix_or_a_new_condition() {
        assert!(!condition_should_suggest_automatically("", 0));
        assert!(!condition_should_suggest_automatically("id = ", 6));
        assert!(condition_should_suggest_automatically("fir", 3));
        assert!(condition_should_suggest_automatically("first_name AND ", 15));
    }

    #[test]
    fn condition_suggestions_skip_values_and_sql_string_literals() {
        let columns = columns();
        assert!(condition_column_candidates("id = Al", 7, &columns).is_empty());
        assert!(condition_column_candidates("first_name = 'Al'", 15, &columns).is_empty());
        assert!(condition_column_candidates("\"first", 6, &columns).is_empty());
        assert_eq!(condition_column_candidates("first_name AND ", 15, &columns).len(), 2);
    }

    #[test]
    fn condition_token_range_replaces_identifier_suffix_at_middle_cursor() {
        let text = "first_namX = 1";
        assert_eq!(condition_token_range(text, 9), Some((0, 10)));
    }

    #[test]
    fn condition_identifier_keeps_common_names_readable_and_quotes_unsafe_names() {
        assert_eq!(condition_identifier("first_name"), "first_name");
        assert_eq!(condition_identifier("order"), "\"order\"");
        assert_eq!(condition_identifier("display name"), "\"display name\"");
    }