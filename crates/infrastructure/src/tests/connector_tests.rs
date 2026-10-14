//! Tests for `../connector.rs` (moved out of the source file).
use super::dialect_for;
use db_pro_core::application::sql_builder::{build_select, FilterOp, SortClause, SortDir, TableFilter};
use db_pro_core::domain::connection::DriverType;
use db_pro_core::domain::query::CellValue;

fn like_name() -> Vec<TableFilter> {
    vec![TableFilter {
        column: "name".into(),
        op: FilterOp::Like,
        value: CellValue::Text("%ada%".into()),
    }]
}

/// Contains, starts-with, and ends-with all become `FilterOp::Like` before this builder.
#[test]
fn like_filter_uses_the_connection_dialect() {
    let filters = like_name();
    let (mysql, _) = build_select(
        dialect_for(DriverType::Mysql).as_ref(),
        "app",
        "users",
        &filters,
        &[],
        10,
        0,
    )
    .expect("mysql select");
    assert!(mysql.contains("CAST(`name` AS CHAR) LIKE ?"), "{mysql}");
    assert!(!mysql.contains("AS TEXT"), "{mysql}");

    let sorts = vec![SortClause {
        column: "id".into(),
        direction: SortDir::Asc,
    }];
    let (server, _) = build_select(
        dialect_for(DriverType::SqlServer).as_ref(),
        "dbo",
        "users",
        &filters,
        &sorts,
        10,
        0,
    )
    .expect("sql server select");
    assert!(server.contains("CAST([name] AS NVARCHAR(MAX)) LIKE @p1"), "{server}");
    assert!(!server.contains("AS TEXT"), "{server}");

    let (postgres, _) = build_select(
        dialect_for(DriverType::Postgres).as_ref(),
        "public",
        "users",
        &filters,
        &[],
        10,
        0,
    )
    .expect("postgres select");
    assert!(postgres.contains(r#"CAST("name" AS TEXT) LIKE $1"#), "{postgres}");

    let (sqlite, _) = build_select(
        dialect_for(DriverType::SQLite).as_ref(),
        "main",
        "users",
        &filters,
        &[],
        10,
        0,
    )
    .expect("sqlite select");
    assert!(sqlite.contains(r#"CAST("name" AS TEXT) LIKE ?"#), "{sqlite}");
}
