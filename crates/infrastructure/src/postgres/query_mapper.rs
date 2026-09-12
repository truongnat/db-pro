use std::net::{Ipv4Addr, Ipv6Addr};

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
            QueryParam::Decimal(v) => {
                let decimal = v
                    .parse::<sqlx::types::BigDecimal>()
                    .map_err(|error| DbError::QueryFailed(format!("invalid decimal parameter: {error}")))?;
                args.add(decimal)
            }
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
                } else if let Ok(dt) = chrono::DateTime::parse_from_str(v, "%Y-%m-%d %H:%M:%S%z") {
                    args.add(dt.with_timezone(&chrono::Utc))
                } else if let Ok(ndt) = chrono::NaiveDateTime::parse_from_str(v, "%Y-%m-%d %H:%M:%S%.f") {
                    args.add(ndt.and_utc())
                } else if let Ok(ndt) = chrono::NaiveDateTime::parse_from_str(v, "%Y-%m-%d %H:%M:%S") {
                    args.add(ndt.and_utc())
                } else if let Ok(ndt) = chrono::NaiveDateTime::parse_from_str(v, "%Y-%m-%dT%H:%M:%S%.f") {
                    args.add(ndt.and_utc())
                } else if let Ok(ndt) = chrono::NaiveDateTime::parse_from_str(v, "%Y-%m-%dT%H:%M:%S") {
                    args.add(ndt.and_utc())
                } else if let Ok(date) = chrono::NaiveDate::parse_from_str(v, "%Y-%m-%d") {
                    args.add(date)
                } else {
                    return Err(DbError::QueryFailed(format!("invalid date/datetime parameter: {v}")));
                }
            }
            QueryParam::Time(v) => match parse_time_parameter(v)? {
                PostgresTimeParameter::Time(time) => args.add(time),
                PostgresTimeParameter::TimeWithOffset(time) => args.add(time),
            },
            QueryParam::Interval(v) => args.add(parse_interval_parameter(v)?),
            QueryParam::Inet(v) => {
                let network = v
                    .parse::<ipnetwork::IpNetwork>()
                    .map_err(|error| DbError::QueryFailed(format!("invalid PostgreSQL INET parameter: {error}")))?;
                args.add(network)
            }
            QueryParam::Json(v) => args.add(sqlx::types::Json(v)),
        };
        result.map_err(|e| DbError::QueryFailed(format!("failed to bind parameter: {e}")))?;
    }
    Ok(())
}

enum PostgresTimeParameter {
    Time(chrono::NaiveTime),
    TimeWithOffset(sqlx::postgres::types::PgTimeTz<chrono::NaiveTime, chrono::FixedOffset>),
}

fn parse_time_parameter(value: &str) -> Result<PostgresTimeParameter, DbError> {
    let time_with_offset = parse_time_with_offset_parameter(value);
    if let Ok(time) = time_with_offset {
        return Ok(PostgresTimeParameter::TimeWithOffset(time));
    }

    for format in ["%H:%M:%S%.f", "%H:%M"] {
        if let Ok(time) = chrono::NaiveTime::parse_from_str(value, format) {
            return Ok(PostgresTimeParameter::Time(time));
        }
    }

    Err(DbError::QueryFailed(format!(
        "invalid PostgreSQL TIME parameter: {value}"
    )))
}

fn parse_time_with_offset_parameter(
    value: &str,
) -> Result<sqlx::postgres::types::PgTimeTz<chrono::NaiveTime, chrono::FixedOffset>, DbError> {
    let mut value_with_date = String::with_capacity(value.len() + 11);
    value_with_date.push_str("2001-07-08 ");
    value_with_date.push_str(value);

    for format in ["%Y-%m-%d %H:%M:%S%.f%#z", "%Y-%m-%d %H:%M:%S%#z", "%Y-%m-%d %H:%M%#z"] {
        if let Ok(parsed) = chrono::DateTime::parse_from_str(&value_with_date, format) {
            return Ok(sqlx::postgres::types::PgTimeTz {
                time: parsed.time(),
                offset: *parsed.offset(),
            });
        }
    }

    Err(DbError::QueryFailed(format!(
        "invalid PostgreSQL TIMETZ parameter: {value}"
    )))
}

fn parse_interval_parameter(value: &str) -> Result<sqlx::postgres::types::PgInterval, DbError> {
    let tokens: Vec<&str> = value.split_whitespace().collect();
    if tokens.is_empty() {
        return Err(DbError::QueryFailed(
            "invalid PostgreSQL INTERVAL parameter: empty value".into(),
        ));
    }

    let mut interval = sqlx::postgres::types::PgInterval::default();
    let mut token_index = 0;
    while token_index < tokens.len() {
        let token = tokens[token_index];
        if token.contains(':') {
            if token_index + 1 != tokens.len() {
                return Err(DbError::QueryFailed(format!(
                    "invalid PostgreSQL INTERVAL parameter: unexpected token after {token}"
                )));
            }
            let component = parse_interval_time(token)?;
            interval.microseconds = interval
                .microseconds
                .checked_add(component)
                .ok_or_else(|| DbError::QueryFailed("PostgreSQL INTERVAL time value is out of range".into()))?;
            token_index += 1;
            continue;
        }

        let unit = tokens.get(token_index + 1).ok_or_else(|| {
            DbError::QueryFailed(format!(
                "invalid PostgreSQL INTERVAL parameter: missing unit after {token}"
            ))
        })?;
        add_interval_component(&mut interval, token, unit)?;
        token_index += 2;
    }

    Ok(interval)
}

fn add_interval_component(
    interval: &mut sqlx::postgres::types::PgInterval,
    magnitude: &str,
    unit: &str,
) -> Result<(), DbError> {
    let normalized_unit = unit.to_ascii_lowercase();
    let target = match normalized_unit.as_str() {
        "year" | "years" => IntervalComponent::Months(12),
        "mon" | "mons" | "month" | "months" => IntervalComponent::Months(1),
        "week" | "weeks" => IntervalComponent::Days(7),
        "day" | "days" => IntervalComponent::Days(1),
        "hour" | "hours" => IntervalComponent::Microseconds(3_600_000_000),
        "minute" | "minutes" => IntervalComponent::Microseconds(60_000_000),
        "second" | "seconds" => IntervalComponent::Microseconds(1_000_000),
        _ => {
            return Err(DbError::QueryFailed(format!(
                "invalid PostgreSQL INTERVAL unit: {unit}"
            )))
        }
    };

    match target {
        IntervalComponent::Months(multiplier) => {
            let component = parse_integer_component(magnitude, "months")?;
            let months = component
                .checked_mul(multiplier)
                .ok_or_else(|| DbError::QueryFailed("PostgreSQL INTERVAL months value is out of range".into()))?;
            interval.months = interval
                .months
                .checked_add(months)
                .ok_or_else(|| DbError::QueryFailed("PostgreSQL INTERVAL months value is out of range".into()))?;
        }
        IntervalComponent::Days(multiplier) => {
            let component = parse_integer_component(magnitude, "days")?;
            let days = component
                .checked_mul(multiplier)
                .ok_or_else(|| DbError::QueryFailed("PostgreSQL INTERVAL days value is out of range".into()))?;
            interval.days = interval
                .days
                .checked_add(days)
                .ok_or_else(|| DbError::QueryFailed("PostgreSQL INTERVAL days value is out of range".into()))?;
        }
        IntervalComponent::Microseconds(multiplier) => {
            let component = parse_scaled_component(magnitude, multiplier)?;
            interval.microseconds = interval
                .microseconds
                .checked_add(component)
                .ok_or_else(|| DbError::QueryFailed("PostgreSQL INTERVAL time value is out of range".into()))?;
        }
    }

    Ok(())
}

enum IntervalComponent {
    Months(i32),
    Days(i32),
    Microseconds(i64),
}

fn parse_integer_component(value: &str, component: &str) -> Result<i32, DbError> {
    value.parse::<i32>().map_err(|error| {
        DbError::QueryFailed(format!(
            "invalid PostgreSQL INTERVAL {component} value {value}: {error}"
        ))
    })
}

fn parse_scaled_component(value: &str, multiplier: i64) -> Result<i64, DbError> {
    let (sign, unsigned) = match value.as_bytes().first() {
        Some(b'-') => (-1i64, &value[1..]),
        Some(b'+') => (1i64, &value[1..]),
        _ => (1i64, value),
    };
    let (integer, fraction) = unsigned.split_once('.').unwrap_or((unsigned, ""));
    if fraction.len() > 6 || fraction.bytes().any(|byte| !byte.is_ascii_digit()) {
        return Err(DbError::QueryFailed(format!(
            "invalid PostgreSQL INTERVAL fractional value: {value}"
        )));
    }
    let integer = if integer.is_empty() {
        0
    } else {
        integer
            .parse::<i64>()
            .map_err(|error| DbError::QueryFailed(format!("invalid PostgreSQL INTERVAL value {value}: {error}")))?
    };
    let fraction_value = if fraction.is_empty() {
        0
    } else {
        let fraction_length = u32::try_from(fraction.len())
            .map_err(|_| DbError::QueryFailed("PostgreSQL INTERVAL fractional value is too long".into()))?;
        let denominator = 10i64.pow(fraction_length);
        if multiplier % denominator != 0 {
            return Err(DbError::QueryFailed(
                "PostgreSQL INTERVAL precision exceeds microseconds".into(),
            ));
        }
        let numerator = fraction.parse::<i64>().map_err(|error| {
            DbError::QueryFailed(format!("invalid PostgreSQL INTERVAL fractional value {value}: {error}"))
        })?;
        numerator
            .checked_mul(multiplier / denominator)
            .ok_or_else(|| DbError::QueryFailed("PostgreSQL INTERVAL value is out of range".into()))?
    };
    let scaled_integer = integer
        .checked_mul(multiplier)
        .ok_or_else(|| DbError::QueryFailed("PostgreSQL INTERVAL value is out of range".into()))?;
    sign.checked_mul(
        scaled_integer
            .checked_add(fraction_value)
            .ok_or_else(|| DbError::QueryFailed("PostgreSQL INTERVAL value is out of range".into()))?,
    )
    .ok_or_else(|| DbError::QueryFailed("PostgreSQL INTERVAL value is out of range".into()))
}

fn parse_interval_time(value: &str) -> Result<i64, DbError> {
    let (sign, unsigned) = match value.as_bytes().first() {
        Some(b'-') => (-1i64, &value[1..]),
        Some(b'+') => (1i64, &value[1..]),
        _ => (1i64, value),
    };
    let fields: Vec<&str> = unsigned.split(':').collect();
    if fields.len() != 3 {
        return Err(DbError::QueryFailed(format!(
            "invalid PostgreSQL INTERVAL time value: {value}"
        )));
    }
    let hours = fields[0]
        .parse::<i64>()
        .map_err(|error| DbError::QueryFailed(format!("invalid PostgreSQL INTERVAL hour value {value}: {error}")))?;
    let minutes = fields[1]
        .parse::<i64>()
        .map_err(|error| DbError::QueryFailed(format!("invalid PostgreSQL INTERVAL minute value {value}: {error}")))?;
    if !(0..60).contains(&minutes) {
        return Err(DbError::QueryFailed(format!(
            "invalid PostgreSQL INTERVAL minute value: {value}"
        )));
    }
    let seconds = parse_scaled_component(fields[2], 1_000_000)?;
    if !(0..60_000_000).contains(&seconds) {
        return Err(DbError::QueryFailed(format!(
            "invalid PostgreSQL INTERVAL second value: {value}"
        )));
    }
    let total = hours
        .checked_mul(3_600_000_000)
        .and_then(|value| value.checked_add(minutes * 60_000_000))
        .and_then(|value| value.checked_add(seconds))
        .ok_or_else(|| DbError::QueryFailed("PostgreSQL INTERVAL time value is out of range".into()))?;
    sign.checked_mul(total)
        .ok_or_else(|| DbError::QueryFailed("PostgreSQL INTERVAL time value is out of range".into()))
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
            decode_cell(row, i, &col.data_type)?
        };
        cells.push(cell);
    }
    Ok(Row(cells))
}

fn decode_cell(row: &sqlx::postgres::PgRow, i: usize, data_type: &str) -> Result<CellValue, DbError> {
    let dt_upper = data_type.to_uppercase();
    match dt_upper.as_str() {
        "BOOL" => row
            .try_get::<bool, _>(i)
            .map(CellValue::Bool)
            .map_err(crate::error::from_sqlx),
        "INT2" => row
            .try_get::<i16, _>(i)
            .map(|v| CellValue::Int64(v as i64))
            .map_err(crate::error::from_sqlx),
        "INT4" => row
            .try_get::<i32, _>(i)
            .map(|v| CellValue::Int64(v as i64))
            .map_err(crate::error::from_sqlx),
        "INT8" => row
            .try_get::<i64, _>(i)
            .map(CellValue::Int64)
            .map_err(crate::error::from_sqlx),
        "OID" => row
            .try_get::<sqlx::postgres::types::Oid, _>(i)
            .map(|v| CellValue::Int64(v.0 as i64))
            .map_err(crate::error::from_sqlx),
        "FLOAT4" => row
            .try_get::<f32, _>(i)
            .map(|v| CellValue::Float64(v as f64))
            .map_err(crate::error::from_sqlx),
        "FLOAT8" => row
            .try_get::<f64, _>(i)
            .map(CellValue::Float64)
            .map_err(crate::error::from_sqlx),
        "NUMERIC" | "DECIMAL" => decode_numeric(row, i),
        "UUID" => row
            .try_get::<uuid::Uuid, _>(i)
            .map(|v| CellValue::Uuid(v.to_string()))
            .map_err(crate::error::from_sqlx),
        "TIMESTAMPTZ" => row
            .try_get::<chrono::DateTime<chrono::Utc>, _>(i)
            .map(|v| CellValue::DateTime(v.to_rfc3339()))
            .map_err(crate::error::from_sqlx),
        "TIMESTAMP" => row
            .try_get::<chrono::NaiveDateTime, _>(i)
            .map(|v| CellValue::DateTime(v.and_utc().to_rfc3339()))
            .map_err(crate::error::from_sqlx),
        "DATE" => row
            .try_get::<chrono::NaiveDate, _>(i)
            .map(|v| CellValue::Date(v.to_string()))
            .map_err(crate::error::from_sqlx),
        "TIME" => row
            .try_get::<chrono::NaiveTime, _>(i)
            .map(|v| CellValue::Time(v.to_string()))
            .map_err(crate::error::from_sqlx),
        "TIMETZ" => row
            .try_get::<sqlx::postgres::types::PgTimeTz<chrono::NaiveTime, chrono::FixedOffset>, _>(i)
            .map(|v| CellValue::Time(format_time_with_offset(v.time, v.offset)))
            .map_err(crate::error::from_sqlx),
        "INTERVAL" => row
            .try_get::<sqlx::postgres::types::PgInterval, _>(i)
            .map(|v| CellValue::Interval(format_interval(v)))
            .map_err(crate::error::from_sqlx),
        "INET" | "CIDR" => decode_inet(row, i),
        "JSON" | "JSONB" => row
            .try_get::<serde_json::Value, _>(i)
            .map(CellValue::Json)
            .map_err(crate::error::from_sqlx),
        "BYTEA" => row
            .try_get::<Vec<u8>, _>(i)
            .map(CellValue::Bytes)
            .map_err(crate::error::from_sqlx),
        _ => decode_textual_value(row, i, data_type),
    }
}

fn decode_textual_value(row: &sqlx::postgres::PgRow, i: usize, data_type: &str) -> Result<CellValue, DbError> {
    let raw = row.try_get_raw(i).map_err(crate::error::from_sqlx)?;
    let value = raw
        .as_str()
        .map(str::to_owned)
        .map_err(|error| DbError::QueryFailed(format!("cannot decode PostgreSQL {data_type} as text: {error}")))?;
    Ok(CellValue::Text(value))
}

fn decode_numeric(row: &sqlx::postgres::PgRow, i: usize) -> Result<CellValue, DbError> {
    let raw = row.try_get_raw(i).map_err(crate::error::from_sqlx)?;

    let value = match raw.format() {
        PgValueFormat::Text => raw
            .as_str()
            .map(str::to_owned)
            .map_err(|error| DbError::QueryFailed(format!("invalid PostgreSQL NUMERIC text: {error}"))),
        PgValueFormat::Binary => raw
            .as_bytes()
            .map_err(|error| DbError::QueryFailed(format!("invalid PostgreSQL NUMERIC binary value: {error}")))
            .and_then(|bytes| {
                decode_binary_numeric(bytes)
                    .ok_or_else(|| DbError::QueryFailed("invalid PostgreSQL NUMERIC binary value".into()))
            }),
    };

    value.map(CellValue::Decimal)
}

fn decode_inet(row: &sqlx::postgres::PgRow, i: usize) -> Result<CellValue, DbError> {
    let raw = row.try_get_raw(i).map_err(crate::error::from_sqlx)?;
    let value = match raw.format() {
        PgValueFormat::Text => raw
            .as_str()
            .map(str::to_owned)
            .map_err(|error| DbError::QueryFailed(format!("invalid PostgreSQL INET text: {error}"))),
        PgValueFormat::Binary => raw
            .as_bytes()
            .map_err(|error| DbError::QueryFailed(format!("invalid PostgreSQL INET binary value: {error}")))
            .and_then(decode_binary_inet),
    }?;
    Ok(CellValue::Inet(value))
}

fn decode_binary_inet(bytes: &[u8]) -> Result<String, DbError> {
    if bytes.len() < 4 {
        return Err(DbError::QueryFailed("invalid PostgreSQL INET binary value".into()));
    }
    let family = bytes[0];
    let prefix = bytes[1];
    let address_len = usize::from(bytes[3]);
    let expected_len = 4usize
        .checked_add(address_len)
        .ok_or_else(|| DbError::QueryFailed("invalid PostgreSQL INET address length".into()))?;
    if bytes.len() != expected_len {
        return Err(DbError::QueryFailed("invalid PostgreSQL INET binary length".into()));
    }
    let address = &bytes[4..];
    match (family, address_len) {
        (2, 4) => Ok(format!(
            "{}/{}",
            Ipv4Addr::new(address[0], address[1], address[2], address[3]),
            prefix
        )),
        (3, 16) => {
            let octets: [u8; 16] = address
                .try_into()
                .map_err(|_| DbError::QueryFailed("invalid PostgreSQL INET IPv6 length".into()))?;
            Ok(format!("{}/{}", Ipv6Addr::from(octets), prefix))
        }
        _ => Err(DbError::QueryFailed(
            "unsupported PostgreSQL INET address family".into(),
        )),
    }
}

fn format_time_with_offset(time: chrono::NaiveTime, offset: chrono::FixedOffset) -> String {
    let seconds = offset.local_minus_utc();
    let sign = if seconds >= 0 { '+' } else { '-' };
    let absolute = seconds.unsigned_abs();
    format!("{time}{sign}{:02}:{:02}", absolute / 3_600, (absolute % 3_600) / 60)
}

fn format_interval(interval: sqlx::postgres::types::PgInterval) -> String {
    let mut parts = Vec::new();
    if interval.months != 0 {
        parts.push(format!("{} mons", interval.months));
    }
    if interval.days != 0 {
        parts.push(format!("{} days", interval.days));
    }
    if interval.microseconds != 0 {
        let negative = interval.microseconds < 0;
        let absolute = interval.microseconds.unsigned_abs();
        let hours = absolute / 3_600_000_000;
        let minutes = (absolute % 3_600_000_000) / 60_000_000;
        let seconds = (absolute % 60_000_000) / 1_000_000;
        let micros = absolute % 1_000_000;
        let time = if micros == 0 {
            format!("{hours:02}:{minutes:02}:{seconds:02}")
        } else {
            format!("{hours:02}:{minutes:02}:{seconds:02}.{micros:06}")
        };
        parts.push(if negative { format!("-{time}") } else { time });
    }
    if parts.is_empty() {
        "00:00:00".into()
    } else {
        parts.join(" ")
    }
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
            QueryParam::Decimal("1234567890123456.1234".into()),
            QueryParam::Text("hello".into()),
            QueryParam::Bytes(vec![1, 2, 3]),
            QueryParam::Uuid("550e8400-e29b-41d4-a716-446655440000".into()),
            QueryParam::DateTime("2026-01-01T00:00:00Z".into()),
            QueryParam::Time("12:34:56.123456".into()),
            QueryParam::Interval("1 days 02:00:00".into()),
            QueryParam::Inet("192.0.2.1/24".into()),
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
    fn parses_local_time_and_time_with_offset() {
        assert!(matches!(
            parse_time_parameter("12:34:56.123456"),
            Ok(PostgresTimeParameter::Time(time)) if time.to_string() == "12:34:56.123456"
        ));
        assert!(matches!(
            parse_time_parameter("12:34:56.123456+02:00"),
            Ok(PostgresTimeParameter::TimeWithOffset(time))
                if time.time.to_string() == "12:34:56.123456"
                    && time.offset.local_minus_utc() == 7_200
        ));
    }

    #[test]
    fn parses_interval_components_without_precision_loss() {
        let interval = parse_interval_parameter("1 mon 2 days 03:04:05.123456").unwrap();
        assert_eq!(interval.months, 1);
        assert_eq!(interval.days, 2);
        assert_eq!(interval.microseconds, 11_045_123_456);
    }

    #[test]
    fn rejects_invalid_interval_values() {
        assert!(parse_interval_parameter("1 day 60:00:00.1234567").is_err());
        assert!(parse_interval_parameter("1 fortnights").is_err());
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
}
