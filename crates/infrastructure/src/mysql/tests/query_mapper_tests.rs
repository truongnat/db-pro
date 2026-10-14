//! Tests for `../query_mapper.rs` (moved out of the source file).
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
// cc-scan:allow TOO_MANY_PARAMS — context params passed through
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
