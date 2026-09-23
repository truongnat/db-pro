//! Pure result-grid export and value formatting functions.

use super::*;

pub(super) fn quote_sql_identifier(identifier: &str) -> String {
    format!("\"{}\"", identifier.replace('"', "\"\""))
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

pub(super) fn format_result_json(result: &UiQueryResult) -> String {
    let array: Vec<serde_json::Map<String, serde_json::Value>> = result
        .rows
        .iter()
        .map(|row| {
            let mut obj = serde_json::Map::new();
            for (col_idx, cell) in row.iter().enumerate() {
                if let Some(col) = result.columns.get(col_idx) {
                    obj.insert(col.name.clone(), cell_to_json_value(cell));
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
