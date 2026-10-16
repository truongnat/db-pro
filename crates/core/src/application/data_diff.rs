use std::collections::{btree_map::Entry, BTreeMap, BTreeSet};
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

/// Values retain their provider-reported kind; display text is never identity.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
enum DiffValue<'a> {
    Null,
    Bool(bool),
    Int64(i64),
    Float64(u64),
    Decimal(&'a str),
    Text(&'a str),
    Bytes(&'a [u8]),
    Uuid(&'a str),
    DateTime(&'a str),
    Timestamp(&'a str),
    TimestampTz(&'a str),
    TimeTz(&'a str),
    Date(&'a str),
    Time(&'a str),
    Interval(&'a str),
    Inet(&'a str),
    Json(String),
}

impl<'a> From<&'a CellValue> for DiffValue<'a> {
    fn from(value: &'a CellValue) -> Self {
        match value {
            CellValue::Null => Self::Null,
            CellValue::Bool(value) => Self::Bool(*value),
            CellValue::Int64(value) => Self::Int64(*value),
            CellValue::Float64(value) => Self::Float64(value.to_bits()),
            CellValue::Decimal(value) => Self::Decimal(value),
            CellValue::Text(value) => Self::Text(value),
            CellValue::Bytes(value) => Self::Bytes(value),
            CellValue::Uuid(value) => Self::Uuid(value),
            CellValue::DateTime(value) => Self::DateTime(value),
            CellValue::Timestamp(value) => Self::Timestamp(value),
            CellValue::TimestampTz(value) => Self::TimestampTz(value),
            CellValue::TimeTz(value) => Self::TimeTz(value),
            CellValue::Date(value) => Self::Date(value),
            CellValue::Time(value) => Self::Time(value),
            CellValue::Interval(value) => Self::Interval(value),
            CellValue::Inet(value) => Self::Inet(value),
            CellValue::Json(value) => Self::Json(value.to_string()),
        }
    }
}

type ComparisonKey<'a> = Vec<DiffValue<'a>>;
type ComparisonRow<'a> = BTreeMap<&'a str, DiffValue<'a>>;
type KeyedRows<'a> = BTreeMap<ComparisonKey<'a>, ComparisonRow<'a>>;

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

        let all_keys: BTreeSet<_> = source_map.keys().chain(target_map.keys()).collect();
        for key in all_keys {
            match (source_map.get(key), target_map.get(key)) {
                (Some(src), None) => {
                    added += 1;
                    if row_diffs.len() < MAX_ROW_DIFF_SAMPLES {
                        row_diffs.push(DataRowDiff {
                            state: DataRowState::Added,
                            key: key_display(key),
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
                            key: key_display(key),
                            column_changes: Vec::new(),
                        });
                    }
                    if sync_sql_preview.len() < 50 {
                        sync_sql_preview.push(sync_delete(&*target_dialect, &target_qualified, key_columns, key));
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
                                key: key_display(key),
                                column_changes: changes.clone(),
                            });
                        }
                        if sync_sql_preview.len() < 50 {
                            if let Some(sql) =
                                sync_update(&*target_dialect, &target_qualified, key_columns, key, &changes)
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

fn sync_insert(dialect: &dyn crate::ports::SqlDialect, qualified: &str, row: &ComparisonRow<'_>) -> String {
    let columns = row
        .keys()
        .map(|column| dialect.quote_identifier(column))
        .collect::<Vec<_>>()
        .join(", ");
    let values = row.values().map(sql_literal).collect::<Vec<_>>().join(", ");
    format!("INSERT INTO {qualified} ({columns}) VALUES ({values});")
}

fn sync_delete(
    dialect: &dyn crate::ports::SqlDialect,
    qualified: &str,
    key_columns: &[String],
    key: &[DiffValue<'_>],
) -> String {
    format!(
        "DELETE FROM {qualified} WHERE {};",
        key_predicate(dialect, key_columns, key)
    )
}

fn sync_update(
    dialect: &dyn crate::ports::SqlDialect,
    qualified: &str,
    key_columns: &[String],
    key: &[DiffValue<'_>],
    changes: &[DataColumnChange],
) -> Option<String> {
    let assignments = changes
        .iter()
        .filter(|change| !key_columns.iter().any(|column| column == &change.column))
        .map(|change| {
            format!(
                "{} = {}",
                dialect.quote_identifier(&change.column),
                change
                    .source_value
                    .as_deref()
                    .map_or_else(|| "NULL".into(), |value| format!("'{}'", value.replace('\'', "''")))
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

fn key_predicate(dialect: &dyn crate::ports::SqlDialect, key_columns: &[String], key: &[DiffValue<'_>]) -> String {
    key_columns
        .iter()
        .zip(key)
        .map(|(column, value)| {
            let column = dialect.quote_identifier(column);
            if matches!(value, DiffValue::Null) {
                format!("{column} IS NULL")
            } else {
                format!("{column} = {}", sql_literal(value))
            }
        })
        .collect::<Vec<_>>()
        .join(" AND ")
}

fn sql_literal(value: &DiffValue<'_>) -> String {
    match value {
        DiffValue::Null => "NULL".into(),
        _ => format!("'{}'", cell_display(value).replace('\'', "''")),
    }
}

fn cell_display(cell: &DiffValue<'_>) -> String {
    match cell {
        DiffValue::Null => "∅".into(),
        DiffValue::Bool(value) => value.to_string(),
        DiffValue::Int64(value) => value.to_string(),
        DiffValue::Float64(value) => f64::from_bits(*value).to_string(),
        DiffValue::Decimal(value)
        | DiffValue::Text(value)
        | DiffValue::Uuid(value)
        | DiffValue::DateTime(value)
        | DiffValue::Timestamp(value)
        | DiffValue::TimestampTz(value)
        | DiffValue::TimeTz(value)
        | DiffValue::Date(value)
        | DiffValue::Time(value)
        | DiffValue::Interval(value)
        | DiffValue::Inet(value) => (*value).into(),
        // Retain the pre-existing binary preview spelling; provider-specific
        // binary literal support is outside this comparison/key correction.
        DiffValue::Bytes(value) => format!("Bytes({value:?})"),
        DiffValue::Json(value) => value.clone(),
    }
}

fn key_display(key: &[DiffValue<'_>]) -> String {
    // This is display only. Debug escaping keeps embedded separators and NULL
    // distinct from text, without creating a format that SQL needs to decode.
    key.iter()
        .map(|value| match value {
            DiffValue::Float64(bits) => format!("Float64({})", f64::from_bits(*bits)),
            _ => format!("{value:?}"),
        })
        .collect::<Vec<_>>()
        .join(" | ")
}

fn rows_by_key<'a>(result: &'a QueryResult, key_columns: &[String]) -> Result<KeyedRows<'a>, DbError> {
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
            .map(|idx| DiffValue::from(&row.0[*idx]))
            .collect::<Vec<_>>();
        let mut values = BTreeMap::new();
        for (col_idx, col) in result.columns.iter().enumerate() {
            values.insert(col.name.as_str(), DiffValue::from(&row.0[col_idx]));
        }
        match map.entry(key) {
            Entry::Vacant(entry) => {
                entry.insert(values);
            }
            Entry::Occupied(entry) => {
                return Err(DbError::Validation(format!(
                    "duplicate comparison key in sample: {}",
                    key_display(entry.key())
                )));
            }
        }
    }
    Ok(map)
}

fn column_changes(source: &ComparisonRow<'_>, target: &ComparisonRow<'_>) -> Vec<DataColumnChange> {
    let columns: BTreeSet<_> = source.keys().chain(target.keys()).copied().collect();
    columns
        .into_iter()
        .filter_map(|column| {
            let src = source.get(column);
            let tgt = target.get(column);
            if src == tgt {
                None
            } else {
                Some(DataColumnChange {
                    column: column.into(),
                    source_value: src.filter(|value| !matches!(value, DiffValue::Null)).map(cell_display),
                    target_value: tgt.filter(|value| !matches!(value, DiffValue::Null)).map(cell_display),
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
#[path = "tests/data_diff_tests.rs"]
mod tests;
