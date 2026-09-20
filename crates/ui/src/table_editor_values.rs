//! Provider-neutral value parsing and sample generation for table editing.
//!
//! This module is deliberately independent from the composition root: parsing
//! is a deterministic boundary concern and must not depend on UI state or
//! runtime orchestration.

use super::UiCell;
use crate::{ColumnWritePolicy, UiTableInfo};
use bigdecimal::BigDecimal;

pub(crate) fn duplicate_row_values(info: &UiTableInfo, row: &[UiCell]) -> Vec<String> {
    info.columns
        .iter()
        .enumerate()
        .map(|(index, column)| {
            if column.is_primary_key || !ColumnWritePolicy::read(column).is_writable() {
                String::new()
            } else {
                row.get(index).map_or_else(String::new, |cell| match cell {
                    UiCell::Null => String::new(),
                    _ => crate::cell_text(cell),
                })
            }
        })
        .collect()
}

pub(crate) fn parse_insert_row_values(
    info: &UiTableInfo,
    raw_values: &[String],
) -> Result<(Vec<String>, Vec<UiCell>), String> {
    let mut columns = Vec::new();
    let mut values = Vec::new();
    for (column, raw) in info.columns.iter().zip(raw_values) {
        let write_block = ColumnWritePolicy::read(column).write_block();
        if raw.trim().is_empty() && write_block.is_some() {
            continue;
        }
        if let Some(block) = write_block {
            return Err(format!("{}: {}", column.name, block.reason()));
        }
        if raw.trim().is_empty() && !column.nullable && column.default.is_none() {
            return Err(format!("{} is required", column.name));
        }
        match parse_insert_value(raw, &column.data_type) {
            Ok(Some(value)) => {
                columns.push(column.name.clone());
                values.push(value);
            }
            Ok(None) => {}
            Err(error) => return Err(format!("{}: {error}", column.name)),
        }
        if matches!(values.last(), Some(UiCell::Null)) && !column.nullable {
            return Err(format!("{} is NOT NULL; enter a value instead", column.name));
        }
    }
    if columns.is_empty() {
        return Err("Enter at least one value; leave defaulted columns empty".to_owned());
    }
    Ok((columns, values))
}

pub(crate) fn generate_sample_value(column_name: &str, data_type: &str) -> String {
    let lower_type = data_type.to_ascii_lowercase();
    let lower_name = column_name.to_ascii_lowercase();
    let rand_num = (rand::random::<u32>() % 9000) + 1000;

    if lower_type.contains("uuid") || lower_type.contains("guid") {
        uuid::Uuid::new_v4().to_string()
    } else if lower_type.contains("timestamptz") || lower_type.contains("timestamp") || lower_type.contains("datetime")
    {
        chrono::Utc::now().to_rfc3339()
    } else if lower_type.contains("date") {
        chrono::Utc::now().format("%Y-%m-%d").to_string()
    } else if lower_type.contains("time") {
        chrono::Utc::now().format("%H:%M:%S").to_string()
    } else if lower_type.contains("bool") {
        if rand::random::<bool>() {
            "true".to_owned()
        } else {
            "false".to_owned()
        }
    } else if lower_type.contains("int")
        || lower_type.contains("serial")
        || lower_type.contains("bigint")
        || lower_type.contains("smallint")
    {
        rand_num.to_string()
    } else if lower_type.contains("float")
        || lower_type.contains("double")
        || lower_type.contains("real")
        || lower_type.contains("numeric")
        || lower_type.contains("decimal")
    {
        format!("{}.{:02}", rand_num / 10, rand_num % 100)
    } else if lower_type.contains("json") {
        r#"{"status": "active", "version": 1}"#.to_owned()
    } else if lower_name.contains("email") || lower_name.contains("mail") {
        format!("user_{rand_num}@example.com")
    } else if lower_name.contains("username") || lower_name.contains("user_name") {
        format!("user_{rand_num}")
    } else if lower_name.contains("first_name") || lower_name.contains("firstname") {
        "Alex".to_owned()
    } else if lower_name.contains("last_name") || lower_name.contains("lastname") {
        "Morgan".to_owned()
    } else if lower_name.contains("full_name") || lower_name.contains("fullname") || lower_name == "name" {
        format!("Alex Morgan {}", rand_num % 100)
    } else if lower_name.contains("phone") || lower_name.contains("tel") || lower_name.contains("mobile") {
        format!("+1-555-{:04}", rand_num)
    } else if lower_name.contains("url") || lower_name.contains("website") || lower_name.contains("link") {
        format!("https://example.com/items/{rand_num}")
    } else if lower_name.contains("avatar")
        || lower_name.contains("image")
        || lower_name.contains("icon")
        || lower_name.contains("photo")
    {
        format!("https://picsum.photos/seed/{rand_num}/200")
    } else if lower_name.contains("slug") || lower_name.contains("code") {
        format!("item-{rand_num}")
    } else if lower_name.contains("title") || lower_name.contains("headline") || lower_name.contains("subject") {
        format!("Sample Title {rand_num}")
    } else if lower_name.contains("desc")
        || lower_name.contains("content")
        || lower_name.contains("note")
        || lower_name.contains("bio")
        || lower_name.contains("comment")
        || lower_name.contains("body")
    {
        format!("Sample description for {column_name}")
    } else if lower_name.contains("status") || lower_name.contains("state") {
        "active".to_owned()
    } else if lower_name.contains("role") {
        "user".to_owned()
    } else if lower_name.contains("address") || lower_name.contains("street") {
        format!("{rand_num} Market Street")
    } else if lower_name.contains("city") {
        "San Francisco".to_owned()
    } else if lower_name.contains("country") {
        "US".to_owned()
    } else if lower_name.contains("ip") {
        format!("192.168.1.{}", rand_num % 254 + 1)
    } else {
        format!("{column_name}_{rand_num}")
    }
}

pub(crate) fn parse_insert_value(raw: &str, data_type: &str) -> Result<Option<UiCell>, String> {
    let normalized_type = data_type.to_ascii_lowercase();
    let value = raw.trim();
    if value.is_empty() && !is_text_type(&normalized_type) {
        return Ok(None);
    }
    if value.eq_ignore_ascii_case("null") {
        return Ok(Some(UiCell::Null));
    }
    if is_text_type(&normalized_type) {
        return Ok(Some(UiCell::Text(raw.to_owned())));
    }
    if is_binary_type(&normalized_type) {
        return Err("binary values require a binary editor; use NULL or a query parameter".to_owned());
    }
    if normalized_type.contains("uuid") || normalized_type.contains("guid") {
        return uuid::Uuid::parse_str(value)
            .map(|u| Some(UiCell::Text(u.to_string())))
            .map_err(|_| format!("{value} is not a valid UUID"));
    }
    if normalized_type.contains("bool") {
        return match value.to_ascii_lowercase().as_str() {
            "true" | "1" | "yes" => Ok(Some(UiCell::Boolean(true))),
            "false" | "0" | "no" => Ok(Some(UiCell::Boolean(false))),
            _ => Err("expected true or false".to_owned()),
        };
    }
    if normalized_type.contains("int") || normalized_type.contains("serial") {
        let parsed = if normalized_type.contains("smallint") || normalized_type.contains("int2") {
            value.parse::<i16>().map(|number| number.to_string())
        } else if normalized_type == "int" || normalized_type.contains("integer") || normalized_type.contains("int4") {
            value.parse::<i32>().map(|number| number.to_string())
        } else {
            value.parse::<i64>().map(|number| number.to_string())
        };
        return parsed
            .map(|number| Some(UiCell::Number(number)))
            .map_err(|_| format!("{value} is outside the range of {data_type}"));
    }
    if normalized_type.contains("real") || normalized_type.contains("float") || normalized_type.contains("double") {
        return value
            .parse::<f64>()
            .map(|number| Some(UiCell::Number(number.to_string())))
            .map_err(|_| format!("{value} is not a valid floating-point number"));
    }
    if is_decimal_type(&normalized_type) {
        return parse_decimal_value(value, data_type).map(Some);
    }
    if normalized_type.contains("json") {
        return serde_json::from_str::<serde_json::Value>(value)
            .map(|_| Some(UiCell::Json(value.to_owned())))
            .map_err(|_| "expected valid JSON".to_owned());
    }
    if normalized_type.contains("timestamptz")
        || normalized_type.contains("timestamp with time zone")
        || (normalized_type.contains("timestamp") && normalized_type.contains("timezone"))
    {
        return validate_timestamp_with_timezone(value).map(|_| Some(UiCell::Text(value.to_owned())));
    }
    if normalized_type.contains("timestamp") {
        return validate_timestamp(value).map(|_| Some(UiCell::Text(value.to_owned())));
    }
    if normalized_type == "date" || normalized_type.starts_with("date(") {
        return chrono::NaiveDate::parse_from_str(value, "%Y-%m-%d")
            .map(|_| Some(UiCell::Text(value.to_owned())))
            .map_err(|_| format!("{value} is not a valid date (expected YYYY-MM-DD)"));
    }
    if normalized_type.starts_with("time") {
        return validate_time(value).map(|_| Some(UiCell::Text(value.to_owned())));
    }
    Ok(Some(UiCell::Text(value.to_owned())))
}

pub(crate) fn parse_update_value(raw: &str, data_type: &str) -> Result<UiCell, String> {
    let normalized_type = data_type.to_ascii_lowercase();
    if raw.trim().eq_ignore_ascii_case("null") {
        return Ok(UiCell::Null);
    }
    if raw.is_empty() {
        if is_text_type(&normalized_type) {
            return Ok(UiCell::Text(String::new()));
        }
        return Err(format!("{} cannot be empty; use NULL to clear it", data_type));
    }
    if is_text_type(&normalized_type) {
        // Do not trim text: empty, whitespace-only, and NULL are distinct values.
        return Ok(UiCell::Text(raw.to_owned()));
    }
    parse_insert_value(raw, data_type).map(|value| value.unwrap_or(UiCell::Null))
}

pub(crate) fn is_text_type(normalized_type: &str) -> bool {
    normalized_type == "text"
        || normalized_type.starts_with("varchar")
        || normalized_type.starts_with("character varying")
        || normalized_type.starts_with("char")
        || normalized_type.starts_with("bpchar")
        || normalized_type == "citext"
}

fn is_binary_type(normalized_type: &str) -> bool {
    normalized_type.contains("bytea")
        || normalized_type.contains("blob")
        || normalized_type.contains("binary")
        || normalized_type.contains("varbinary")
}

fn validate_timestamp(value: &str) -> Result<(), String> {
    [
        "%Y-%m-%d %H:%M:%S%.f",
        "%Y-%m-%dT%H:%M:%S%.f",
        "%Y-%m-%d %H:%M:%S",
        "%Y-%m-%dT%H:%M:%S",
    ]
    .iter()
    .any(|format| chrono::NaiveDateTime::parse_from_str(value, format).is_ok())
    .then_some(())
    .ok_or_else(|| format!("{value} is not a valid timestamp"))
}

fn validate_timestamp_with_timezone(value: &str) -> Result<(), String> {
    chrono::DateTime::parse_from_rfc3339(value)
        .or_else(|_| chrono::DateTime::parse_from_str(value, "%Y-%m-%d %H:%M:%S%.f %:z"))
        .map(|_| ())
        .map_err(|_| format!("{value} is not a valid timestamptz (include a timezone offset)"))
}

fn validate_time(value: &str) -> Result<(), String> {
    if chrono::NaiveTime::parse_from_str(value, "%H:%M:%S%.f").is_ok()
        || chrono::NaiveTime::parse_from_str(value, "%H:%M:%S").is_ok()
    {
        return Ok(());
    }
    chrono::DateTime::parse_from_str(&format!("1970-01-01 {value}"), "%Y-%m-%d %H:%M:%S%.f %:z")
        .map(|_| ())
        .map_err(|_| format!("{value} is not a valid time"))
}

pub(crate) fn is_decimal_type(normalized_type: &str) -> bool {
    normalized_type
        .split('(')
        .next()
        .is_some_and(|name| matches!(name.trim(), "numeric" | "decimal"))
}

fn decimal_constraints(data_type: &str) -> Option<(u64, i64)> {
    let normalized_type = data_type.to_ascii_lowercase();
    if !is_decimal_type(&normalized_type) {
        return None;
    }
    let arguments = normalized_type.split_once('(')?.1.split_once(')')?.0;
    let mut parts = arguments.split(',').map(str::trim);
    let precision = parts.next()?.parse::<u64>().ok()?;
    let scale = parts.next().and_then(|part| part.parse::<i64>().ok()).unwrap_or(0);
    Some((precision, scale))
}

fn parse_decimal_value(value: &str, data_type: &str) -> Result<UiCell, String> {
    let decimal = value
        .parse::<BigDecimal>()
        .map_err(|_| format!("{value} is not a valid exact decimal"))?;
    let actual_scale = decimal.fractional_digit_count();
    if let Some((precision, declared_scale)) = decimal_constraints(data_type) {
        if actual_scale > declared_scale {
            return Err(format!("{value} has more than {declared_scale} fractional digits"));
        }
        let effective_precision = if actual_scale < 0 {
            decimal.digits().saturating_add((-actual_scale) as u64)
        } else {
            decimal.digits()
        };
        if effective_precision > precision {
            return Err(format!("{value} exceeds NUMERIC precision {precision}"));
        }
    }
    Ok(UiCell::Number(value.to_owned()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{UiTableColumn, UiTableInfo};

    fn info() -> UiTableInfo {
        UiTableInfo {
            schema: "public".to_owned(),
            name: "users".to_owned(),
            row_count: None,
            columns: vec![
                UiTableColumn {
                    name: "id".to_owned(),
                    data_type: "integer".to_owned(),
                    nullable: true,
                    is_primary_key: true,
                    is_identity: true,
                    ..Default::default()
                },
                UiTableColumn {
                    name: "name".to_owned(),
                    data_type: "text".to_owned(),
                    nullable: false,
                    ..Default::default()
                },
            ],
            primary_key: Some(vec!["id".to_owned()]),
            indexes: Vec::new(),
            foreign_keys: Vec::new(),
            check_constraints: Vec::new(),
            dependencies: Vec::new(),
        }
    }

    #[test]
    fn insert_row_mapping_skips_identity_and_preserves_text() {
        let table = info();

        let (columns, values) = parse_insert_row_values(&table, &[String::new(), "Ada".to_owned()]).expect("row");

        assert_eq!(columns, vec!["name"]);
        assert_eq!(values, vec![UiCell::Text("Ada".to_owned())]);
    }

    #[test]
    fn duplicate_row_mapping_clears_identity_and_copies_writable_values() {
        let table = info();
        let row = vec![UiCell::Number("7".to_owned()), UiCell::Text("Ada".to_owned())];

        let values = duplicate_row_values(&table, &row);

        assert_eq!(values, vec![String::new(), "Ada".to_owned()]);
    }
}
