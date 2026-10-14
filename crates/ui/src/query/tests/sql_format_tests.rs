//! Tests for `../sql_format.rs` (moved out of the source file).
use super::*;

#[test]
fn formats_clause_boundaries_without_rewriting_literals_or_comments() {
    let sql = "select 'select from', note from users -- where stays\nwhere id = 1";
    let formatted = format_sql_for_dialect(sql, SqlDialect::Postgres);

    assert!(formatted.contains("'select from'"));
    assert!(formatted.contains("-- where stays"));
    assert!(formatted.contains("SELECT 'select from', note\nFROM users"));
    assert!(formatted.ends_with("\nWHERE id = 1"));
}

#[test]
fn preserves_dollar_quoted_postgres_body() {
    let sql = "select $tag$from where select$tag$ from functions";
    let formatted = format_sql_for_dialect(sql, SqlDialect::Postgres);

    assert!(formatted.contains("$tag$from where select$tag$"));
    assert!(formatted.ends_with("\nFROM functions"));
}

#[test]
fn formats_incomplete_sql_without_panicking() {
    assert_eq!(format_sql("select * from"), "SELECT *\nFROM");
}
