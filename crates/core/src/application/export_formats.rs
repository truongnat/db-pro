use std::collections::HashSet;

use crate::domain::error::DbError;
use crate::domain::query::{CellValue, QueryResult};

pub(super) fn render_csv(result: &QueryResult) -> Result<Vec<u8>, DbError> {
    let mut writer = csv::Writer::from_writer(Vec::new());
    let headers: Vec<&str> = result.columns.iter().map(|column| column.name.as_str()).collect();
    writer
        .write_record(&headers)
        .map_err(|error| DbError::Internal(format!("csv header write failed: {error}")))?;

    for row in &result.rows {
        let fields: Vec<String> = row.0.iter().map(cell_to_csv_string).collect();
        let references: Vec<&str> = fields.iter().map(String::as_str).collect();
        writer
            .write_record(&references)
            .map_err(|error| DbError::Internal(format!("csv row write failed: {error}")))?;
    }

    writer
        .into_inner()
        .map_err(|error| DbError::Internal(format!("csv flush failed: {error}")))
}

pub(super) fn render_json(result: &QueryResult) -> Result<Vec<u8>, DbError> {
    validate_json_column_names(result)?;
    let rows: Vec<serde_json::Map<String, serde_json::Value>> = result
        .rows
        .iter()
        .map(|row| {
            let mut object = serde_json::Map::new();
            for (column, cell) in result.columns.iter().zip(row.0.iter()) {
                object.insert(column.name.clone(), cell_to_json(cell)?);
            }
            Ok(object)
        })
        .collect::<Result<_, DbError>>()?;

    serde_json::to_vec_pretty(&rows).map_err(|error| DbError::Internal(format!("json serialization failed: {error}")))
}

pub(super) fn render_excel(result: &QueryResult) -> Result<Vec<u8>, DbError> {
    let mut workbook = rust_xlsxwriter::Workbook::new();
    let worksheet = workbook.add_worksheet();
    let header_format = rust_xlsxwriter::Format::new().set_bold();

    write_excel_headers(worksheet, &header_format, result)?;
    write_excel_rows(worksheet, result)?;

    workbook
        .save_to_buffer()
        .map_err(|error| DbError::Internal(format!("excel save failed: {error}")))
}

fn write_excel_headers(
    worksheet: &mut rust_xlsxwriter::Worksheet,
    header_format: &rust_xlsxwriter::Format,
    result: &QueryResult,
) -> Result<(), DbError> {
    for (column_index, column) in result.columns.iter().enumerate() {
        let column_index = excel_column_index(column_index)?;
        worksheet
            .write_string_with_format(0, column_index, &column.name, header_format)
            .map_err(|error| DbError::Internal(format!("excel header write failed: {error}")))?;
    }
    Ok(())
}

fn write_excel_rows(worksheet: &mut rust_xlsxwriter::Worksheet, result: &QueryResult) -> Result<(), DbError> {
    for (row_index, row) in result.rows.iter().enumerate() {
        let row_index = excel_row_index(row_index)?;
        for (column_index, cell) in row.0.iter().enumerate() {
            write_excel_cell(worksheet, row_index, excel_column_index(column_index)?, cell)?;
        }
    }
    Ok(())
}

pub(super) fn excel_column_index(index: usize) -> Result<u16, DbError> {
    u16::try_from(index).map_err(|_| DbError::Validation("Excel export has too many columns".into()))
}

pub(super) fn excel_row_index(index: usize) -> Result<u32, DbError> {
    index
        .checked_add(1)
        .and_then(|index| u32::try_from(index).ok())
        .ok_or_else(|| DbError::Validation("Excel export has too many rows".into()))
}

fn validate_json_column_names(result: &QueryResult) -> Result<(), DbError> {
    let mut names = HashSet::with_capacity(result.columns.len());
    for column in &result.columns {
        if !names.insert(column.name.as_str()) {
            return Err(DbError::Validation(format!(
                "JSON export requires unique column names; duplicate column: {}",
                column.name
            )));
        }
    }
    Ok(())
}

pub(super) const MAX_EXACT_EXCEL_INTEGER: i64 = 1_i64 << 53;

pub(super) fn excel_integer_is_exact(value: i64) -> bool {
    (-MAX_EXACT_EXCEL_INTEGER..=MAX_EXACT_EXCEL_INTEGER).contains(&value)
}

fn cell_to_csv_string(cell: &CellValue) -> String {
    match cell {
        CellValue::Null => String::new(),
        CellValue::Bool(value) => value.to_string(),
        CellValue::Int64(value) => value.to_string(),
        CellValue::Float64(value) => value.to_string(),
        CellValue::Decimal(value) | CellValue::Text(value) => value.clone(),
        CellValue::Bytes(_) => "[binary]".into(),
        CellValue::Uuid(value)
        | CellValue::DateTime(value)
        | CellValue::Timestamp(value)
        | CellValue::TimestampTz(value)
        | CellValue::TimeTz(value)
        | CellValue::Date(value)
        | CellValue::Time(value)
        | CellValue::Interval(value)
        | CellValue::Inet(value) => value.clone(),
        CellValue::Json(value) => value.to_string(),
    }
}

fn cell_to_json(cell: &CellValue) -> Result<serde_json::Value, DbError> {
    match cell {
        CellValue::Null => Ok(serde_json::Value::Null),
        CellValue::Bool(value) => Ok(serde_json::Value::Bool(*value)),
        CellValue::Int64(value) => Ok(serde_json::Value::Number((*value).into())),
        CellValue::Float64(value) => serde_json::Number::from_f64(*value)
            .map(serde_json::Value::Number)
            .ok_or_else(|| DbError::Validation("JSON export cannot represent a non-finite float".into())),
        // Keep decimal text exact instead of converting through f64.
        CellValue::Decimal(value) | CellValue::Text(value) => Ok(serde_json::Value::String(value.clone())),
        CellValue::Bytes(value) => Ok(serde_json::Value::String(format!("[{} bytes]", value.len()))),
        CellValue::Uuid(value)
        | CellValue::DateTime(value)
        | CellValue::Timestamp(value)
        | CellValue::TimestampTz(value)
        | CellValue::TimeTz(value)
        | CellValue::Date(value)
        | CellValue::Time(value)
        | CellValue::Interval(value)
        | CellValue::Inet(value) => Ok(serde_json::Value::String(value.clone())),
        CellValue::Json(value) => Ok(value.clone()),
    }
}

fn write_excel_cell(
    worksheet: &mut rust_xlsxwriter::Worksheet,
    row: u32,
    column: u16,
    cell: &CellValue,
) -> Result<(), DbError> {
    let result = match cell {
        CellValue::Null => return Ok(()),
        CellValue::Bool(value) => worksheet.write_boolean(row, column, *value),
        CellValue::Int64(value) if excel_integer_is_exact(*value) => worksheet.write_number(row, column, *value as f64),
        CellValue::Int64(value) => worksheet.write_string(row, column, value.to_string()),
        CellValue::Float64(value) => worksheet.write_number(row, column, *value),
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
        | CellValue::Inet(value) => worksheet.write_string(row, column, value),
        CellValue::Bytes(value) => worksheet.write_string(row, column, format!("[{} bytes]", value.len())),
        CellValue::Json(value) => worksheet.write_string(row, column, value.to_string()),
    };
    result
        .map(|_| ())
        .map_err(|error| DbError::Internal(format!("excel write failed: {error}")))
}
