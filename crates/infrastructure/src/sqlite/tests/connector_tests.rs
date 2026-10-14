//! Tests for `../connector.rs` (moved out of the source file).
use super::SqliteDialect;
use db_pro_core::application::sql_builder::{build_select, SortClause, SortDir};

#[test]
fn table_pagination_uses_sqlite_placeholders() {
    let (sql, params) = build_select(
        &SqliteDialect,
        "main",
        "customers",
        &[],
        &[SortClause {
            column: "id".to_owned(),
            direction: SortDir::Desc,
        }],
        100,
        200,
    )
    .expect("SQLite pagination should build");

    assert_eq!(
        sql,
        r#"SELECT * FROM "main"."customers" ORDER BY "id" DESC LIMIT ? OFFSET ?"#
    );
    assert_eq!(params.len(), 2);
}
