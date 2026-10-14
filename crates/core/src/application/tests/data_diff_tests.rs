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
    src.insert("a".into(), "1".into());
    src.insert("b".into(), "∅".into());
    let mut tgt = BTreeMap::new();
    tgt.insert("a".into(), "2".into());
    tgt.insert("b".into(), "∅".into());
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
