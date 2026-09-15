use chrono::{DateTime, NaiveDate, NaiveDateTime, NaiveTime, Utc};
use db_pro_core::domain::error::DbError;
use db_pro_core::domain::query::{CellValue, ColumnMeta, QueryParam, QueryResult, Row};
use sqlx::mysql::types::MySqlTime;
use sqlx::mysql::{MySqlArguments, MySqlRow};
use sqlx::{Arguments, Column, Row as SqlxRow, TypeInfo, ValueRef};

pub struct MySqlQueryMapper;

/// Bind typed query parameters for MySQL's positional `?` placeholders.
pub fn bind_params(params: &[QueryParam], args: &mut MySqlArguments) -> Result<(), DbError> {
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
            // MySQL has no native UUID type in the shipped capability set; bind the
            // canonical text so a CHAR/VARCHAR/BINARY(16) column can still receive it.
            QueryParam::Uuid(v) => {
                uuid::Uuid::parse_str(v).map_err(|e| DbError::QueryFailed(format!("invalid UUID parameter: {e}")))?;
                args.add(v.as_str())
            }
            QueryParam::DateTime(v) => bind_mysql_datetime(args, v),
            QueryParam::Time(v) => {
                let time = chrono::NaiveTime::parse_from_str(v, "%H:%M:%S%.f")
                    .or_else(|_| chrono::NaiveTime::parse_from_str(v, "%H:%M:%S"))
                    .or_else(|_| chrono::NaiveTime::parse_from_str(v, "%H:%M"))
                    .map_err(|_| DbError::QueryFailed(format!("invalid time parameter: {v}")))?;
                args.add(time)
            }
            QueryParam::Interval(v) => {
                return Err(DbError::Unsupported(format!(
                    "MySQL does not bind INTERVAL parameters (got {v})"
                )));
            }
            QueryParam::Inet(v) => {
                return Err(DbError::Unsupported(format!(
                    "MySQL does not bind INET parameters (got {v})"
                )));
            }
            QueryParam::Json(v) => args.add(sqlx::types::Json(v)),
        };
        result.map_err(|e| DbError::QueryFailed(format!("failed to bind MySQL parameter: {e}")))?;
    }
    Ok(())
}

fn bind_mysql_datetime(args: &mut MySqlArguments, value: &str) -> Result<(), sqlx::error::BoxDynError> {
    if let Ok(dt) = DateTime::parse_from_rfc3339(value) {
        return args.add(dt.with_timezone(&Utc));
    }
    if let Ok(ndt) = NaiveDateTime::parse_from_str(value, "%Y-%m-%dT%H:%M:%S%.f") {
        return args.add(ndt);
    }
    if let Ok(ndt) = NaiveDateTime::parse_from_str(value, "%Y-%m-%dT%H:%M:%S") {
        return args.add(ndt);
    }
    if let Ok(ndt) = NaiveDateTime::parse_from_str(value, "%Y-%m-%d %H:%M:%S%.f") {
        return args.add(ndt);
    }
    if let Ok(ndt) = NaiveDateTime::parse_from_str(value, "%Y-%m-%d %H:%M:%S") {
        return args.add(ndt);
    }
    if let Ok(date) = NaiveDate::parse_from_str(value, "%Y-%m-%d") {
        return args.add(date);
    }
    Err(format!("invalid date/datetime parameter: {value}").into())
}

impl MySqlQueryMapper {
    pub fn map_rows(rows: Vec<sqlx::mysql::MySqlRow>) -> Result<QueryResult, DbError> {
        let mut result_rows = Vec::new();
        let mut columns: Vec<ColumnMeta> = Vec::new();

        for row in &rows {
            let cols = row.columns();
            if columns.is_empty() {
                columns = cols
                    .iter()
                    .map(|c| ColumnMeta {
                        name: c.name().to_string(),
                        data_type: c.type_info().name().to_string(),
                        // MySQL result metadata carries no per-column nullability flag.
                        nullable: true,
                    })
                    .collect();
            }
            let mut cells = Vec::with_capacity(cols.len());
            for (i, _col) in cols.iter().enumerate() {
                cells.push(Self::extract_cell(row, i)?);
            }
            result_rows.push(Row(cells));
        }

        let row_count = result_rows.len() as u64;

        Ok(QueryResult {
            columns,
            rows: result_rows,
            row_count,
            duration_ms: 0,
        })
    }

    /// Decodes one cell by the class the provider reports for its column, the way the
    /// PostgreSQL decoder does.
    ///
    /// The previous implementation probed Rust types in turn (`bool`, `i64`, `f64`,
    /// `String`, bytes) and kept the first the driver agreed to decode. That is not a
    /// decoding rule, and live measurement showed what it produced: the driver's `bool`
    /// decoder is `i8::decode(value) != 0` and its `bool` list accepts every integer class,
    /// so an integer that fits in an `i8` — `SELECT count(*)` among them — came back as
    /// `Bool`, `BIGINT UNSIGNED` above `i64::MAX` as `Bool(true)` (its payload read as
    /// `-1`), wider integers fell through to `Int64`, and the classes no probe accepted
    /// (DECIMAL, every date/time class, JSON, geometry) were replaced by the literal text
    /// `"unsupported"`.
    fn extract_cell(row: &MySqlRow, index: usize) -> Result<CellValue, DbError> {
        let raw = row.try_get_raw(index).map_err(crate::error::from_sqlx)?;
        if raw.is_null() {
            return Ok(CellValue::Null);
        }
        let data_type = row.columns()[index].type_info().name().to_string();
        decode_cell(row, index, &data_type)
    }
}

/// The class dispatch, keyed on the type name the provider reports for the column.
///
/// The arms cover every name `MySqlTypeInfo::name()` can produce (`sqlx-mysql 0.8.6`,
/// `protocol/text/column.rs:169`), so a value can only reach the byte-exact fallback by
/// being a class MySQL adds later than this list.
fn decode_cell(row: &MySqlRow, index: usize, data_type: &str) -> Result<CellValue, DbError> {
    match data_type {
        // MySQL has no boolean type: `BOOLEAN` is `TINYINT(1)`, which is the single name
        // the driver reports for it. Every other integer class is an integer.
        "BOOLEAN" => get_typed::<i8>(row, index).map(|value| CellValue::Bool(value != 0)),
        "TINYINT" | "SMALLINT" | "MEDIUMINT" | "INT" | "BIGINT" => get_typed::<i64>(row, index).map(CellValue::Int64),
        // The narrow unsigned classes all fit `i64`; the name carries the flag, so the
        // driver's own compatibility check applies.
        "TINYINT UNSIGNED" | "SMALLINT UNSIGNED" | "MEDIUMINT UNSIGNED" | "INT UNSIGNED" => {
            get_typed::<u64>(row, index).map(|value| CellValue::Int64(value as i64))
        }
        "BIGINT UNSIGNED" => get_typed::<u64>(row, index).map(unsigned_integer_cell),
        // `BIT(n)` is reported without its width, so the driver's unsigned gate cannot be
        // consulted here; the driver reads the raw big-endian payload for this class.
        "BIT" => get_ungated::<u64>(row, index).map(unsigned_integer_cell),
        "YEAR" => get_ungated::<u64>(row, index).map(|value| CellValue::Int64(value as i64)),
        "FLOAT" => get_typed::<f32>(row, index).map(|value| CellValue::Float64(f64::from(value))),
        "DOUBLE" => get_typed::<f64>(row, index).map(CellValue::Float64),
        // Exact digits and declared scale: `DECIMAL` never passes through `f64`.
        "DECIMAL" => {
            get_typed::<sqlx::types::BigDecimal>(row, index).map(|value| CellValue::Decimal(value.to_string()))
        }
        "DATE" => get_typed::<NaiveDate>(row, index).map(|value| CellValue::Date(value.to_string())),
        // `DATETIME` is a wall-clock reading that MySQL does not convert, so it carries no
        // marker; `TIMESTAMP` is an instant that MySQL renders in the session zone (pinned
        // to UTC by the driver), so it is normalized to UTC and marked as such.
        "DATETIME" => {
            get_typed::<NaiveDateTime>(row, index).map(|value| CellValue::Timestamp(format_naive_timestamp(value)))
        }
        "TIMESTAMP" => {
            get_typed::<DateTime<Utc>>(row, index).map(|value| CellValue::TimestampTz(format_utc_instant(value)))
        }
        "TIME" => get_typed::<MySqlTime>(row, index).map(|value| CellValue::Time(format_mysql_time(value))),
        "ENUM" | "CHAR" | "VARCHAR" | "TINYTEXT" | "TEXT" | "MEDIUMTEXT" | "LONGTEXT" => {
            get_typed::<String>(row, index).map(CellValue::Text)
        }
        // MySQL 8 reports a `SET` column as a string column (measured), so it arrives as
        // one of the names above; a server that reports the type itself is read the same
        // way, from its label list.
        "SET" => get_ungated::<String>(row, index).map(CellValue::Text),
        // Binary payloads stay byte-exact: a blob whose bytes happen to be valid UTF-8 is
        // still bytes, not text.
        "BINARY" | "VARBINARY" | "TINYBLOB" | "BLOB" | "MEDIUMBLOB" | "LONGBLOB" => {
            get_typed::<Vec<u8>>(row, index).map(CellValue::Bytes)
        }
        "JSON" => get_typed::<serde_json::Value>(row, index).map(CellValue::Json),
        // Structured classes have no domain variant: `GEOMETRY` (SRID + WKB) and anything a
        // server adds later are kept byte-exact, which the UI renders read-only as hex.
        // Never mojibake text, and never a failed query: the bytes are the value.
        _ => get_ungated::<Vec<u8>>(row, index).map(CellValue::Bytes),
    }
}

/// Decodes through the driver's own `Type::compatible` gate.
fn get_typed<'r, T>(row: &'r MySqlRow, index: usize) -> Result<T, DbError>
where
    T: sqlx::Decode<'r, sqlx::MySql> + sqlx::Type<sqlx::MySql>,
{
    row.try_get(index).map_err(crate::error::from_sqlx)
}

/// Decodes a class the dispatch above has already established, without the driver's
/// `Type::compatible` gate.
///
/// The gate keys on the column's own unsigned/binary flags, which the reported class name
/// does not always carry (`BIT`, `YEAR`, a `SET` reported as a set type). Skipping the gate
/// skips no decoding: the payload is still read by the decoder of the class that was
/// established from the type name.
fn get_ungated<'r, T>(row: &'r MySqlRow, index: usize) -> Result<T, DbError>
where
    T: sqlx::Decode<'r, sqlx::MySql>,
{
    row.try_get_unchecked(index).map_err(crate::error::from_sqlx)
}

/// Exact representation of an unsigned integer: `Int64` while it fits, and above
/// `i64::MAX` the exact digits as `Decimal` — the contract's variant for exact numeric
/// text. Never a wrapped negative, and never a float.
fn unsigned_integer_cell(value: u64) -> CellValue {
    match i64::try_from(value) {
        Ok(signed) => CellValue::Int64(signed),
        Err(_) => CellValue::Decimal(value.to_string()),
    }
}

/// Canonical string for a MySQL `TIMESTAMP`: the instant in UTC, marked as UTC.
///
/// `Z` rather than `+00:00`, matching `postgres::query_mapper::format_utc_instant`, so an
/// absolute instant has one shape whichever provider produced it.
fn format_utc_instant(value: DateTime<Utc>) -> String {
    value.to_rfc3339_opts(chrono::SecondsFormat::Micros, true)
}

/// Canonical string for a MySQL `DATETIME`: a wall-clock reading with no timezone.
///
/// MySQL's `DATETIME` carries no offset and is not converted by the session zone, so a
/// marker here would invent an instant the database never stored (#56 is the PostgreSQL
/// case of this defect). Microseconds are spelled out, matching
/// `postgres::query_mapper::format_naive_timestamp`.
fn format_naive_timestamp(value: NaiveDateTime) -> String {
    value.format("%Y-%m-%dT%H:%M:%S%.6f").to_string()
}

/// Canonical string for a MySQL `TIME`.
///
/// A time-of-day reading takes the same shape as the PostgreSQL `Time` variant
/// (`HH:MM:SS.ffffff`, no marker). MySQL's `TIME` is signed and spans ±838 h, so a value
/// that shape cannot hold — negative, or at least 24 h — keeps MySQL's own rendering
/// instead of being squeezed into a time of day that would silently change it.
///
/// The driver's `TryFrom<MySqlTime> for NaiveTime` drops the sign of a negative sub-day
/// value (`-00:30:00` converts to `00:30:00`), so the conversion is used only when the
/// rendering has no sign to lose.
fn format_mysql_time(value: MySqlTime) -> String {
    let rendered = format!("{value:.6}");
    if rendered.starts_with('-') {
        return rendered;
    }
    NaiveTime::try_from(value)
        .map(|time| time.to_string())
        .unwrap_or(rendered)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unsigned_integer_stays_exact_above_i64_max() {
        assert!(matches!(unsigned_integer_cell(0), CellValue::Int64(0)));
        assert!(matches!(unsigned_integer_cell(255), CellValue::Int64(255)));
        assert!(matches!(
            unsigned_integer_cell(4_294_967_295),
            CellValue::Int64(4_294_967_295)
        ));
        assert!(matches!(
            unsigned_integer_cell(i64::MAX as u64),
            CellValue::Int64(9_223_372_036_854_775_807)
        ));
        assert!(matches!(
            unsigned_integer_cell(u64::MAX),
            CellValue::Decimal(value) if value == "18446744073709551615"
        ));
        assert!(matches!(
            unsigned_integer_cell(9_223_372_036_854_775_808),
            CellValue::Decimal(value) if value == "9223372036854775808"
        ));
    }

    #[test]
    fn naive_timestamp_carries_no_timezone_marker() {
        let value = NaiveDate::from_ymd_opt(2024, 3, 15)
            .unwrap()
            .and_hms_micro_opt(10, 20, 30, 123_456)
            .unwrap();

        assert_eq!(format_naive_timestamp(value), "2024-03-15T10:20:30.123456");
        assert!(!format_naive_timestamp(value).contains('Z'));
        assert!(!format_naive_timestamp(value).contains('+'));
    }

    #[test]
    fn naive_timestamp_spells_out_whole_second_fraction() {
        let value = NaiveDate::from_ymd_opt(2024, 3, 15)
            .unwrap()
            .and_hms_opt(10, 20, 30)
            .unwrap();

        assert_eq!(format_naive_timestamp(value), "2024-03-15T10:20:30.000000");
    }

    #[test]
    fn utc_instant_keeps_explicit_utc_marker() {
        let value = DateTime::parse_from_rfc3339("2024-03-15T08:20:30.123456Z")
            .unwrap()
            .with_timezone(&Utc);

        assert_eq!(format_utc_instant(value), "2024-03-15T08:20:30.123456Z");
    }

    #[test]
    fn utc_instant_is_normalized_to_utc() {
        let value = DateTime::parse_from_rfc3339("2024-03-15T10:20:30.123456+02:00")
            .unwrap()
            .with_timezone(&Utc);

        assert_eq!(format_utc_instant(value), "2024-03-15T08:20:30.123456Z");
    }

    /// Builds a `MySqlTime` for `HH:MM:SS.ffffff`, negated when `negative` is set.
    fn mysql_time_of(hours: u32, minutes: u32, seconds: u32, microseconds: u32, negative: bool) -> MySqlTime {
        let magnitude = chrono::TimeDelta::hours(i64::from(hours))
            + chrono::TimeDelta::minutes(i64::from(minutes))
            + chrono::TimeDelta::seconds(i64::from(seconds))
            + chrono::TimeDelta::microseconds(i64::from(microseconds));
        let delta = if negative { -magnitude } else { magnitude };
        MySqlTime::try_from(delta).expect("in range")
    }

    /// The `TIME` class is dual-purpose in MySQL: a time of day is rendered the way the
    /// PostgreSQL `Time` variant is, while an interval-shaped value keeps its own shape
    /// (sign and hours) instead of losing either to a time of day.
    #[test]
    fn mysql_time_keeps_times_of_day_and_interval_shapes() {
        assert_eq!(
            format_mysql_time(mysql_time_of(10, 20, 30, 123_456, false)),
            "10:20:30.123456"
        );

        assert_eq!(format_mysql_time(MySqlTime::ZERO), "00:00:00");

        assert_eq!(
            format_mysql_time(mysql_time_of(800, 59, 59, 123_456, false)),
            "800:59:59.123456"
        );
    }

    /// A negative sub-day `TIME` must not lose its sign, which is exactly what the
    /// driver's `TryFrom<MySqlTime> for NaiveTime` does: `-00:30:00` converts to a
    /// `NaiveTime` of `00:30:00`, so the formatter has to render that case itself.
    #[test]
    fn mysql_time_never_drops_the_sign_of_a_negative_interval() {
        assert_eq!(format_mysql_time(mysql_time_of(0, 30, 0, 0, true)), "-0:30:00.000000");

        assert_eq!(
            format_mysql_time(mysql_time_of(100, 30, 15, 500_000, true)),
            "-100:30:15.500000"
        );
    }

    #[test]
    fn bind_params_accepts_common_scalar_types() {
        use db_pro_core::domain::query::QueryParam;
        let mut args = sqlx::mysql::MySqlArguments::default();
        let params = [
            QueryParam::Null,
            QueryParam::Bool(true),
            QueryParam::Int64(42),
            QueryParam::Float64(1.5),
            QueryParam::Decimal("12.34".into()),
            QueryParam::Text("hello".into()),
            QueryParam::Bytes(vec![1, 2, 3]),
            QueryParam::Uuid("550e8400-e29b-41d4-a716-446655440000".into()),
            QueryParam::DateTime("2024-03-15T10:20:30.123456".into()),
            QueryParam::Time("10:20:30.123456".into()),
            QueryParam::Json(serde_json::json!({"a": 1})),
        ];
        bind_params(&params, &mut args).expect("bind common types");
    }

    #[test]
    fn bind_params_rejects_interval_and_inet() {
        use db_pro_core::domain::query::QueryParam;
        let mut args = sqlx::mysql::MySqlArguments::default();
        let err = bind_params(&[QueryParam::Interval("1 day".into())], &mut args).expect_err("interval");
        assert!(err.to_string().contains("INTERVAL"));
        let err = bind_params(&[QueryParam::Inet("127.0.0.1".into())], &mut args).expect_err("inet");
        assert!(err.to_string().contains("INET"));
    }
}
