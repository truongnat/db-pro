//! Pure result-grid export and value formatting functions.

use super::*;

pub(crate) fn quote_sql_identifier(identifier: &str) -> String {
    format!("\"{}\"", identifier.replace('"', "\"\""))
}

/// `schema.name` with every part ANSI-quoted for generated SQL (Postgres and SQLite share `"`
/// quoting). Quoting each part keeps dotted identifiers — e.g. an AdventureWorks-style table
/// literally named `HumanResources.Department` inside schema `main` — from being reparsed by
/// the engine as extra qualification parts.
pub(crate) fn qualified_sql_name(schema: &str, name: &str) -> String {
    let name = quote_sql_identifier(name);
    if schema.is_empty() {
        name
    } else {
        format!("{}.{}", quote_sql_identifier(schema), name)
    }
}

pub(super) fn cell_sql_literal(cell: &UiCell) -> String {
    match cell {
        UiCell::Null => "NULL".to_owned(),
        UiCell::Boolean(value) => value.to_string().to_uppercase(),
        UiCell::Number(value) if value.parse::<f64>().is_ok() => value.clone(),
        UiCell::Json(value) => format!("'{}'", value.replace('\'', "''")),
        UiCell::Bytes(value) => format!("'{}'", value.replace('\'', "''")),
        UiCell::Number(value) | UiCell::Text(value) => format!("'{}'", value.replace('\'', "''")),
    }
}

pub(super) fn escape_delimited_field(value: &str, delimiter: char) -> String {
    if value.contains(delimiter) || value.contains('"') || value.contains('\n') || value.contains('\r') {
        format!("\"{}\"", value.replace('"', "\"\""))
    } else {
        value.to_owned()
    }
}

pub(super) fn format_cell_delimited(cell: &UiCell, delimiter: char) -> String {
    match cell {
        UiCell::Null => String::new(),
        UiCell::Boolean(value) => value.to_string(),
        UiCell::Number(value) => value.clone(),
        UiCell::Text(value) => escape_delimited_field(value, delimiter),
        UiCell::Json(value) => escape_delimited_field(value, delimiter),
        UiCell::Bytes(value) => escape_delimited_field(value, delimiter),
    }
}

pub(super) fn format_cell_csv(cell: &UiCell) -> String {
    format_cell_delimited(cell, ',')
}

/// Workbook of the visible grid. Integers outside the 53-bit exact range stay text.
pub(super) fn format_result_excel(result: &UiQueryResult) -> Result<Vec<u8>, String> {
    let mut workbook = rust_xlsxwriter::Workbook::new();
    let worksheet = workbook.add_worksheet();
    let header = rust_xlsxwriter::Format::new().set_bold();
    for (index, column) in result.columns.iter().enumerate() {
        let index = excel_column_index(index)?;
        worksheet
            .write_string_with_format(0, index, &column.name, &header)
            .map_err(|error| format!("excel header write failed: {error}"))?;
    }
    for (row_index, row) in result.rows.iter().enumerate() {
        let row_index = excel_row_index(row_index)?;
        for (column_index, cell) in row.iter().enumerate() {
            write_excel_cell(worksheet, row_index, excel_column_index(column_index)?, cell)?;
        }
    }
    workbook
        .save_to_buffer()
        .map_err(|error| format!("excel save failed: {error}"))
}

fn excel_column_index(index: usize) -> Result<u16, String> {
    u16::try_from(index).map_err(|_| "Excel export has too many columns".to_owned())
}

fn excel_row_index(index: usize) -> Result<u32, String> {
    index
        .checked_add(1)
        .and_then(|index| u32::try_from(index).ok())
        .ok_or_else(|| "Excel export has too many rows".to_owned())
}

fn write_excel_cell(
    worksheet: &mut rust_xlsxwriter::Worksheet,
    row: u32,
    column: u16,
    cell: &UiCell,
) -> Result<(), String> {
    let written = match cell {
        UiCell::Null => return Ok(()),
        UiCell::Boolean(value) => worksheet.write_boolean(row, column, *value).map(|_| ()),
        UiCell::Number(value) => write_excel_number(worksheet, row, column, value),
        UiCell::Text(value) | UiCell::Json(value) | UiCell::Bytes(value) => {
            worksheet.write_string(row, column, value).map(|_| ())
        }
    };
    written.map_err(|error| format!("excel cell write failed: {error}"))
}

fn write_excel_number(
    worksheet: &mut rust_xlsxwriter::Worksheet,
    row: u32,
    column: u16,
    value: &str,
) -> Result<(), rust_xlsxwriter::XlsxError> {
    const EXACT_LIMIT: i64 = 1_i64 << 53;
    if let Ok(integer) = value.parse::<i64>() {
        if (-EXACT_LIMIT..=EXACT_LIMIT).contains(&integer) {
            return worksheet.write_number(row, column, integer as f64).map(|_| ());
        }
        return worksheet.write_string(row, column, value).map(|_| ());
    }
    if let Ok(number) = value.parse::<f64>() {
        if number.is_finite() {
            return worksheet.write_number(row, column, number).map(|_| ());
        }
    }
    worksheet.write_string(row, column, value).map(|_| ())
}

pub(super) fn format_result_delimited(result: &UiQueryResult, delimiter: &str) -> String {
    let separator = delimiter.chars().next().unwrap_or(',');
    let mut output = result
        .columns
        .iter()
        .map(|column| escape_delimited_field(&column.name, separator))
        .collect::<Vec<_>>()
        .join(delimiter);
    output.push('\n');
    for row in &result.rows {
        output.push_str(
            &row.iter()
                .map(|cell| format_cell_delimited(cell, separator))
                .collect::<Vec<_>>()
                .join(delimiter),
        );
        output.push('\n');
    }
    output
}

pub(super) fn format_result_sql_insert(result: &UiQueryResult, table: &str) -> String {
    if result.columns.is_empty() {
        return String::new();
    }
    let cols = result
        .columns
        .iter()
        .map(|column| quote_sql_identifier(&column.name))
        .collect::<Vec<_>>()
        .join(", ");
    let mut output = String::new();
    const BATCH: usize = 50;
    for chunk in result.rows.chunks(BATCH) {
        output.push_str(&format!(
            "INSERT INTO {} ({cols}) VALUES\n",
            quote_sql_identifier(table)
        ));
        for (index, row) in chunk.iter().enumerate() {
            let values = row.iter().map(format_cell_sql_literal).collect::<Vec<_>>().join(", ");
            output.push_str("  (");
            output.push_str(&values);
            output.push(')');
            if index + 1 < chunk.len() {
                output.push_str(",\n");
            } else {
                output.push_str(";\n");
            }
        }
        output.push('\n');
    }
    output
}

pub(super) fn format_result_copy(result: &UiQueryResult, table: &str) -> String {
    let cols = result
        .columns
        .iter()
        .map(|column| quote_sql_identifier(&column.name))
        .collect::<Vec<_>>()
        .join(", ");
    let mut output = format!("COPY {} ({cols}) FROM stdin;\n", quote_sql_identifier(table));
    for row in &result.rows {
        output.push_str(&row.iter().map(format_cell_copy_field).collect::<Vec<_>>().join("\t"));
        output.push('\n');
    }
    output.push_str("\\.\n");
    output
}

pub(super) fn format_result_markdown(result: &UiQueryResult) -> String {
    if result.columns.is_empty() {
        return String::new();
    }
    let mut output = String::new();
    output.push('|');
    for col in &result.columns {
        output.push_str(&format!(" {} |", col.name.replace('|', "\\|")));
    }
    output.push('\n');
    output.push('|');
    for _ in &result.columns {
        output.push_str(" --- |");
    }
    output.push('\n');
    for row in &result.rows {
        output.push('|');
        for cell in row {
            let text = match cell {
                UiCell::Null => "NULL".to_owned(),
                UiCell::Boolean(v) => v.to_string(),
                UiCell::Number(v) | UiCell::Text(v) | UiCell::Json(v) | UiCell::Bytes(v) => {
                    v.replace('|', "\\|").replace('\n', " ")
                }
            };
            output.push_str(&format!(" {text} |"));
        }
        output.push('\n');
    }
    output
}

/// Unique JSON object keys for `columns`: duplicates get `_2`, `_3`, … suffixes so a
/// `SELECT 1 AS x, 2 AS x` or self-join result does not silently drop values in a keyed map.
pub(super) fn unique_json_keys(columns: &[crate::UiColumn]) -> Vec<String> {
    let mut seen: std::collections::HashMap<&str, usize> = std::collections::HashMap::new();
    columns
        .iter()
        .map(|column| {
            let count = seen.entry(column.name.as_str()).or_insert(0);
            *count += 1;
            if *count == 1 { column.name.clone() } else { format!("{}_{}", column.name, count) }
        })
        .collect()
}

pub(super) fn format_result_json(result: &UiQueryResult) -> String {
    let keys = unique_json_keys(&result.columns);
    let array: Vec<serde_json::Map<String, serde_json::Value>> = result
        .rows
        .iter()
        .map(|row| {
            let mut obj = serde_json::Map::new();
            for (col_idx, cell) in row.iter().enumerate() {
                if let Some(key) = keys.get(col_idx) {
                    obj.insert(key.clone(), cell_to_json_value(cell));
                }
            }
            obj
        })
        .collect();

    serde_json::to_string_pretty(&array).unwrap_or_else(|_| "[]".to_owned())
}

pub(super) fn format_cell_sql_literal(cell: &UiCell) -> String {
    match cell {
        UiCell::Null => "NULL".into(),
        UiCell::Boolean(value) => if *value { "TRUE" } else { "FALSE" }.into(),
        UiCell::Number(value) => value.clone(),
        UiCell::Text(value) | UiCell::Json(value) | UiCell::Bytes(value) => {
            format!("'{}'", value.replace('\'', "''"))
        }
    }
}

pub(super) fn format_cell_copy_field(cell: &UiCell) -> String {
    match cell {
        UiCell::Null => "\\N".into(),
        UiCell::Boolean(value) => if *value { "t" } else { "f" }.into(),
        UiCell::Number(value) => value.clone(),
        UiCell::Text(value) | UiCell::Json(value) | UiCell::Bytes(value) => value
            .replace('\\', "\\\\")
            .replace('\t', "\\t")
            .replace('\n', "\\n")
            .replace('\r', "\\r"),
    }
}

pub(super) fn cell_to_json_value(cell: &UiCell) -> serde_json::Value {
    const JSON_EXACT_INTEGER_LIMIT: i64 = 1 << 53;
    match cell {
        UiCell::Null => serde_json::Value::Null,
        UiCell::Boolean(value) => serde_json::Value::Bool(*value),
        UiCell::Number(value) => match value.parse::<i64>() {
            Ok(integer) if (-JSON_EXACT_INTEGER_LIMIT..=JSON_EXACT_INTEGER_LIMIT).contains(&integer) => {
                serde_json::Value::Number(integer.into())
            }
            _ => match value.parse::<f64>() {
                Ok(number) if number.is_finite() && number.to_string() == *value => {
                    serde_json::Number::from_f64(number)
                        .map(serde_json::Value::Number)
                        .unwrap_or_else(|| serde_json::Value::String(value.clone()))
                }
                _ => serde_json::Value::String(value.clone()),
            },
        },
        UiCell::Text(value) => serde_json::Value::String(value.clone()),
        UiCell::Json(value) => serde_json::from_str(value).unwrap_or_else(|_| serde_json::Value::String(value.clone())),
        UiCell::Bytes(value) => serde_json::Value::String(value.clone()),
    }
}

#[cfg(test)]
#[path = "tests/result_grid_export_tests.rs"]
mod tests;

