use chrono::{NaiveDate, NaiveTime};
use serde::{Deserialize, Serialize};

/// Semantic class for PostgreSQL temporal values.
///
/// These classes deliberately keep timezone-free values separate from absolute
/// instants so callers cannot accidentally reinterpret DATE/TIME/TIMESTAMP
/// through the local browser/OS timezone.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TemporalKind {
    Date,
    Time,
    TimeTz,
    Timestamp,
    TimestampTz,
}

impl TemporalKind {
    pub fn wire_name(self) -> &'static str {
        match self {
            Self::Date => "date",
            Self::Time => "time",
            Self::TimeTz => "timetz",
            Self::Timestamp => "timestamp",
            Self::TimestampTz => "timestamptz",
        }
    }
}

/// Canonical PostgreSQL temporal value used by the Gate 5 contract.
///
/// Canonical wire/display/copy/export strings:
/// - DATE: `YYYY-MM-DD`
/// - TIME: `HH:MM:SS.ffffff`
/// - TIMETZ: `HH:MM:SS.ffffff±HH:MM`
/// - TIMESTAMP: `YYYY-MM-DDTHH:MM:SS.ffffff` (never an invented `Z`/offset)
/// - TIMESTAMPTZ: `YYYY-MM-DDTHH:MM:SS.ffffffZ` (absolute instant in UTC)
///
/// Six fractional digits preserve PostgreSQL microsecond precision
/// deterministically across Rust, Tauri and JavaScript boundaries.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TemporalValue {
    kind: TemporalKind,
    value: String,
}

impl TemporalValue {
    pub fn new(kind: TemporalKind, value: impl Into<String>) -> Result<Self, String> {
        let value = value.into();
        validate_canonical(kind, &value)?;
        Ok(Self { kind, value })
    }

    pub fn kind(&self) -> TemporalKind {
        self.kind
    }

    pub fn value(&self) -> &str {
        &self.value
    }

    /// Display, copy and export all use the exact canonical string. No local
    /// timezone conversion or JavaScript `Date` parsing is part of the contract.
    pub fn as_text(&self) -> &str {
        &self.value
    }
}

fn validate_canonical(kind: TemporalKind, value: &str) -> Result<(), String> {
    match kind {
        TemporalKind::Date => validate_date(value),
        TemporalKind::Time => validate_time(value),
        TemporalKind::TimeTz => validate_timetz(value),
        TemporalKind::Timestamp => validate_timestamp(value),
        TemporalKind::TimestampTz => validate_timestamptz(value),
    }
    .map_err(|reason| format!("invalid canonical {} value `{value}`: {reason}", kind.wire_name()))
}

fn validate_date(value: &str) -> Result<(), &'static str> {
    if !value.is_ascii() || value.len() != 10 {
        return Err("expected YYYY-MM-DD");
    }
    NaiveDate::parse_from_str(value, "%Y-%m-%d")
        .map(|_| ())
        .map_err(|_| "invalid calendar date")
}

fn validate_time(value: &str) -> Result<(), &'static str> {
    if !value.is_ascii()
        || value.len() != 15
        || value.as_bytes().get(2) != Some(&b':')
        || value.as_bytes().get(5) != Some(&b':')
        || value.as_bytes().get(8) != Some(&b'.')
        || !value.as_bytes()[9..].iter().all(u8::is_ascii_digit)
    {
        return Err("expected HH:MM:SS.ffffff");
    }
    NaiveTime::parse_from_str(value, "%H:%M:%S%.6f")
        .map(|_| ())
        .map_err(|_| "invalid wall-clock time")
}

fn validate_offset(value: &str) -> Result<(), &'static str> {
    if !value.is_ascii()
        || value.len() != 6
        || !matches!(value.as_bytes()[0], b'+' | b'-')
        || value.as_bytes()[3] != b':'
        || !value.as_bytes()[1..3].iter().all(u8::is_ascii_digit)
        || !value.as_bytes()[4..6].iter().all(u8::is_ascii_digit)
    {
        return Err("expected ±HH:MM offset");
    }

    let hours: i32 = value[1..3].parse().map_err(|_| "invalid offset hour")?;
    let minutes: i32 = value[4..6].parse().map_err(|_| "invalid offset minute")?;
    if hours > 23 || minutes > 59 {
        return Err("offset out of range");
    }
    Ok(())
}

fn validate_timetz(value: &str) -> Result<(), &'static str> {
    if !value.is_ascii() || value.len() != 21 {
        return Err("expected HH:MM:SS.ffffff±HH:MM");
    }
    validate_time(&value[..15])?;
    validate_offset(&value[15..])
}

fn validate_timestamp(value: &str) -> Result<(), &'static str> {
    if !value.is_ascii() || value.len() != 26 || value.as_bytes().get(10) != Some(&b'T') {
        return Err("expected YYYY-MM-DDTHH:MM:SS.ffffff without timezone");
    }
    validate_date(&value[..10])?;
    validate_time(&value[11..])
}

fn validate_timestamptz(value: &str) -> Result<(), &'static str> {
    if !value.is_ascii() || value.len() != 27 || !value.ends_with('Z') {
        return Err("expected UTC YYYY-MM-DDTHH:MM:SS.ffffffZ");
    }
    validate_timestamp(&value[..26])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn canonical_temporal_values_round_trip_without_semantic_conversion() {
        let cases = [
            (TemporalKind::Date, "date", "2026-08-17"),
            (TemporalKind::Time, "time", "23:59:59.123456"),
            (TemporalKind::TimeTz, "timetz", "23:59:59.123456+05:30"),
            (TemporalKind::Timestamp, "timestamp", "2026-08-17T23:59:59.123456"),
            (TemporalKind::TimestampTz, "timestamptz", "2026-08-17T18:29:59.123456Z"),
        ];

        for (kind, wire_name, expected) in cases {
            let value = TemporalValue::new(kind, expected).unwrap();
            assert_eq!(value.kind(), kind);
            assert_eq!(value.value(), expected);
            assert_eq!(value.as_text(), expected);

            let json = serde_json::to_value(&value).unwrap();
            assert_eq!(json["kind"], wire_name);
            assert_eq!(json["value"], expected);

            let decoded: TemporalValue = serde_json::from_value(json).unwrap();
            assert_eq!(decoded, value);
        }
    }

    #[test]
    fn timestamp_rejects_invented_timezone_semantics() {
        assert!(TemporalValue::new(TemporalKind::Timestamp, "2026-08-17T23:59:59.123456Z").is_err());
        assert!(TemporalValue::new(TemporalKind::Timestamp, "2026-08-17T23:59:59.123456+07:00").is_err());
    }

    #[test]
    fn timestamptz_requires_canonical_utc_representation() {
        assert!(TemporalValue::new(TemporalKind::TimestampTz, "2026-08-17T23:59:59.123456+05:30").is_err());
        assert!(TemporalValue::new(TemporalKind::TimestampTz, "2026-08-17T18:29:59.123456Z").is_ok());
    }

    #[test]
    fn fractional_microseconds_are_required_and_preserved() {
        assert!(TemporalValue::new(TemporalKind::Time, "12:34:56").is_err());
        assert!(TemporalValue::new(TemporalKind::Time, "12:34:56.123").is_err());
        let value = TemporalValue::new(TemporalKind::Time, "12:34:56.123400").unwrap();
        assert_eq!(value.as_text(), "12:34:56.123400");
    }

    #[test]
    fn date_and_time_do_not_accept_timezone_suffixes() {
        assert!(TemporalValue::new(TemporalKind::Date, "2026-08-17Z").is_err());
        assert!(TemporalValue::new(TemporalKind::Time, "12:34:56.123456+07:00").is_err());
    }

    #[test]
    fn malformed_non_ascii_inputs_fail_without_panicking() {
        assert!(TemporalValue::new(TemporalKind::TimeTz, "ééééééééééé").is_err());
        assert!(TemporalValue::new(TemporalKind::Timestamp, "ééééééééééééé").is_err());
    }
}
