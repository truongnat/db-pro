//! Tests for `../query.rs` (moved out of the source file).
use super::*;

#[test]
fn query_result_validate_empty() {
    let result = QueryResult::empty();
    assert!(result.validate().is_ok());
}

#[test]
fn query_result_validate_matching_columns() {
    let result = QueryResult {
        columns: vec![
            ColumnMeta {
                name: "id".into(),
                data_type: "int".into(),
                nullable: false,
            },
            ColumnMeta {
                name: "name".into(),
                data_type: "text".into(),
                nullable: true,
            },
        ],
        rows: vec![Row(vec![CellValue::Int64(1), CellValue::Text("alice".into())])],
        row_count: 1,
        duration_ms: 0,
    };
    assert!(result.validate().is_ok());
}

#[test]
fn query_result_validate_cell_count_mismatch() {
    let result = QueryResult {
        columns: vec![
            ColumnMeta {
                name: "id".into(),
                data_type: "int".into(),
                nullable: false,
            },
            ColumnMeta {
                name: "name".into(),
                data_type: "text".into(),
                nullable: true,
            },
        ],
        rows: vec![Row(vec![CellValue::Int64(1)])],
        row_count: 1,
        duration_ms: 0,
    };
    assert!(result.validate().is_err());
}

#[test]
fn query_result_validate_row_count_mismatch() {
    let result = QueryResult {
        columns: vec![ColumnMeta {
            name: "id".into(),
            data_type: "int".into(),
            nullable: false,
        }],
        rows: vec![Row(vec![CellValue::Int64(1)])],
        row_count: 2,
        duration_ms: 0,
    };

    assert!(matches!(result.validate(), Err(message) if message.contains("row_count")));
}

#[test]
fn query_result_validate_affected_rows_without_columns() {
    let result = QueryResult {
        columns: Vec::new(),
        rows: Vec::new(),
        row_count: 3,
        duration_ms: 0,
    };

    assert!(result.validate().is_ok());
}

#[test]
fn query_result_validate_rejects_rows_without_columns() {
    let result = QueryResult {
        columns: Vec::new(),
        rows: vec![Row(Vec::new())],
        row_count: 1,
        duration_ms: 0,
    };

    assert!(matches!(result.validate(), Err(message) if message.contains("no columns")));
}

#[test]
fn int64_serializes_as_string_for_lossless_ipc() {
    let cell = CellValue::Int64(9_007_199_254_740_993); // 2^53 + 1
    let json = serde_json::to_string(&cell).unwrap();
    assert!(
        json.contains("\"9007199254740993\""),
        "i64 must serialize as string: {json}"
    );
    let back: CellValue = serde_json::from_str(&json).unwrap();
    assert!(matches!(back, CellValue::Int64(9_007_199_254_740_993)));
}

#[test]
fn int64_boundary_values_round_trip() {
    for &v in &[
        0i64,
        1,
        -1,
        i64::MAX,
        i64::MIN,
        (1i64 << 53) - 1,
        1i64 << 53,
        (1i64 << 53) + 1,
    ] {
        let cell = CellValue::Int64(v);
        let json = serde_json::to_string(&cell).unwrap();
        let back: CellValue = serde_json::from_str(&json).unwrap();
        assert!(
            matches!(back, CellValue::Int64(x) if x == v),
            "round-trip failed for {v}"
        );
    }
}

#[test]
fn query_param_int64_string_round_trip() {
    let param = QueryParam::Int64(i64::MAX);
    let json = serde_json::to_string(&param).unwrap();
    let back: QueryParam = serde_json::from_str(&json).unwrap();
    assert!(matches!(back, QueryParam::Int64(x) if x == i64::MAX));
}

/// A2 (#52): the five temporal classes are distinguishable in the domain
/// representation itself, not only by parsing the canonical string.
#[test]
fn temporal_classes_keep_their_own_tag_and_canonical_string() {
    let cases: [(CellValue, &str, &str); 5] = [
        (CellValue::Date("2024-03-15".into()), "date", "2024-03-15"),
        (CellValue::Time("10:20:30.123456".into()), "time", "10:20:30.123456"),
        (
            CellValue::TimeTz("10:20:30.123456+07:00".into()),
            "timetz",
            "10:20:30.123456+07:00",
        ),
        (
            CellValue::Timestamp("2024-03-15T10:20:30.123456".into()),
            "timestamp",
            "2024-03-15T10:20:30.123456",
        ),
        (
            CellValue::TimestampTz("2024-03-15T10:20:30.123456Z".into()),
            "timestamptz",
            "2024-03-15T10:20:30.123456Z",
        ),
    ];

    let mut tags = Vec::new();
    for (cell, expected_tag, expected_text) in cases {
        let json = serde_json::to_value(&cell).unwrap();
        assert_eq!(json["type"], expected_tag, "each temporal class keeps its own tag");
        assert_eq!(
            json["value"], expected_text,
            "the canonical string is preserved exactly"
        );
        let back: CellValue = serde_json::from_value(json).unwrap();
        assert_eq!(
            serde_json::to_value(&back).unwrap()["value"],
            expected_text,
            "a temporal cell round-trips through IPC JSON without reinterpretation"
        );
        tags.push(expected_tag);
    }
    assert_eq!(tags.len(), tags.iter().collect::<std::collections::HashSet<_>>().len());

    // The distinction a consumer would otherwise have to parse the string for:
    // only the instant carries a marker, and only the zoned time carries an offset.
    let naive = serde_json::to_value(CellValue::Timestamp("2024-03-15T10:20:30.123456".into())).unwrap();
    assert!(!naive["value"].as_str().unwrap().contains('Z'));
    assert!(!naive["value"].as_str().unwrap().contains('+'));
    let instant = serde_json::to_value(CellValue::TimestampTz("2024-03-15T10:20:30.123456Z".into())).unwrap();
    assert_ne!(naive, instant);
}

#[test]
fn typed_query_params_round_trip_through_ipc_json() {
    let params = vec![
        QueryParam::Time("12:34:56.123456".into()),
        QueryParam::Interval("1 days 02:00:00".into()),
        QueryParam::Inet("192.0.2.1/24".into()),
    ];
    let json = serde_json::to_string(&params).unwrap();
    let back: Vec<QueryParam> = serde_json::from_str(&json).unwrap();

    assert!(matches!(
        back.as_slice(),
        [
            QueryParam::Time(time),
            QueryParam::Interval(interval),
            QueryParam::Inet(address)
        ] if time == "12:34:56.123456"
            && interval == "1 days 02:00:00"
            && address == "192.0.2.1/24"
    ));
}
