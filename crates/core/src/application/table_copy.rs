//! Copy rows between two open connections as CSV or JSON.
//!
//! The first selected column is the conflict key. With no column list the
//! source is read with `SELECT *` and that result's first column is the key.

use crate::domain::query::{CellValue, QueryParam, QueryResult};
use crate::domain::transfer::TransferConflictPolicy;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CopyFormat {
    Csv,
    Json,
}

impl CopyFormat {
    pub fn parse(value: &str) -> Result<Self, String> {
        match value.trim().to_ascii_lowercase().as_str() {
            "csv" => Ok(Self::Csv),
            "json" => Ok(Self::Json),
            _ => Err("choose CSV or JSON".into()),
        }
    }
}

#[derive(Debug, Clone)]
pub struct TableCopyRequest {
    pub source_connection_id: String,
    pub target_connection_id: String,
    pub tables: Vec<String>,
    pub format: CopyFormat,
    pub conflict: TransferConflictPolicy,
    /// Empty means `SELECT *`. The first column is the conflict key.
    pub columns: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TableCopyOutput {
    pub source_connection_id: String,
    pub target_connection_id: String,
    pub tables: Vec<String>,
    pub format: CopyFormat,
    pub conflict: TransferConflictPolicy,
    pub body: String,
    pub rows_written: u64,
    pub rows_skipped: u64,
}

/// Reads the chosen tables and writes the rows that survive the conflict policy.
/// Implementations look up an already-open connection by its id.
pub trait TableCopyConnector: Send + Sync {
    fn quote(&self, connection_id: &str, identifier: &str) -> Result<String, String>;
    fn placeholder(&self, connection_id: &str, index: usize) -> Result<String, String>;
    fn query(&self, connection_id: &str, sql: &str) -> Result<QueryResult, String>;
    fn execute(&self, connection_id: &str, sql: &str, params: &[QueryParam]) -> Result<u64, String>;
}

struct TablePlan {
    columns: Vec<String>,
    written: Vec<Vec<String>>,
    skipped: u64,
    statements: Vec<(String, Vec<QueryParam>)>,
}

pub fn copy_tables(connector: &dyn TableCopyConnector, request: &TableCopyRequest) -> Result<TableCopyOutput, String> {
    let source_id = request.source_connection_id.trim();
    let target_id = request.target_connection_id.trim();
    if source_id.is_empty() || target_id.is_empty() {
        return Err("choose a source connection and a target connection".into());
    }
    let tables: Vec<&str> = request
        .tables
        .iter()
        .map(|table| table.trim())
        .filter(|table| !table.is_empty())
        .collect();
    if tables.is_empty() {
        return Err("choose at least one table".into());
    }

    let mut plans = Vec::new();
    for table in &tables {
        plans.push(plan_table(
            connector,
            source_id,
            target_id,
            table,
            &request.columns,
            request.conflict,
        )?);
    }
    for plan in &plans {
        for (sql, params) in &plan.statements {
            connector.execute(target_id, sql, params)?;
        }
    }

    let body = plans
        .iter()
        .map(|plan| match request.format {
            CopyFormat::Csv => render_csv(&plan.columns, &plan.written),
            CopyFormat::Json => render_json(&plan.columns, &plan.written),
        })
        .collect::<String>();
    let rows_written = plans.iter().map(|plan| plan.written.len() as u64).sum();
    let rows_skipped = plans.iter().map(|plan| plan.skipped).sum();
    Ok(TableCopyOutput {
        source_connection_id: source_id.to_owned(),
        target_connection_id: target_id.to_owned(),
        tables: tables.into_iter().map(str::to_owned).collect(),
        format: request.format,
        conflict: request.conflict,
        body,
        rows_written,
        rows_skipped,
    })
}

fn plan_table(
    connector: &dyn TableCopyConnector,
    source_id: &str,
    target_id: &str,
    table: &str,
    requested_columns: &[String],
    conflict: TransferConflictPolicy,
) -> Result<TablePlan, String> {
    let source_sql = select_sql(connector, source_id, table, requested_columns)?;
    let source = connector.query(source_id, &source_sql)?;
    source
        .validate()
        .map_err(|error| format!("source read failed: {error}"))?;
    let columns = if requested_columns.is_empty() {
        source
            .columns
            .iter()
            .map(|column| column.name.clone())
            .collect::<Vec<_>>()
    } else {
        requested_columns.to_vec()
    };
    if columns.is_empty() {
        let target_sql = select_sql(connector, target_id, table, &[])?;
        let target = connector.query(target_id, &target_sql)?;
        target
            .validate()
            .map_err(|error| format!("target read failed: {error}"))?;
        return Ok(TablePlan {
            columns,
            written: Vec::new(),
            skipped: 0,
            statements: Vec::new(),
        });
    }
    let source_rows = string_rows(&source);

    let key_column = columns[0].clone();
    let target_sql = select_sql(connector, target_id, table, &[key_column])?;
    let target = connector.query(target_id, &target_sql)?;
    target
        .validate()
        .map_err(|error| format!("target read failed: {error}"))?;
    let target_keys: Vec<String> = string_rows(&target)
        .into_iter()
        .map(|row| row.first().cloned().unwrap_or_default())
        .collect();
    let (written, skipped) = apply_conflict(&source_rows, &target_keys, conflict)?;
    let qualified = qualified_name(connector, target_id, table)?;
    let statements = write_statements(
        connector,
        target_id,
        &qualified,
        &columns,
        &written,
        &target_keys,
        conflict,
    )?;
    Ok(TablePlan {
        columns,
        written,
        skipped,
        statements,
    })
}

fn apply_conflict(
    rows: &[Vec<String>],
    target_keys: &[String],
    conflict: TransferConflictPolicy,
) -> Result<(Vec<Vec<String>>, u64), String> {
    let mut written = Vec::new();
    let mut skipped = 0u64;
    for row in rows {
        let key = row.first().map(String::as_str).unwrap_or("");
        if target_keys.iter().any(|existing| existing == key) {
            match conflict {
                TransferConflictPolicy::Abort => return Err(format!("conflict on key {key}")),
                TransferConflictPolicy::Skip => {
                    skipped += 1;
                    continue;
                }
                TransferConflictPolicy::Replace => written.push(row.clone()),
            }
        } else {
            written.push(row.clone());
        }
    }
    Ok((written, skipped))
}

fn write_statements(
    connector: &dyn TableCopyConnector,
    target_id: &str,
    qualified_table: &str,
    columns: &[String],
    written: &[Vec<String>],
    target_keys: &[String],
    conflict: TransferConflictPolicy,
) -> Result<Vec<(String, Vec<QueryParam>)>, String> {
    let mut statements = Vec::new();
    let column_sql = columns
        .iter()
        .map(|column| connector.quote(target_id, column))
        .collect::<Result<Vec<_>, _>>()?
        .join(", ");
    let key_sql = connector.quote(target_id, &columns[0])?;
    for row in written {
        let mut values = row.to_vec();
        values.resize(columns.len(), String::new());
        values.truncate(columns.len());
        let key = values.first().cloned().unwrap_or_default();
        if conflict == TransferConflictPolicy::Replace && target_keys.iter().any(|existing| existing == &key) {
            let placeholder = connector.placeholder(target_id, 1)?;
            statements.push((
                format!("DELETE FROM {qualified_table} WHERE {key_sql} = {placeholder}"),
                vec![QueryParam::Text(key)],
            ));
        }
        let placeholders = (1..=columns.len())
            .map(|index| connector.placeholder(target_id, index))
            .collect::<Result<Vec<_>, _>>()?
            .join(", ");
        let params = values.into_iter().map(QueryParam::Text).collect();
        statements.push((
            format!("INSERT INTO {qualified_table} ({column_sql}) VALUES ({placeholders})"),
            params,
        ));
    }
    Ok(statements)
}

fn select_sql(
    connector: &dyn TableCopyConnector,
    connection_id: &str,
    table: &str,
    columns: &[String],
) -> Result<String, String> {
    let qualified = qualified_name(connector, connection_id, table)?;
    if columns.is_empty() {
        return Ok(format!("SELECT * FROM {qualified}"));
    }
    let column_sql = columns
        .iter()
        .map(|column| connector.quote(connection_id, column))
        .collect::<Result<Vec<_>, _>>()?
        .join(", ");
    Ok(format!("SELECT {column_sql} FROM {qualified}"))
}

fn qualified_name(connector: &dyn TableCopyConnector, connection_id: &str, table: &str) -> Result<String, String> {
    let parts: Vec<&str> = table
        .split('.')
        .map(str::trim)
        .filter(|part| !part.is_empty())
        .collect();
    if parts.is_empty() {
        return Err("choose at least one table".into());
    }
    parts
        .into_iter()
        .map(|part| connector.quote(connection_id, part))
        .collect::<Result<Vec<_>, _>>()
        .map(|quoted| quoted.join("."))
}

fn string_rows(result: &QueryResult) -> Vec<Vec<String>> {
    result
        .rows
        .iter()
        .map(|row| row.0.iter().map(cell_text).collect())
        .collect()
}

fn cell_text(cell: &CellValue) -> String {
    match cell {
        CellValue::Null => String::new(),
        CellValue::Bool(value) => value.to_string(),
        CellValue::Int64(value) => value.to_string(),
        CellValue::Float64(value) => value.to_string(),
        CellValue::Decimal(value)
        | CellValue::Text(value)
        | CellValue::Uuid(value)
        | CellValue::DateTime(value)
        | CellValue::Timestamp(value)
        | CellValue::TimestampTz(value)
        | CellValue::TimeTz(value)
        | CellValue::Date(value)
        | CellValue::Time(value)
        | CellValue::Interval(value)
        | CellValue::Inet(value) => value.clone(),
        CellValue::Bytes(value) => String::from_utf8_lossy(value).into_owned(),
        CellValue::Json(value) => value.to_string(),
    }
}

fn render_csv(columns: &[String], rows: &[Vec<String>]) -> String {
    let mut out = columns
        .iter()
        .map(|column| csv_cell(column))
        .collect::<Vec<_>>()
        .join(",");
    out.push('\n');
    for row in rows {
        out.push_str(&row.iter().map(|cell| csv_cell(cell)).collect::<Vec<_>>().join(","));
        out.push('\n');
    }
    out
}

fn csv_cell(value: &str) -> String {
    if value.contains([',', '"', '\n']) {
        format!("\"{}\"", value.replace('"', "\"\""))
    } else {
        value.to_owned()
    }
}

fn render_json(columns: &[String], rows: &[Vec<String>]) -> String {
    let mut objects = Vec::new();
    for row in rows {
        let fields = columns
            .iter()
            .enumerate()
            .map(|(index, column)| {
                let value = row.get(index).map(String::as_str).unwrap_or("");
                format!("\"{}\":\"{}\"", json_escape(column), json_escape(value))
            })
            .collect::<Vec<_>>()
            .join(",");
        objects.push(format!("{{{fields}}}"));
    }
    format!("[{}]", objects.join(","))
}

fn json_escape(value: &str) -> String {
    value.replace('\\', "\\\\").replace('"', "\\\"")
}

#[cfg(test)]
#[path = "tests/table_copy_tests.rs"]
mod tests;
