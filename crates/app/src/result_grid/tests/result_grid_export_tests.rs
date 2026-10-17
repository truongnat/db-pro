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

    // #429: cells that Excel/LibreOffice would evaluate as formulas must be emitted as
    // `'`-prefixed literal text instead.
    #[test]
    fn delimited_export_escapes_formula_leading_cells() {
        assert_eq!(
            escape_delimited_field("=HYPERLINK(\"http://evil\",\"x\")", ','),
            "\"'=HYPERLINK(\"\"http://evil\"\",\"\"x\"\")\""
        );
        assert_eq!(escape_delimited_field("=1+1", ','), "'=1+1");
        assert_eq!(escape_delimited_field("+SUM(A1)", ','), "'+SUM(A1)");
        assert_eq!(escape_delimited_field("@SUM(1)", ','), "'@SUM(1)");
        assert_eq!(escape_delimited_field("\t=1", ','), "'\t=1");
        // A `-` value that parses as a number stays a number — otherwise every negative
        // export would silently become text.
        assert_eq!(escape_delimited_field("-42", ','), "-42");
        assert_eq!(escape_delimited_field("-1+1", ','), "'-1+1");
        assert_eq!(escape_delimited_field("plain", ','), "plain");
    }

    #[test]
    fn csv_header_and_number_cells_get_formula_guard() {
        let result = UiQueryResult {
            columns: vec![crate::UiColumn {
                name: "=evil".to_owned(),
                data_type: "text".to_owned(),
                nullable: false,
            }],
            rows: vec![vec![UiCell::Number("-1+1".to_owned())]],
            row_count: 1,
            duration_ms: 0,
        };
        assert_eq!(format_result_delimited(&result, ","), "'=evil\n'-1+1\n");
    }

    // #487: NULL must not collapse into the same field as `''` — the Postgres `\copy`
    // sentinel `\N` keeps the distinction through a delimited round-trip.
    #[test]
    fn delimited_cells_distinguish_null_from_empty_text() {
        assert_eq!(format_cell_delimited(&UiCell::Null, '\t'), "\\N");
        assert_eq!(format_cell_delimited(&UiCell::Text(String::new()), '\t'), "");
        assert_eq!(format_cell_delimited(&UiCell::Null, ','), "\\N");

        let result = UiQueryResult {
            columns: vec![
                crate::UiColumn { name: "a".to_owned(), data_type: "text".to_owned(), nullable: true },
                crate::UiColumn { name: "b".to_owned(), data_type: "text".to_owned(), nullable: true },
            ],
            rows: vec![vec![UiCell::Null, UiCell::Text(String::new())]],
            row_count: 1,
            duration_ms: 0,
        };
        assert_eq!(format_result_delimited(&result, "\t"), "a\tb\n\\N\t\n");
    }
