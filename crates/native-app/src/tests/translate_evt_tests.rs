//! Tests for `../translate_evt.rs` (moved out of the source file).
use super::*;

/// One representative value per domain class, mapped through `map_cell`: the
/// UI class is part of the shipping (non-serde) channel contract, so
/// precision-sensitive values stay exact text, a null stays `Null`, bytes stay
/// byte-safe hex and JSON stays structured text (Gate 5 A4).
#[test]
// cc-scan:allow LONG_FUNCTION — linear pipeline — one cohesive pass
fn map_cell_keeps_one_ui_class_per_domain_value_class() {
    let cases = vec![
        (CellValue::Null, UiCell::Null),
        (CellValue::Bool(true), UiCell::Boolean(true)),
        (
            CellValue::Int64(9_007_199_254_740_993),
            UiCell::Number("9007199254740993".to_owned()),
        ),
        (CellValue::Float64(0.5), UiCell::Number("0.5".to_owned())),
        (
            CellValue::Decimal("12345678901234567890.12345".to_owned()),
            UiCell::Number("12345678901234567890.12345".to_owned()),
        ),
        (CellValue::Text("hello".to_owned()), UiCell::Text("hello".to_owned())),
        (
            CellValue::Uuid("3f2504e0-4f89-11d3-9a0c-0305e82c3301".to_owned()),
            UiCell::Text("3f2504e0-4f89-11d3-9a0c-0305e82c3301".to_owned()),
        ),
        (
            CellValue::DateTime("2024-03-15T10:20:30.123456".to_owned()),
            UiCell::Text("2024-03-15T10:20:30.123456".to_owned()),
        ),
        (
            CellValue::Timestamp("2024-03-15T10:20:30.123456".to_owned()),
            UiCell::Text("2024-03-15T10:20:30.123456".to_owned()),
        ),
        (
            CellValue::TimestampTz("2024-03-15T10:20:30.123456Z".to_owned()),
            UiCell::Text("2024-03-15T10:20:30.123456Z".to_owned()),
        ),
        (
            CellValue::TimeTz("10:20:30.123456+07:00".to_owned()),
            UiCell::Text("10:20:30.123456+07:00".to_owned()),
        ),
        (
            CellValue::Date("2024-03-15".to_owned()),
            UiCell::Text("2024-03-15".to_owned()),
        ),
        (
            CellValue::Time("10:20:30.123456".to_owned()),
            UiCell::Text("10:20:30.123456".to_owned()),
        ),
        (
            CellValue::Interval("1 mons 2 days 03:04:05.000006".to_owned()),
            UiCell::Text("1 mons 2 days 03:04:05.000006".to_owned()),
        ),
        (
            CellValue::Inet("192.168.0.1/24".to_owned()),
            UiCell::Text("192.168.0.1/24".to_owned()),
        ),
        (
            CellValue::Bytes(vec![0x00, 0xff, 0x10]),
            UiCell::Bytes("\\x00ff10".to_owned()),
        ),
        (
            CellValue::Json(serde_json::json!({ "a": 1 })),
            UiCell::Json("{\"a\":1}".to_owned()),
        ),
    ];

    for (domain, expected) in cases {
        assert_eq!(
            map_cell(domain.clone()),
            expected,
            "the UI class for {domain:?} changed"
        );
    }
}

#[test]
fn test_ui_cell_to_domain_datetime_and_date() {
    let rfc3339 = "2026-09-12T09:50:00Z".to_string();
    assert!(matches!(
        ui_cell_to_domain(UiCell::Text(rfc3339.clone())),
        Some(CellValue::DateTime(v)) if v == rfc3339
    ));

    let standard_dt = "2026-09-12 16:50:00".to_string();
    assert!(matches!(
        ui_cell_to_domain(UiCell::Text(standard_dt.clone())),
        Some(CellValue::DateTime(v)) if v == standard_dt
    ));

    let date_only = "2026-09-12".to_string();
    assert!(matches!(
        ui_cell_to_domain(UiCell::Text(date_only.clone())),
        Some(CellValue::Date(v)) if v == date_only
    ));

    let time_only = "16:50:00".to_string();
    assert!(matches!(
        ui_cell_to_domain(UiCell::Text(time_only.clone())),
        Some(CellValue::Time(v)) if v == time_only
    ));

    let uuid_str = "550e8400-e29b-41d4-a716-446655440000".to_string();
    assert!(matches!(
        ui_cell_to_domain(UiCell::Text(uuid_str.clone())),
        Some(CellValue::Uuid(v)) if v == uuid_str
    ));
}

#[test]
fn table_filter_operator_maps_to_parameterized_sql_filter_kind() {
    let filter = map_table_data_filter(UiTableDataFilter {
        column: "amount".to_owned(),
        data_type: "BIGINT".to_owned(),
        operator: UiTableFilterOperator::GreaterThanOrEqual,
        value: "100".to_owned(),
    });
    assert!(matches!(filter.op, FilterOp::Gte));
    assert!(matches!(filter.value, CellValue::Int64(100)));

    let decimal_filter = map_table_data_filter(UiTableDataFilter {
        column: "price".to_owned(),
        data_type: "NUMERIC(20,4)".to_owned(),
        operator: UiTableFilterOperator::Equals,
        value: "1234567890123456.1234".to_owned(),
    });
    assert!(matches!(decimal_filter.value, CellValue::Decimal(value) if value == "1234567890123456.1234"));

    let null_filter = map_table_data_filter(UiTableDataFilter {
        column: "deleted_at".to_owned(),
        data_type: "TIMESTAMPTZ".to_owned(),
        operator: UiTableFilterOperator::IsNull,
        value: String::new(),
    });
    assert!(matches!(null_filter.op, FilterOp::IsNull));
    assert!(matches!(null_filter.value, CellValue::Null));
}

#[test]
fn map_table_info_preserves_check_constraints_for_native_ui() {
    use db_pro_core::domain::schema::{CheckConstraint, Table, TableInfo};

    let ui = map_table_info(TableInfo {
        table: Table {
            name: "orders".to_owned(),
            schema: "public".to_owned(),
            row_count: Some(3),
        },
        columns: Vec::new(),
        primary_key: None,
        indexes: Vec::new(),
        foreign_keys: Vec::new(),
        check_constraints: vec![CheckConstraint {
            name: "orders_total_positive".to_owned(),
            table_name: "orders".to_owned(),
            schema: "public".to_owned(),
            definition: "CHECK (total >= 0)".to_owned(),
        }],
        dependencies: Vec::new(),
    });

    assert_eq!(ui.check_constraints.len(), 1);
    assert_eq!(ui.check_constraints[0].name, "orders_total_positive");
    assert_eq!(ui.check_constraints[0].definition, "CHECK (total >= 0)");
}
