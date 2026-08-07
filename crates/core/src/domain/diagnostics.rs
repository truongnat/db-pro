use serde::{Deserialize, Serialize};

/// Structured diagnostic event for logging and support bundles.
///
/// Events follow a dot-notation naming convention:
/// - `query.execute.started`
/// - `query.execute.completed`
/// - `metadata.refresh.started`
/// - `metadata.refresh.failed`
/// - `connection.open.failed`
///
/// **Never** include passwords, SQL values, credentials, or full connection strings.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiagnosticEvent {
    /// Dot-notation event name (e.g. `query.execute.started`).
    pub event: String,
    /// Timestamp in RFC 3339 format.
    pub timestamp: String,
    /// Log level: trace, debug, info, warn, error.
    pub level: DiagnosticLevel,
    /// Module that emitted the event (e.g. `query_service`, `connector`).
    pub module: String,
    /// Optional operation ID for correlation.
    pub operation_id: Option<String>,
    /// Optional connection ID (never includes credentials).
    pub connection_id: Option<String>,
    /// Duration in milliseconds (for completed operations).
    pub duration_ms: Option<u64>,
    /// Optional structured details (must be redacted of secrets).
    pub details: Option<serde_json::Value>,
}

/// Severity level for diagnostic events.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DiagnosticLevel {
    Trace,
    Debug,
    Info,
    Warn,
    Error,
}

/// System-wide diagnostic summary for support bundles.
///
/// This is the data that would be included in a `db-pro-diagnostics.zip`.
/// All fields are sanitized — no credentials or sensitive data.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiagnosticsSummary {
    /// Application version.
    pub app_version: String,
    /// Operating system (e.g. "macOS 14.5").
    pub os: String,
    /// CPU architecture (e.g. "aarch64").
    pub architecture: String,
    /// Database drivers available.
    pub drivers: Vec<DriverDiagnostic>,
    /// Active connection summaries (no credentials).
    pub connections: Vec<ConnectionDiagnostic>,
    /// Recent errors (last N, redacted).
    pub recent_errors: Vec<ErrorDiagnostic>,
    /// Runtime status.
    pub runtime: RuntimeDiagnostic,
    /// Schema version of the persistence store.
    pub schema_version: u32,
}

/// Diagnostic info about a database driver.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DriverDiagnostic {
    pub driver: String,
    pub available: bool,
}

/// Diagnostic info about a connection (no credentials).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionDiagnostic {
    pub connection_id: String,
    pub driver: String,
    pub host: String,
    pub port: u16,
    pub database: String,
    pub username: String,
    pub has_password: bool,
    pub has_ssh: bool,
    pub is_connected: bool,
}

/// Diagnostic info about a recent error (redacted).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorDiagnostic {
    pub timestamp: String,
    pub error_code: String,
    pub message: String,
    pub module: String,
}

/// Runtime status information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeDiagnostic {
    pub active_connections: usize,
    pub active_executions: usize,
    pub uptime_seconds: u64,
}

impl DiagnosticsSummary {
    /// Create a summary with redacted/placeholder values.
    /// In production, populate from actual runtime state.
    pub fn placeholder() -> Self {
        Self {
            app_version: env!("CARGO_PKG_VERSION").to_string(),
            os: std::env::consts::OS.to_string(),
            architecture: std::env::consts::ARCH.to_string(),
            drivers: vec![
                DriverDiagnostic {
                    driver: "postgres".into(),
                    available: true,
                },
                DriverDiagnostic {
                    driver: "sqlite".into(),
                    available: true,
                },
            ],
            connections: Vec::new(),
            recent_errors: Vec::new(),
            runtime: RuntimeDiagnostic {
                active_connections: 0,
                active_executions: 0,
                uptime_seconds: 0,
            },
            schema_version: 0,
        }
    }
}

/// Redact a string for safe inclusion in diagnostics.
///
/// Replaces the value with `"***"` if it looks like a credential.
pub fn redact_sensitive(value: &str) -> String {
    // Simple heuristic: if it contains common password/secret patterns, redact.
    let lower = value.to_ascii_lowercase();
    if lower.contains("password")
        || lower.contains("secret")
        || lower.contains("token")
        || lower.contains("private_key")
        || lower.contains("passphrase")
    {
        "***".to_string()
    } else {
        value.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn placeholder_has_basic_info() {
        let diag = DiagnosticsSummary::placeholder();
        assert!(!diag.app_version.is_empty());
        assert!(!diag.os.is_empty());
        assert!(!diag.architecture.is_empty());
        assert_eq!(diag.drivers.len(), 2);
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
}
