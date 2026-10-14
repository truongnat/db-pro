//! Tests for `../query_mapper.rs` (moved out of the source file).
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
