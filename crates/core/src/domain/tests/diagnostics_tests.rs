//! Tests for `../diagnostics.rs` (moved out of the source file).
use super::*;

#[test]
fn placeholder_has_basic_info() {
    let diag = DiagnosticsSummary::placeholder();
    assert!(!diag.app_version.is_empty());
    assert!(!diag.os.is_empty());
    assert!(!diag.architecture.is_empty());
    assert_eq!(diag.drivers.len(), 3);
}

#[test]
fn redact_sensitive_password() {
    assert_eq!(redact_sensitive("password=hunter2"), "***");
}

#[test]
fn redact_sensitive_normal() {
    assert_eq!(redact_sensitive("SELECT 1"), "SELECT 1");
}

#[test]
fn redact_sensitive_token() {
    assert_eq!(redact_sensitive("bearer token_value"), "***");
}

#[test]
fn redact_connection_url_strips_embedded_password() {
    assert_eq!(
        redact_connection_url("postgres://alice:s3cret@localhost:5432/app"),
        "postgres://***@localhost:5432/app"
    );
    assert_eq!(
        redact_connection_url("mysql://root:hunter2@127.0.0.1/db"),
        "mysql://***@127.0.0.1/db"
    );
}

#[test]
fn diagnostic_event_serializes() {
    let event = DiagnosticEvent {
        event: "query.execute.started".into(),
        timestamp: "2024-01-01T00:00:00Z".into(),
        level: DiagnosticLevel::Info,
        module: "query_service".into(),
        operation_id: Some("op-123".into()),
        connection_id: Some("conn-456".into()),
        duration_ms: None,
        details: None,
    };
    let json = serde_json::to_string(&event).unwrap();
    assert!(json.contains("query.execute.started"));
    assert!(!json.contains("password"));
}
