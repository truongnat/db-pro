//! Tests for `../data_diff.rs` (moved out of the source file).
use super::*;
use crate::domain::query::{CellValue, ColumnMeta, QueryResult, Row};

#[test]
fn extract_count_rejects_negative_provider_value() {
    let result = QueryResult {
        columns: vec![ColumnMeta {
            name: "count".into(),
            data_type: "INT".into(),
            nullable: false,
        }],
        rows: vec![Row(vec![CellValue::Int64(-1)])],
        row_count: 1,
        duration_ms: 0,
    };

    assert!(matches!(
        extract_count(&result),
        Err(DbError::Internal(message)) if message.contains("negative")
    ));
}

#[test]
fn duplicate_keys_are_rejected() {
    let result = QueryResult {
        columns: vec![
            ColumnMeta {
                name: "id".into(),
                data_type: "INT".into(),
                nullable: false,
            },
            ColumnMeta {
                name: "v".into(),
                data_type: "TEXT".into(),
                nullable: true,
            },
        ],
        rows: vec![
            Row(vec![CellValue::Int64(1), CellValue::Text("a".into())]),
            Row(vec![CellValue::Int64(1), CellValue::Text("b".into())]),
        ],
        row_count: 2,
        duration_ms: 0,
    };
    let err = rows_by_key(&result, &["id".into()]).expect_err("dup");
    assert!(matches!(err, DbError::Validation(message) if message.contains("duplicate")));
}

#[test]
fn column_changes_detect_null_and_value_diffs() {
    let mut src = BTreeMap::new();
    src.insert("a", DiffValue::Int64(1));
    src.insert("b", DiffValue::Null);
    let mut tgt = BTreeMap::new();
    tgt.insert("a", DiffValue::Int64(2));
    tgt.insert("b", DiffValue::Null);
    let changes = column_changes(&src, &tgt);
    assert_eq!(changes.len(), 1);
    assert_eq!(changes[0].column, "a");
}

#[test]
fn row_count_difference_handles_both_directions() {
    assert_eq!(row_count_difference(10, 3).unwrap(), 7);
    assert_eq!(row_count_difference(3, 10).unwrap(), -7);
}

fn count_result(count: i64) -> QueryResult {
    QueryResult {
        columns: vec![ColumnMeta {
            name: "count".into(),
            data_type: "bigint".into(),
            nullable: false,
        }],
        rows: vec![Row(vec![CellValue::Int64(count)])],
        row_count: 1,
        duration_ms: 0,
    }
}

fn table_result(rows: Vec<(i64, &str)>) -> QueryResult {
    QueryResult {
        columns: vec![
            ColumnMeta {
                name: "id".into(),
                data_type: "bigint".into(),
                nullable: false,
            },
            ColumnMeta {
                name: "name".into(),
                data_type: "text".into(),
                nullable: false,
            },
        ],
        row_count: rows.len() as u64,
        rows: rows
            .into_iter()
            .map(|(id, name)| Row(vec![CellValue::Int64(id), CellValue::Text(name.to_owned())]))
            .collect(),
        duration_ms: 0,
    }
}

#[tokio::test]
async fn keyed_sample_states_its_window_and_sync_sql_runs() {
    use crate::application::driver_dialect::DriverDialect;
    use crate::domain::connection::{ConnectionHandle, ConnectionId};
    use crate::ports::MockDbConnector;

    let source_id = ConnectionId::new();
    let target_id = ConnectionId::new();
    let registry = Arc::new(ConnectionRegistry::new());
    registry.register(source_id, ConnectionHandle(1));
    registry.register(target_id, ConnectionHandle(2));

    let mut connector = MockDbConnector::new();
    connector
        .expect_dialect()
        .returning(|_| Ok(Box::new(DriverDialect::from_name("postgresql"))));
    connector.expect_query().returning(|handle, sql, _| {
        if sql.contains("COUNT(*)") {
            Ok(count_result(2))
        } else if handle.0 == 1 {
            Ok(table_result(vec![(1, "Alice"), (2, "Bob")]))
        } else {
            Ok(table_result(vec![(1, "Ann"), (3, "Cara")]))
        }
    });

    let service = DataDiffService::new(Box::new(connector), registry);
    let diff = service
        .diff_table_data_keyed(&source_id, &target_id, "public", "orders", &["id".into()], Some(1_000))
        .await
        .expect("diff");

    let window = describe_sample_window(diff.sample_limit, diff.truncated).expect("window");
    assert!(window.contains("Sample window"), "{window}");
    assert!(window.contains("1000"), "{window}");
    assert!(!diff.truncated);
    assert!(!diff.sync_sql_preview.is_empty());
    for sql in &diff.sync_sql_preview {
        assert!(!sql.trim_start().starts_with("--"), "{sql}");
        let statements = crate::domain::safety::split_statements(sql);
        assert_eq!(statements.len(), 1, "{sql}");
        let verb = statements[0].split_whitespace().next().unwrap().to_ascii_uppercase();
        assert!(verb == "INSERT" || verb == "UPDATE" || verb == "DELETE", "{sql}");
    }
    let joined = diff.sync_sql_preview.join("\n");
    assert!(joined.contains("INSERT INTO") && joined.contains("Bob"), "{joined}");
    assert!(joined.contains("UPDATE") && joined.contains("Alice"), "{joined}");
    assert!(joined.contains("DELETE FROM"), "{joined}");
}

fn value_result(columns: &[&str], rows: Vec<Vec<CellValue>>) -> QueryResult {
    QueryResult {
        columns: columns
            .iter()
            .map(|name| ColumnMeta {
                name: (*name).into(),
                data_type: "text".into(),
                nullable: true,
            })
            .collect(),
        row_count: rows.len() as u64,
        rows: rows.into_iter().map(Row).collect(),
        duration_ms: 0,
    }
}

async fn compare_values(source: QueryResult, target: QueryResult, keys: &[&str]) -> DataDiff {
    use crate::application::driver_dialect::DriverDialect;
    use crate::domain::connection::{ConnectionHandle, ConnectionId};
    use crate::ports::MockDbConnector;

    let source_id = ConnectionId::new();
    let target_id = ConnectionId::new();
    let registry = Arc::new(ConnectionRegistry::new());
    registry.register(source_id, ConnectionHandle(1));
    registry.register(target_id, ConnectionHandle(2));
    let mut connector = MockDbConnector::new();
    connector
        .expect_dialect()
        .returning(|_| Ok(Box::new(DriverDialect::from_name("postgresql"))));
    connector.expect_query().returning(move |handle, sql, _| {
        let result = if handle.0 == 1 { &source } else { &target };
        if sql.contains("COUNT(*)") {
            Ok(count_result(result.row_count as i64))
        } else {
            Ok(result.clone())
        }
    });
    DataDiffService::new(Box::new(connector), registry)
        .diff_table_data_keyed(
            &source_id,
            &target_id,
            "public",
            "orders",
            &keys.iter().map(|key| (*key).into()).collect::<Vec<_>>(),
            None,
        )
        .await
        .expect("typed diff")
}

#[tokio::test]
async fn null_source_and_empty_set_text_target_are_changed() {
    let diff = compare_values(
        value_result(&["id", "v"], vec![vec![CellValue::Int64(1), CellValue::Null]]),
        value_result(
            &["id", "v"],
            vec![vec![CellValue::Int64(1), CellValue::Text("∅".into())]],
        ),
        &["id"],
    )
    .await;
    assert_eq!((diff.changed, diff.equal), (1, 0));
    assert_eq!(diff.row_diffs[0].column_changes[0].source_value, None);
    assert_eq!(diff.row_diffs[0].column_changes[0].target_value.as_deref(), Some("∅"));
    assert_eq!(
        diff.sync_sql_preview,
        vec!["UPDATE \"public\".\"orders\" SET \"v\" = NULL WHERE \"id\" = '1';"]
    );
}

#[tokio::test]
async fn null_and_empty_set_text_keys_remain_distinct() {
    let rows = vec![
        vec![CellValue::Null, CellValue::Text("null key".into())],
        vec![CellValue::Text("∅".into()), CellValue::Text("text key".into())],
    ];
    let diff = compare_values(
        value_result(&["id", "v"], rows.clone()),
        value_result(&["id", "v"], rows),
        &["id"],
    )
    .await;
    assert_eq!((diff.added, diff.removed, diff.changed, diff.equal), (0, 0, 0, 2));
    assert!(diff.sync_sql_preview.is_empty());

    let diff = compare_values(
        value_result(&["id", "v"], vec![]),
        value_result(
            &["id", "v"],
            vec![
                vec![CellValue::Null, CellValue::Text("null key".into())],
                vec![CellValue::Text("∅".into()), CellValue::Text("text key".into())],
            ],
        ),
        &["id"],
    )
    .await;
    assert_eq!(diff.removed, 2);
    assert!(diff
        .sync_sql_preview
        .contains(&"DELETE FROM \"public\".\"orders\" WHERE \"id\" IS NULL;".into()));
    assert!(diff
        .sync_sql_preview
        .contains(&"DELETE FROM \"public\".\"orders\" WHERE \"id\" = '∅';".into()));
    assert_ne!(diff.row_diffs[0].key, diff.row_diffs[1].key);
}

#[tokio::test]
async fn empty_set_text_survives_insert_and_null_key_update() {
    let diff = compare_values(
        value_result(
            &["id", "v"],
            vec![
                vec![CellValue::Null, CellValue::Text("∅".into())],
                vec![CellValue::Text("∅".into()), CellValue::Text("∅".into())],
            ],
        ),
        value_result(&["id", "v"], vec![vec![CellValue::Null, CellValue::Null]]),
        &["id"],
    )
    .await;
    assert_eq!((diff.added, diff.changed), (1, 1));
    assert!(diff
        .sync_sql_preview
        .contains(&"UPDATE \"public\".\"orders\" SET \"v\" = '∅' WHERE \"id\" IS NULL;".into()));
    assert!(diff
        .sync_sql_preview
        .contains(&"INSERT INTO \"public\".\"orders\" (\"id\", \"v\") VALUES ('∅', '∅');".into()));
}

#[tokio::test]
async fn composite_separator_values_do_not_collide_or_change_predicate_boundaries() {
    let source_rows = vec![
        vec![
            CellValue::Text("a\u{1f}b".into()),
            CellValue::Text("c".into()),
            CellValue::Text("first".into()),
        ],
        vec![
            CellValue::Text("a".into()),
            CellValue::Text("b\u{1f}c".into()),
            CellValue::Text("second".into()),
        ],
    ];
    let target_rows = vec![
        vec![
            CellValue::Text("a\u{1f}b".into()),
            CellValue::Text("c".into()),
            CellValue::Text("old first".into()),
        ],
        vec![
            CellValue::Text("a".into()),
            CellValue::Text("b\u{1f}c".into()),
            CellValue::Text("old second".into()),
        ],
    ];
    let diff = compare_values(
        value_result(&["left", "right", "v"], source_rows),
        value_result(&["left", "right", "v"], target_rows),
        &["left", "right"],
    )
    .await;
    assert_eq!((diff.changed, diff.equal), (2, 0));
    assert!(diff.sync_sql_preview.contains(
        &"UPDATE \"public\".\"orders\" SET \"v\" = 'first' WHERE \"left\" = 'a\u{1f}b' AND \"right\" = 'c';".into()
    ));
    assert!(diff.sync_sql_preview.contains(
        &"UPDATE \"public\".\"orders\" SET \"v\" = 'second' WHERE \"left\" = 'a' AND \"right\" = 'b\u{1f}c';".into()
    ));
    assert_ne!(diff.row_diffs[0].key, diff.row_diffs[1].key);
}

#[tokio::test]
async fn composite_separator_tuple_difference_is_added_and_removed() {
    let diff = compare_values(
        value_result(
            &["left", "right"],
            vec![vec![CellValue::Text("a\u{1f}b".into()), CellValue::Text("c".into())]],
        ),
        value_result(
            &["left", "right"],
            vec![vec![CellValue::Text("a".into()), CellValue::Text("b\u{1f}c".into())]],
        ),
        &["left", "right"],
    )
    .await;
    assert_eq!((diff.added, diff.removed, diff.changed, diff.equal), (1, 1, 0, 0));
    assert!(diff
        .sync_sql_preview
        .contains(&"DELETE FROM \"public\".\"orders\" WHERE \"left\" = 'a' AND \"right\" = 'b\u{1f}c';".into()));
}

#[tokio::test]
async fn typed_values_preserve_precision_and_escape_sql_text() {
    let diff = compare_values(
        value_result(
            &["id", "amount", "note"],
            vec![vec![
                CellValue::Int64(i64::MAX),
                CellValue::Decimal("12345678901234567890.1234567890".into()),
                CellValue::Text("x'); DELETE FROM orders; --".into()),
            ]],
        ),
        value_result(&["id", "amount", "note"], vec![]),
        &["id"],
    )
    .await;
    assert_eq!(
        diff.sync_sql_preview,
        vec!["INSERT INTO \"public\".\"orders\" (\"amount\", \"id\", \"note\") VALUES ('12345678901234567890.1234567890', '9223372036854775807', 'x''); DELETE FROM orders; --');"]
    );
    assert_eq!(
        crate::domain::safety::split_statements(&diff.sync_sql_preview[0]).len(),
        1
    );
}
