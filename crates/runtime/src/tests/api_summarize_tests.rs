//! Tests for `../api.rs` (moved out of the source file).
use super::summarize_introspection;
use db_pro_core::domain::schema::{Column, ForeignKey, IntrospectResult, Table};

#[test]
// cc-scan:allow HUGE_FUNCTION — one linear arrange-act-assert scenario; splitting it fragments the case under test
fn columns_stay_with_their_table_without_a_cross_product() {
    let result = IntrospectResult {
        schemas: vec![],
        tables: vec![
            Table {
                name: "orders".into(),
                schema: "sales".into(),
                row_count: Some(3),
            },
            Table {
                name: "orders".into(),
                schema: "archive".into(),
                row_count: None,
            },
        ],
        columns: vec![
            Column {
                name: "id".into(),
                data_type: "int".into(),
                table_name: "orders".into(),
                schema: "sales".into(),
                is_primary_key: true,
                ..Column::default()
            },
            Column {
                name: "id".into(),
                data_type: "int".into(),
                table_name: "orders".into(),
                schema: "archive".into(),
                ..Column::default()
            },
            Column {
                name: "total".into(),
                data_type: "numeric".into(),
                table_name: "orders".into(),
                schema: "sales".into(),
                ..Column::default()
            },
        ],
        foreign_keys: vec![ForeignKey {
            name: "orders_customer".into(),
            from_table: "orders".into(),
            from_columns: vec!["customer_id".into()],
            to_table: "customers".into(),
            to_columns: vec!["id".into()],
            schema: "sales".into(),
            to_schema: "sales".into(),
            ..ForeignKey::default()
        }],
        ..IntrospectResult::empty()
    };

    let summary = summarize_introspection(result);
    let sales = summary
        .table_details
        .iter()
        .find(|table| table.schema == "sales")
        .unwrap();
    assert_eq!(
        sales
            .columns
            .iter()
            .map(|column| column.name.as_str())
            .collect::<Vec<_>>(),
        vec!["id", "total"]
    );
    assert_eq!(sales.foreign_keys.len(), 1);
    let archive = summary
        .table_details
        .iter()
        .find(|table| table.schema == "archive")
        .unwrap();
    assert_eq!(archive.columns.len(), 1);
    assert!(archive.foreign_keys.is_empty());
}
