//! Tests for `../result_grid_export.rs` (moved out of the source file).
    use super::*;

    #[test]
    fn quote_sql_identifier_escapes_embedded_quotes() {
        assert_eq!(quote_sql_identifier("customers"), "\"customers\"");
        assert_eq!(quote_sql_identifier("we\"ird"), "\"we\"\"ird\"");
    }

    #[test]
    fn qualified_sql_name_quotes_each_part() {
        assert_eq!(
            qualified_sql_name("main", "customers"),
            "\"main\".\"customers\""
        );
        assert_eq!(qualified_sql_name("", "customers"), "\"customers\"");
    }

    #[test]
    fn qualified_sql_name_preserves_literal_dot_in_table_name() {
        assert_eq!(
            qualified_sql_name("main", "HumanResources.Department"),
            "\"main\".\"HumanResources.Department\""
        );
    }

    #[test]
    fn qualified_sql_name_escapes_quotes_in_both_parts() {
        assert_eq!(
            qualified_sql_name("sch\"ema", "ta\"ble"),
            "\"sch\"\"ema\".\"ta\"\"ble\""
        );
    }

    #[test]
    fn unique_json_keys_suffixes_duplicate_column_names() {
        let columns = vec![
            crate::UiColumn { name: "id".to_owned(), data_type: "int".to_owned(), nullable: false },
            crate::UiColumn { name: "id".to_owned(), data_type: "int".to_owned(), nullable: false },
            crate::UiColumn { name: "id".to_owned(), data_type: "text".to_owned(), nullable: true },
            crate::UiColumn { name: "name".to_owned(), data_type: "text".to_owned(), nullable: false },
        ];
        assert_eq!(unique_json_keys(&columns), vec!["id", "id_2", "id_3", "name"]);
    }

    #[test]
    fn format_result_json_keeps_all_duplicate_named_columns() {
        let result = UiQueryResult {
            columns: vec![
                crate::UiColumn { name: "x".to_owned(), data_type: "int".to_owned(), nullable: false },
                crate::UiColumn { name: "x".to_owned(), data_type: "int".to_owned(), nullable: false },
            ],
            rows: vec![vec![UiCell::Number("1".to_owned()), UiCell::Number("2".to_owned())]],
            row_count: 1,
            duration_ms: 0,
        };
        let json = format_result_json(&result);
        assert!(json.contains("\"x\": 1"), "{json}");
        assert!(json.contains("\"x_2\": 2"), "{json}");
    }
