use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

use crate::domain::connection::ConnectionId;
use crate::domain::cross_connection::{DataColumnChange, DataDiff, DataRowDiff, DataRowState};
use crate::domain::error::DbError;
use crate::domain::query::{CellValue, QueryResult};
use crate::ports::DbConnector;

use super::registry::ConnectionRegistry;

const DEFAULT_SAMPLE_LIMIT: u64 = 1_000;
const MAX_SAMPLE_LIMIT: u64 = 10_000;
const MAX_ROW_DIFF_SAMPLES: usize = 200;

pub struct DataDiffService {
    connector: Box<dyn DbConnector>,
    registry: Arc<ConnectionRegistry>,
}

impl DataDiffService {
    pub fn new(connector: Box<dyn DbConnector>, registry: Arc<ConnectionRegistry>) -> Self {
        Self { connector, registry }
    }

    pub async fn diff_table_data(
        &self,
        source_id: &ConnectionId,
        target_id: &ConnectionId,
        schema: &str,
        table: &str,
    ) -> Result<DataDiff, DbError> {
        self.diff_table_data_keyed(source_id, target_id, schema, table, &[], None)
            .await
    }

    /// Key-aware bounded compare. Empty `key_columns` keeps row-count-only mode.
    pub async fn diff_table_data_keyed(
        &self,
        source_id: &ConnectionId,
        target_id: &ConnectionId,
        schema: &str,
        table: &str,
        key_columns: &[String],
        sample_limit: Option<u64>,
    ) -> Result<DataDiff, DbError> {
        let source_handle = self
            .registry
            .get(source_id)
            .ok_or_else(|| DbError::ConnectionFailed(format!("connection {source_id} is not active")))?;
        let target_handle = self
            .registry
            .get(target_id)
            .ok_or_else(|| DbError::ConnectionFailed(format!("connection {target_id} is not active")))?;

        let source_dialect = self.connector.dialect(&source_handle)?;
        let target_dialect = self.connector.dialect(&target_handle)?;

        let source_qualified = qualify(&*source_dialect, schema, table);
        let target_qualified = qualify(&*target_dialect, schema, table);

        let source_count_sql = format!("SELECT COUNT(*) FROM {source_qualified}");
        let target_count_sql = format!("SELECT COUNT(*) FROM {target_qualified}");
        let source_result = self.connector.query(&source_handle, &source_count_sql, &[]).await?;
        let target_result = self.connector.query(&target_handle, &target_count_sql, &[]).await?;
        source_result.validate().map_err(DbError::QueryFailed)?;
        target_result.validate().map_err(DbError::QueryFailed)?;
        let source_count = extract_count(&source_result)?;
        let target_count = extract_count(&target_result)?;
        let row_count_diff = row_count_difference(source_count, target_count)?;

        if key_columns.is_empty() {
            return Ok(DataDiff {
                schema: schema.to_string(),
                table: table.to_string(),
                source_row_count: source_count,
                target_row_count: target_count,
                row_count_diff,
                key_columns: Vec::new(),
                sample_limit: None,
                added: 0,
                removed: 0,
                changed: 0,
                equal: 0,
                truncated: false,
                row_diffs: Vec::new(),
                sync_sql_preview: Vec::new(),
            });
        }

        let limit = sample_limit.unwrap_or(DEFAULT_SAMPLE_LIMIT).clamp(1, MAX_SAMPLE_LIMIT);
        let order_source = key_columns
            .iter()
            .map(|c| source_dialect.quote_identifier(c))
            .collect::<Vec<_>>()
            .join(", ");
        let order_target = key_columns
            .iter()
            .map(|c| target_dialect.quote_identifier(c))
            .collect::<Vec<_>>()
            .join(", ");
        let source_sql = format!("SELECT * FROM {source_qualified} ORDER BY {order_source} LIMIT {limit}");
        let target_sql = format!("SELECT * FROM {target_qualified} ORDER BY {order_target} LIMIT {limit}");
        let source_rows = self.connector.query(&source_handle, &source_sql, &[]).await?;
        let target_rows = self.connector.query(&target_handle, &target_sql, &[]).await?;
        source_rows.validate().map_err(DbError::QueryFailed)?;
        target_rows.validate().map_err(DbError::QueryFailed)?;

        let source_map = rows_by_key(&source_rows, key_columns)?;
        let target_map = rows_by_key(&target_rows, key_columns)?;

        let mut added = 0u64;
        let mut removed = 0u64;
        let mut changed = 0u64;
        let mut equal = 0u64;
        let mut row_diffs = Vec::new();
        let mut sync_sql_preview = Vec::new();

        let all_keys: BTreeSet<_> = source_map.keys().chain(target_map.keys()).cloned().collect();
        for key in all_keys {
            match (source_map.get(&key), target_map.get(&key)) {
                (Some(src), None) => {
                    added += 1;
                    if row_diffs.len() < MAX_ROW_DIFF_SAMPLES {
                        row_diffs.push(DataRowDiff {
                            state: DataRowState::Added,
                            key: key.clone(),
                            column_changes: Vec::new(),
                        });
                    }
                    if sync_sql_preview.len() < 50 {
                        sync_sql_preview.push(sync_insert(&*target_dialect, &target_qualified, src));
                    }
                }
                (None, Some(_)) => {
                    removed += 1;
                    if row_diffs.len() < MAX_ROW_DIFF_SAMPLES {
                        row_diffs.push(DataRowDiff {
                            state: DataRowState::Removed,
                            key: key.clone(),
                            column_changes: Vec::new(),
                        });
                    }
                    if sync_sql_preview.len() < 50 {
                        sync_sql_preview.push(sync_delete(&*target_dialect, &target_qualified, key_columns, &key));
                    }
                }
                (Some(src), Some(tgt)) => {
                    let changes = column_changes(src, tgt);
                    if changes.is_empty() {
                        equal += 1;
                    } else {
                        changed += 1;
                        if row_diffs.len() < MAX_ROW_DIFF_SAMPLES {
                            row_diffs.push(DataRowDiff {
                                state: DataRowState::Changed,
                                key: key.clone(),
                                column_changes: changes.clone(),
                            });
                        }
                        if sync_sql_preview.len() < 50 {
                            if let Some(sql) =
                                sync_update(&*target_dialect, &target_qualified, key_columns, &key, &changes)
                            {
                                sync_sql_preview.push(sql);
                            }
                        }
                    }
                }
                (None, None) => {}
            }
        }

        let truncated = source_count as u64 > limit || target_count as u64 > limit;

        Ok(DataDiff {
            schema: schema.to_string(),
            table: table.to_string(),
            source_row_count: source_count,
            target_row_count: target_count,
            row_count_diff,
            key_columns: key_columns.to_vec(),
            sample_limit: Some(limit),
            added,
            removed,
            changed,
            equal,
            truncated,
            row_diffs,
            sync_sql_preview,
        })
    }
}

fn qualify(dialect: &dyn crate::ports::SqlDialect, schema: &str, table: &str) -> String {
    if schema.is_empty() {
        dialect.quote_identifier(table)
    } else {
        format!(
            "{}.{}",
            dialect.quote_identifier(schema),
            dialect.quote_identifier(table)
        )
    }
}

pub fn describe_sample_window(sample_limit: Option<u64>, truncated: bool) -> Option<String> {
    sample_limit.map(|limit| {
        if truncated {
            format!("Sample window: first {limit} rows ordered by key. Later rows were not compared.")
        } else {
            format!("Sample window: first {limit} rows ordered by key.")
        }
    })
}

fn sync_insert(dialect: &dyn crate::ports::SqlDialect, qualified: &str, row: &BTreeMap<String, String>) -> String {
    let columns = row
        .keys()
        .map(|column| dialect.quote_identifier(column))
        .collect::<Vec<_>>()
        .join(", ");
    let values = row
        .values()
        .map(|value| sql_literal(value))
        .collect::<Vec<_>>()
        .join(", ");
    format!("INSERT INTO {qualified} ({columns}) VALUES ({values});")
}

fn sync_delete(dialect: &dyn crate::ports::SqlDialect, qualified: &str, key_columns: &[String], key: &str) -> String {
    format!(
        "DELETE FROM {qualified} WHERE {};",
        key_predicate(dialect, key_columns, key)
    )
}

fn sync_update(
    dialect: &dyn crate::ports::SqlDialect,
    qualified: &str,
    key_columns: &[String],
    key: &str,
    changes: &[DataColumnChange],
) -> Option<String> {
    let assignments = changes
        .iter()
        .filter(|change| !key_columns.iter().any(|column| column == &change.column))
        .map(|change| {
            format!(
                "{} = {}",
                dialect.quote_identifier(&change.column),
                sql_literal(change.source_value.as_deref().unwrap_or("∅"))
            )
        })
        .collect::<Vec<_>>();
    if assignments.is_empty() {
        return None;
    }
    Some(format!(
        "UPDATE {qualified} SET {} WHERE {};",
        assignments.join(", "),
        key_predicate(dialect, key_columns, key)
    ))
}

fn key_predicate(dialect: &dyn crate::ports::SqlDialect, key_columns: &[String], key: &str) -> String {
    key_columns
        .iter()
        .zip(key.split('\u{1f}'))
        .map(|(column, value)| format!("{} = {}", dialect.quote_identifier(column), sql_literal(value)))
        .collect::<Vec<_>>()
        .join(" AND ")
}

fn sql_literal(value: &str) -> String {
    if value == "∅" {
        "NULL".into()
    } else {
        format!("'{}'", value.replace('\'', "''"))
    }
}

fn cell_display(cell: &CellValue) -> String {
    match cell {
        CellValue::Null => "∅".into(),
        CellValue::Text(s) => s.clone(),
        CellValue::Bool(b) => b.to_string(),
        CellValue::Int64(n) => n.to_string(),
        CellValue::Float64(n) => n.to_string(),
        other => format!("{other:?}"),
    }
}

fn rows_by_key(
    result: &QueryResult,
    key_columns: &[String],
) -> Result<BTreeMap<String, BTreeMap<String, String>>, DbError> {
    let mut key_indexes = Vec::new();
    for key in key_columns {
        let idx = result
            .columns
            .iter()
            .position(|c| c.name == *key)
            .ok_or_else(|| DbError::Validation(format!("key column `{key}` not found in result")))?;
        key_indexes.push(idx);
    }

    let mut map = BTreeMap::new();
    for row in &result.rows {
        let key = key_indexes
            .iter()
            .map(|idx| cell_display(row.0.get(*idx).unwrap_or(&CellValue::Null)))
            .collect::<Vec<_>>()
            .join("\u{1f}");
        let mut values = BTreeMap::new();
        for (col_idx, col) in result.columns.iter().enumerate() {
            values.insert(
                col.name.clone(),
                cell_display(row.0.get(col_idx).unwrap_or(&CellValue::Null)),
            );
        }
        if map.insert(key.clone(), values).is_some() {
            return Err(DbError::Validation(format!(
                "duplicate comparison key in sample: {key}"
            )));
        }
    }
    Ok(map)
}

fn column_changes(source: &BTreeMap<String, String>, target: &BTreeMap<String, String>) -> Vec<DataColumnChange> {
    let columns: BTreeSet<_> = source.keys().chain(target.keys()).cloned().collect();
    columns
        .into_iter()
        .filter_map(|column| {
            let src = source.get(&column).cloned();
            let tgt = target.get(&column).cloned();
            if src == tgt {
                None
            } else {
                Some(DataColumnChange {
                    column,
                    source_value: src,
                    target_value: tgt,
                })
            }
        })
        .collect()
}

fn extract_count(result: &QueryResult) -> Result<i64, DbError> {
    if result.columns.len() != 1 || result.rows.len() != 1 || result.row_count != 1 {
        return Err(DbError::Internal(
            "count query must return exactly one row and one column".into(),
        ));
    }

    let count = result.rows[0]
        .0
        .first()
        .and_then(|cell| match cell {
            CellValue::Int64(n) => Some(*n),
            _ => None,
        })
        .ok_or_else(|| DbError::Internal("failed to extract row count".into()))?;

    if count < 0 {
        return Err(DbError::Internal("row count cannot be negative".into()));
    }

    Ok(count)
}

fn row_count_difference(source: i64, target: i64) -> Result<i64, DbError> {
    source
        .checked_sub(target)
        .or_else(|| target.checked_sub(source).and_then(i64::checked_neg))
        .ok_or_else(|| DbError::Internal("row count difference overflowed".into()))
}

#[cfg(test)]
mod tests {
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
}
