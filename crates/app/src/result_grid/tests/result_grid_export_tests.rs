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