use chrono::{DateTime, FixedOffset, NaiveDate, NaiveDateTime, NaiveTime};
use db_pro_core::domain::error::DbError;
use db_pro_core::domain::query::{CellValue, QueryParam};
use tiberius::{ColumnData, Row};

/// Bind the canonical parameter model to SQL Server's `@pN` parameters.
pub fn bind_query<'a>(sql: &'a str, params: &'a [QueryParam]) -> Result<tiberius::Query<'a>, DbError> {
    let mut query = tiberius::Query::new(sql);
    for param in params {
        match param {
            QueryParam::Null => query.bind(Option::<String>::None),
            QueryParam::Bool(value) => query.bind(*value),
            QueryParam::Int64(value) => query.bind(*value),
            QueryParam::Float64(value) => query.bind(*value),
            QueryParam::Decimal(value) => {
                let decimal = value
                    .parse::<tiberius::numeric::BigDecimal>()
                    .map_err(|error| DbError::QueryFailed(format!("invalid SQL Server decimal parameter: {error}")))?;
                query.bind(decimal);
            }
            QueryParam::Text(value) => query.bind(value.as_str()),
            QueryParam::Bytes(value) => query.bind(value.as_slice()),
            QueryParam::Uuid(value) => {
                let uuid = uuid::Uuid::parse_str(value)
                    .map_err(|error| DbError::QueryFailed(format!("invalid SQL Server UUID parameter: {error}")))?;
                query.bind(uuid);
            }
            QueryParam::DateTime(value) => bind_datetime(&mut query, value)?,
            QueryParam::Time(value) => {
                let time = parse_time(value)?;
                query.bind(time);
            }
            QueryParam::Json(value) => query.bind(value.to_string()),
            QueryParam::Interval(value) | QueryParam::Inet(value) => {
                return Err(DbError::Unsupported(format!(
                    "SQL Server does not bind {value} as a native parameter"
                )));
            }
        }
    }
    Ok(query)
}

fn bind_datetime<'a>(query: &mut tiberius::Query<'a>, value: &str) -> Result<(), DbError> {
    if let Ok(datetime) = DateTime::parse_from_rfc3339(value) {
        query.bind(datetime);
        return Ok(());
    }
    if let Ok(datetime) = NaiveDateTime::parse_from_str(value, "%Y-%m-%dT%H:%M:%S%.f") {
        query.bind(datetime);
        return Ok(());
    }
    if let Ok(date) = NaiveDate::parse_from_str(value, "%Y-%m-%d") {
        query.bind(date);
        return Ok(());
    }
    Err(DbError::QueryFailed(format!(
        "invalid SQL Server datetime parameter: {value}"
    )))
}

fn parse_time(value: &str) -> Result<NaiveTime, DbError> {
    NaiveTime::parse_from_str(value, "%H:%M:%S%.f")
        .or_else(|_| NaiveTime::parse_from_str(value, "%H:%M:%S"))
        .or_else(|_| NaiveTime::parse_from_str(value, "%H:%M"))
        .map_err(|_| DbError::QueryFailed(format!("invalid SQL Server time parameter: {value}")))
}

/// Decode a TDS cell by its concrete variant, preserving exact numeric and binary values.
pub fn decode_cell(row: &Row, index: usize) -> Result<CellValue, DbError> {
    let (_, value) = row
        .cells()
        .nth(index)
        .ok_or_else(|| DbError::QueryFailed(format!("SQL Server row has no column at index {index}")))?;

    match value {
        ColumnData::U8(value) => value.map_or(Ok(CellValue::Null), |value| Ok(CellValue::Int64(i64::from(value)))),
        ColumnData::I16(value) => value.map_or(Ok(CellValue::Null), |value| Ok(CellValue::Int64(i64::from(value)))),
        ColumnData::I32(value) => value.map_or(Ok(CellValue::Null), |value| Ok(CellValue::Int64(i64::from(value)))),
        ColumnData::I64(value) => value.map_or(Ok(CellValue::Null), |value| Ok(CellValue::Int64(value))),
        ColumnData::F32(value) => value.map_or(Ok(CellValue::Null), |value| Ok(CellValue::Float64(f64::from(value)))),
        ColumnData::F64(value) => value.map_or(Ok(CellValue::Null), |value| Ok(CellValue::Float64(value))),
        ColumnData::Bit(value) => value.map_or(Ok(CellValue::Null), |value| Ok(CellValue::Bool(value))),
        ColumnData::String(value) => value
            .as_ref()
            .map_or(Ok(CellValue::Null), |value| Ok(CellValue::Text(value.to_string()))),
        ColumnData::Guid(value) => value
            .as_ref()
            .map_or(Ok(CellValue::Null), |value| Ok(CellValue::Uuid(value.to_string()))),
        ColumnData::Binary(value) => value
            .as_ref()
            .map_or(Ok(CellValue::Null), |value| Ok(CellValue::Bytes(value.to_vec()))),
        ColumnData::Numeric(value) => value
            .as_ref()
            .map_or(Ok(CellValue::Null), |value| Ok(CellValue::Decimal(value.to_string()))),
        ColumnData::Xml(value) => value
            .as_ref()
            .map_or(Ok(CellValue::Null), |value| Ok(CellValue::Text(value.to_string()))),
        ColumnData::DateTime(_) | ColumnData::SmallDateTime(_) | ColumnData::DateTime2(_) => row
            .try_get::<NaiveDateTime, _>(index)
            .map_err(|error| DbError::QueryFailed(format!("SQL Server datetime decode failed: {error}")))?
            .map_or(Ok(CellValue::Null), |value| {
                Ok(CellValue::Timestamp(value.format("%Y-%m-%dT%H:%M:%S%.6f").to_string()))
            }),
        ColumnData::Date(value) => {
            if value.is_none() {
                return Ok(CellValue::Null);
            }
            row.try_get::<NaiveDate, _>(index)
                .map_err(|error| DbError::QueryFailed(format!("SQL Server date decode failed: {error}")))?
                .map_or(Ok(CellValue::Null), |value| Ok(CellValue::Date(value.to_string())))
        }
        ColumnData::Time(value) => {
            if value.is_none() {
                return Ok(CellValue::Null);
            }
            row.try_get::<NaiveTime, _>(index)
                .map_err(|error| DbError::QueryFailed(format!("SQL Server time decode failed: {error}")))?
                .map_or(Ok(CellValue::Null), |value| Ok(CellValue::Time(value.to_string())))
        }
        ColumnData::DateTimeOffset(value) => {
            if value.is_none() {
                return Ok(CellValue::Null);
            }
            row.try_get::<DateTime<FixedOffset>, _>(index)
                .map_err(|error| DbError::QueryFailed(format!("SQL Server datetimeoffset decode failed: {error}")))?
                .map_or(Ok(CellValue::Null), |value| {
                    Ok(CellValue::TimestampTz(value.to_rfc3339()))
                })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_unsupported_interval_and_inet_parameters() {
        let error = bind_query("SELECT @p1", &[QueryParam::Interval("1 day".into())]).unwrap_err();
        assert!(matches!(error, DbError::Unsupported(_)));
    }

    #[test]
    fn parses_sql_server_datetime_parameter_shapes() {
        assert!(bind_query("SELECT @p1", &[QueryParam::DateTime("2026-09-17".into())]).is_ok());
        assert!(bind_query(
            "SELECT @p1",
            &[QueryParam::DateTime("2026-09-17T12:34:56+07:00".into())]
        )
        .is_ok());
    }
}
