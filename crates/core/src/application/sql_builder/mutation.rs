use crate::domain::error::DbError;
use crate::domain::query::{CellValue, QueryParam};
use crate::ports::SqlDialect;

use super::{qualify, PlaceholderWriter};

pub fn build_insert(
    dialect: &dyn SqlDialect,
    schema: &str,
    table: &str,
    columns: &[String],
    values: &[CellValue],
) -> Result<(String, Vec<QueryParam>), DbError> {
    if columns.len() != values.len() {
        return Err(DbError::Validation(format!(
            "column count ({}) does not match value count ({})",
            columns.len(),
            values.len()
        )));
    }
    if columns.is_empty() {
        return Err(DbError::Validation("insert requires at least one column".into()));
    }
    let column_list = columns
        .iter()
        .map(|column| dialect.quote_identifier(column))
        .collect::<Vec<_>>()
        .join(", ");
    let mut placeholders = PlaceholderWriter::new(dialect);
    let value_list = values
        .iter()
        .map(|_| placeholders.next())
        .collect::<Vec<_>>()
        .join(", ");
    let target = qualify(dialect, schema, table);

    let sql = format!("INSERT INTO {target} ({column_list}) VALUES ({value_list})");
    let params = values.iter().map(super::cell_to_param).collect();
    Ok((sql, params))
}

pub fn build_update(
    dialect: &dyn SqlDialect,
    schema: &str,
    table: &str,
    columns: &[String],
    values: &[CellValue],
    pk_columns: &[String],
    pk_values: &[CellValue],
) -> Result<(String, Vec<QueryParam>), DbError> {
    validate_parallel_values(columns, values, "update")?;
    validate_primary_key(pk_columns, pk_values)?;

    let mut placeholders = PlaceholderWriter::new(dialect);
    let set_parts = columns
        .iter()
        .map(|column| format!("{} = {}", dialect.quote_identifier(column), placeholders.next()))
        .collect::<Vec<_>>();
    let pk_where = pk_columns
        .iter()
        .map(|column| format!("{} = {}", dialect.quote_identifier(column), placeholders.next()))
        .collect::<Vec<_>>();
    let target = qualify(dialect, schema, table);
    let sql = format!(
        "UPDATE {target} SET {} WHERE {}",
        set_parts.join(", "),
        pk_where.join(" AND ")
    );

    let mut params = values.iter().map(super::cell_to_param).collect::<Vec<_>>();
    params.extend(pk_values.iter().map(super::cell_to_param));
    Ok((sql, params))
}

pub fn build_delete(
    dialect: &dyn SqlDialect,
    schema: &str,
    table: &str,
    pk_columns: &[String],
    pk_values: &[CellValue],
) -> Result<(String, Vec<QueryParam>), DbError> {
    validate_primary_key(pk_columns, pk_values)?;
    let mut placeholders = PlaceholderWriter::new(dialect);
    let pk_where = pk_columns
        .iter()
        .map(|column| format!("{} = {}", dialect.quote_identifier(column), placeholders.next()))
        .collect::<Vec<_>>();
    let target = qualify(dialect, schema, table);
    let sql = format!("DELETE FROM {target} WHERE {}", pk_where.join(" AND "));
    let params = pk_values.iter().map(super::cell_to_param).collect();
    Ok((sql, params))
}

pub fn build_select_by_pk(
    dialect: &dyn SqlDialect,
    schema: &str,
    table: &str,
    pk_columns: &[String],
    pk_values: &[CellValue],
) -> Result<(String, Vec<QueryParam>), DbError> {
    validate_primary_key(pk_columns, pk_values)?;
    let mut placeholders = PlaceholderWriter::new(dialect);
    let pk_where = pk_columns
        .iter()
        .map(|column| format!("{} = {}", dialect.quote_identifier(column), placeholders.next()))
        .collect::<Vec<_>>();
    let target = qualify(dialect, schema, table);
    let sql = format!("SELECT * FROM {target} WHERE {} LIMIT 1", pk_where.join(" AND "));
    let params = pk_values.iter().map(super::cell_to_param).collect();
    Ok((sql, params))
}

fn validate_parallel_values(columns: &[String], values: &[CellValue], operation: &str) -> Result<(), DbError> {
    if columns.len() != values.len() {
        return Err(DbError::Validation(format!(
            "column count ({}) does not match value count ({})",
            columns.len(),
            values.len()
        )));
    }
    if columns.is_empty() {
        return Err(DbError::Validation(format!("{operation} requires at least one column")));
    }
    Ok(())
}

fn validate_primary_key(pk_columns: &[String], pk_values: &[CellValue]) -> Result<(), DbError> {
    if pk_columns.is_empty() || pk_columns.len() != pk_values.len() {
        return Err(DbError::Validation(format!(
            "pk column count ({}) does not match pk value count ({})",
            pk_columns.len(),
            pk_values.len()
        )));
    }
    Ok(())
}
