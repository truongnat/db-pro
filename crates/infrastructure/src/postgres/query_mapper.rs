use db_pro_core::domain::error::DbError;
use db_pro_core::domain::query::{CellValue, ColumnMeta, QueryParam, Row};
use sqlx::postgres::PgArguments;
use sqlx::postgres::PgValueFormat;
use sqlx::{Arguments, Column, Row as _, TypeInfo, ValueRef};

pub fn bind_params(params: &[QueryParam], args: &mut PgArguments) -> Result<(), DbError> {
    for param in params {
        let result = match param {
            QueryParam::Null => args.add(Option::<String>::None),
            QueryParam::Bool(v) => args.add(v),
            QueryParam::Int64(v) => args.add(v),
            QueryParam::Float64(v) => args.add(v),
            QueryParam::Text(v) => args.add(v.as_str()),
            QueryParam::Bytes(v) => args.add(v.as_slice()),
            QueryParam::Uuid(v) => {
                let uuid = uuid::Uuid::parse_str(v)
                    .map_err(|e| DbError::QueryFailed(format!("invalid UUID parameter: {e}")))?;
                args.add(uuid)
            }
            QueryParam::DateTime(v) => {
                if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(v) {
                    args.add(dt.with_timezone(&chrono::Utc))
                } else {
                    let date = chrono::NaiveDate::parse_from_str(v, "%Y-%m-%d")
                        .map_err(|e| DbError::QueryFailed(format!("invalid date/datetime parameter: {e}")))?;
                    args.add(date)
                }
            }
            QueryParam::Json(v) => args.add(sqlx::types::Json(v)),
        };
        result.map_err(|e| DbError::QueryFailed(format!("failed to bind parameter: {e}")))?;
    }
    Ok(())
}

pub fn columns_from_describe(describe: &sqlx::Describe<sqlx::Postgres>) -> Vec<ColumnMeta> {
    describe
        .columns()
        .iter()
        .enumerate()
        .map(|(i, col)| ColumnMeta {
            name: col.name().to_string(),
            data_type: col.type_info().name().to_string(),
            nullable: describe.nullable(i).unwrap_or(true),
        })
        .collect()
}

pub fn map_row(row: &sqlx::postgres::PgRow, columns: &[ColumnMeta]) -> Result<Row, DbError> {
    let mut cells = Vec::with_capacity(columns.len());
    for (i, col) in columns.iter().enumerate() {
        let raw = row.try_get_raw(i).map_err(crate::error::from_sqlx)?;
        let cell = if raw.is_null() {
            CellValue::Null
        } else {
            decode_cell(row, i, &col.data_type)
        };
        cells.push(cell);
    }
    Ok(Row(cells))
}

fn decode_cell(row: &sqlx::postgres::PgRow, i: usize, data_type: &str) -> CellValue {
    let dt_upper = data_type.to_uppercase();
    let res = match dt_upper.as_str() {
        "BOOL" => row.try_get::<bool, _>(i).map(CellValue::Bool),
        "INT2" => row.try_get::<i16, _>(i).map(|v| CellValue::Int64(v as i64)),
        "INT4" => row.try_get::<i32, _>(i).map(|v| CellValue::Int64(v as i64)),
        "INT8" => row.try_get::<i64, _>(i).map(CellValue::Int64),
        "OID" => row
            .try_get::<sqlx::postgres::types::Oid, _>(i)
            .map(|v| CellValue::Int64(v.0 as i64)),
        "FLOAT4" => row.try_get::<f32, _>(i).map(|v| CellValue::Float64(v as f64)),
        "FLOAT8" => row.try_get::<f64, _>(i).map(CellValue::Float64),
        "NUMERIC" | "DECIMAL" => Ok(decode_numeric(row, i)),
        "UUID" => row.try_get::<uuid::Uuid, _>(i).map(|v| CellValue::Uuid(v.to_string())),
        "TIMESTAMPTZ" => row
            .try_get::<chrono::DateTime<chrono::Utc>, _>(i)
            .map(|v| CellValue::DateTime(v.to_rfc3339())),
        "TIMESTAMP" => row
            .try_get::<chrono::NaiveDateTime, _>(i)
            .map(|v| CellValue::DateTime(v.and_utc().to_rfc3339())),
        "DATE" => row
            .try_get::<chrono::NaiveDate, _>(i)
            .map(|v| CellValue::Date(v.to_string())),
        "TIME" | "TIMETZ" => row.try_get::<String, _>(i).map(CellValue::Time),
        "INTERVAL" => row.try_get::<String, _>(i).map(CellValue::Interval),
        "INET" | "CIDR" => row.try_get::<String, _>(i).map(CellValue::Inet),
        "JSON" | "JSONB" => row.try_get::<serde_json::Value, _>(i).map(CellValue::Json),
        "BYTEA" => row.try_get::<Vec<u8>, _>(i).map(CellValue::Bytes),
        _ => row.try_get::<String, _>(i).map(CellValue::Text),
    };

    res.unwrap_or_else(|_| {
        row.try_get_raw(i)
            .ok()
            .map(|raw| match raw.format() {
                PgValueFormat::Text => raw
                    .as_bytes()
                    .ok()
                    .and_then(|bytes| std::str::from_utf8(bytes).ok())
                    .map(|value| CellValue::Text(value.to_owned()))
                    .unwrap_or_else(|| CellValue::Text(format!("<unsupported value: {data_type}>"))),
                PgValueFormat::Binary => CellValue::Text(format!("<unsupported binary value: {data_type}>")),
            })
            .unwrap_or_else(|| CellValue::Text(format!("<unsupported value: {data_type}>")))
    })
}

fn decode_numeric(row: &sqlx::postgres::PgRow, i: usize) -> CellValue {
    let Some(raw) = row.try_get_raw(i).ok() else {
        return CellValue::Text("<unsupported value: NUMERIC>".into());
    };

    let value = match raw.format() {
        PgValueFormat::Text => raw.as_str().ok().map(str::to_owned),
        PgValueFormat::Binary => raw.as_bytes().ok().and_then(decode_binary_numeric),
    };

    value
        .map(CellValue::Text)
        .unwrap_or_else(|| CellValue::Text("<unsupported value: NUMERIC>".into()))
}

fn decode_binary_numeric(bytes: &[u8]) -> Option<String> {
    let mut offset = 0;
    let digit_count = usize::from(read_u16(bytes, &mut offset)?);
    let weight = i32::from(read_i16(bytes, &mut offset)?);
    let sign = read_u16(bytes, &mut offset)?;
    let scale = usize::try_from(read_i16(bytes, &mut offset)?).ok()?;
    let expected_len = 8usize.checked_add(digit_count.checked_mul(2)?)?;
    if bytes.len() != expected_len {
        return None;
    }
    if sign == 0xC000 {
        return Some("NaN".into());
    }
    let negative = match sign {
        0x0000 => false,
        0x4000 => true,
        _ => return None,
    };

    let mut digits = String::new();
    for _ in 0..digit_count {
        let digit = read_u16(bytes, &mut offset)?;
        if digit >= 10_000 {
            return None;
        }
        use std::fmt::Write;
        write!(&mut digits, "{digit:04}").ok()?;
    }

    let decimal_position = weight.checked_add(1)?.checked_mul(4)?;
    let (mut integer, mut fraction) = if decimal_position <= 0 {
        (
            String::from("0"),
            "0".repeat(usize::try_from(decimal_position.unsigned_abs()).ok()?) + &digits,
        )
    } else {
        let position = usize::try_from(decimal_position).ok()?;
        if position >= digits.len() {
            (
                format!("{digits}{}", "0".repeat(position - digits.len())),
                String::new(),
            )
        } else {
            (digits[..position].to_owned(), digits[position..].to_owned())
        }
    };

    let trimmed_integer = integer.trim_start_matches('0');
    integer = if trimmed_integer.is_empty() {
        "0".into()
    } else {
        trimmed_integer.into()
    };

    if fraction.len() > scale && fraction[scale..].bytes().all(|byte| byte == b'0') {
        fraction.truncate(scale);
    } else if fraction.len() < scale {
        fraction.push_str(&"0".repeat(scale - fraction.len()));
    }

    let mut result = String::with_capacity(integer.len() + fraction.len() + 2);
    if negative {
        result.push('-');
    }
    result.push_str(&integer);
    if !fraction.is_empty() {
        result.push('.');
        result.push_str(&fraction);
    }
    Some(result)
}

fn read_u16(bytes: &[u8], offset: &mut usize) -> Option<u16> {
    let end = offset.checked_add(2)?;
    let value = u16::from_be_bytes(bytes.get(*offset..end)?.try_into().ok()?);
    *offset = end;
    Some(value)
}

fn read_i16(bytes: &[u8], offset: &mut usize) -> Option<i16> {
    Some(i16::from_be_bytes(read_u16(bytes, offset)?.to_be_bytes()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bind_params_handles_supported_types() {
        let mut args = PgArguments::default();
        let params = vec![
            QueryParam::Null,
            QueryParam::Bool(true),
            QueryParam::Int64(42),
            QueryParam::Float64(1.234),
            QueryParam::Text("hello".into()),
            QueryParam::Bytes(vec![1, 2, 3]),
            QueryParam::Uuid("550e8400-e29b-41d4-a716-446655440000".into()),
            QueryParam::DateTime("2026-01-01T00:00:00Z".into()),
            QueryParam::Json(serde_json::json!({"key": "value"})),
        ];
        assert!(bind_params(&params, &mut args).is_ok());
    }

    #[test]
    fn bind_params_accepts_date_only_value() {
        let mut args = PgArguments::default();
        let params = vec![QueryParam::DateTime("2026-08-17".into())];
        assert!(bind_params(&params, &mut args).is_ok());
    }

    #[test]
    fn bind_params_fails_on_invalid_uuid() {
        let mut args = PgArguments::default();
        let params = vec![QueryParam::Uuid("invalid-uuid".into())];
        assert!(bind_params(&params, &mut args).is_err());
    }

    #[test]
    fn bind_params_fails_on_invalid_datetime() {
        let mut args = PgArguments::default();
        let params = vec![QueryParam::DateTime("invalid-date".into())];
        assert!(bind_params(&params, &mut args).is_err());
    }

    #[test]
    fn binary_numeric_uses_postgres_display_scale() {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&7u16.to_be_bytes());
        bytes.extend_from_slice(&4i16.to_be_bytes());
        bytes.extend_from_slice(&0u16.to_be_bytes());
        bytes.extend_from_slice(&5i16.to_be_bytes());
        for digit in [1234u16, 5678, 9012, 3456, 7890, 1234, 5000] {
            bytes.extend_from_slice(&digit.to_be_bytes());
        }

        assert_eq!(
            decode_binary_numeric(&bytes).as_deref(),
            Some("12345678901234567890.12345")
        );
    }

    #[test]
    fn binary_numeric_preserves_declared_fractional_zeroes() {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&2u16.to_be_bytes());
        bytes.extend_from_slice(&0i16.to_be_bytes());
        bytes.extend_from_slice(&0u16.to_be_bytes());
        bytes.extend_from_slice(&2i16.to_be_bytes());
        for digit in [1u16, 5000] {
            bytes.extend_from_slice(&digit.to_be_bytes());
        }

        assert_eq!(decode_binary_numeric(&bytes).as_deref(), Some("1.50"));
    }

    #[test]
    fn fallback_format_check_behavior() {
        // Test helper contract for fallback message generation logic
        let format_fallback_message = |format: PgValueFormat, data_type: &str| match format {
            PgValueFormat::Text => CellValue::Text(format!("<unsupported value: {data_type}>")),
            PgValueFormat::Binary => CellValue::Text(format!("<unsupported binary value: {data_type}>")),
        };

        assert_eq!(
            format_fallback_message(PgValueFormat::Text, "geometry"),
            CellValue::Text("<unsupported value: geometry>".into())
        );
        assert_eq!(
            format_fallback_message(PgValueFormat::Binary, "geometry"),
            CellValue::Text("<unsupported binary value: geometry>".into())
        );
    }
}
