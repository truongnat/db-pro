use serde::{Deserialize, Serialize};

use super::query::QueryError;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorEnvelope {
    pub code: String,
    pub message_id: String,
    pub message: String,
    pub details: Option<serde_json::Value>,
    pub request_id: Option<String>,
    pub retryable: bool,
    pub operation_id: Option<String>,
}

impl From<&QueryError> for ErrorEnvelope {
    fn from(err: &QueryError) -> Self {
        let db_err = DbError::from(err.clone());
        let retryable = db_err.retryable();
        Self {
            code: db_err.code().into(),
            message_id: db_err.message_id().into(),
            message: err.to_string(),
            details: None,
            request_id: None,
            retryable,
            operation_id: None,
        }
    }
}

impl ErrorEnvelope {
    pub fn with_request_id(mut self, request_id: Option<String>) -> Self {
        self.request_id = request_id;
        self
    }

    pub fn with_operation_id(mut self, operation_id: Option<String>) -> Self {
        self.operation_id = operation_id;
        self
    }
}

/// Type of constraint that was violated.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ConstraintType {
    Unique,
    ForeignKey,
    NotNull,
    Check,
}

/// Unified error taxonomy for the entire application.
///
/// Every error that crosses a service or transport boundary must be converted
/// into this type. Raw driver errors (sqlx, rusqlite) must never leak past
/// the infrastructure layer.
#[derive(thiserror::Error, Debug)]
pub enum DbError {
    // ── Connection ──────────────────────────────────────────────
    #[error("authentication failed: {0}")]
    AuthFailed(String),

    #[error("connection refused: {0}")]
    ConnectionRefused(String),

    #[error("connection failed: {0}")]
    ConnectionFailed(String),

    #[error("connection lost: {0}")]
    ConnectionLost(String),

    #[error("connection timeout: {0}")]
    ConnectionTimeout(String),

    #[error("database not found: {0}")]
    DatabaseNotFound(String),

    #[error("SSL error: {0}")]
    SslError(String),

    // ── Query ───────────────────────────────────────────────────
    #[error("query syntax error: {0}")]
    QuerySyntax(String),

    #[error("permission denied: {0}")]
    PermissionDenied(String),

    #[error("query timeout after {timeout_ms}ms")]
    QueryTimeout { timeout_ms: u64 },

    #[error("query cancelled")]
    QueryCancelled,

    #[error("query failed: {0}")]
    QueryFailed(String),

    // ── Schema / Introspection ──────────────────────────────────
    #[error("introspection failed: {0}")]
    IntrospectionFailed(String),

    #[error("schema operation failed: {0}")]
    SchemaFailed(String),

    #[error("unsupported operation: {0}")]
    Unsupported(String),

    // ── Data ────────────────────────────────────────────────────
    #[error("data operation failed: {0}")]
    DataFailed(String),

    #[error("constraint violation ({constraint_type:?}) on {table}: {message}")]
    ConstraintViolation {
        constraint_type: ConstraintType,
        constraint: String,
        table: String,
        column: Option<String>,
        message: String,
    },

    #[error("not found: {0}")]
    NotFound(String),

    // ── Validation ──────────────────────────────────────────────
    #[error("validation: {0}")]
    Validation(String),

    // ── Safety ──────────────────────────────────────────────────
    #[error("read-only violation: {0}")]
    ReadOnlyViolation(String),

    // ── Infrastructure ──────────────────────────────────────────
    #[error("io error: {0}")]
    Io(String),

    #[error("encryption failed: {0}")]
    EncryptionFailed(String),

    #[error("internal: {0}")]
    Internal(String),
}

impl DbError {
    /// Machine-readable error code for frontend consumption.
    /// Frontend should switch on this code, NOT on the message string.
    pub fn code(&self) -> &'static str {
        match self {
            Self::AuthFailed(_) => "DB_AUTH_FAILED",
            Self::ConnectionRefused(_) => "DB_CONNECTION_REFUSED",
            Self::ConnectionFailed(_) => "DB_CONNECTION_FAILED",
            Self::ConnectionLost(_) => "DB_CONNECTION_LOST",
            Self::ConnectionTimeout(_) => "DB_CONNECTION_TIMEOUT",
            Self::DatabaseNotFound(_) => "DB_DATABASE_NOT_FOUND",
            Self::SslError(_) => "DB_SSL_ERROR",
            Self::QuerySyntax(_) => "QUERY_SYNTAX_ERROR",
            Self::PermissionDenied(_) => "QUERY_PERMISSION_DENIED",
            Self::QueryTimeout { .. } => "QUERY_TIMEOUT",
            Self::QueryCancelled => "QUERY_CANCELLED",
            Self::QueryFailed(_) => "QUERY_FAILED",
            Self::IntrospectionFailed(_) => "INTROSPECTION_FAILED",
            Self::SchemaFailed(_) => "SCHEMA_FAILED",
            Self::Unsupported(_) => "OPERATION_UNSUPPORTED",
            Self::DataFailed(_) => "DATA_FAILED",
            Self::ConstraintViolation { .. } => "CONSTRAINT_VIOLATION",
            Self::NotFound(_) => "NOT_FOUND",
            Self::Validation(_) => "VALIDATION_ERROR",
            Self::ReadOnlyViolation(_) => "READ_ONLY_VIOLATION",
            Self::Io(_) => "IO_ERROR",
            Self::EncryptionFailed(_) => "ENCRYPTION_FAILED",
            Self::Internal(_) => "INTERNAL_ERROR",
        }
    }

    /// i18n message identifier for frontend localization.
    pub fn message_id(&self) -> &'static str {
        match self {
            Self::AuthFailed(_) => "error.db.auth_failed",
            Self::ConnectionRefused(_) => "error.db.connection_refused",
            Self::ConnectionFailed(_) => "error.db.connection_failed",
            Self::ConnectionLost(_) => "error.db.connection_lost",
            Self::ConnectionTimeout(_) => "error.db.connection_timeout",
            Self::DatabaseNotFound(_) => "error.db.database_not_found",
            Self::SslError(_) => "error.db.ssl_error",
            Self::QuerySyntax(_) => "error.query.syntax",
            Self::PermissionDenied(_) => "error.query.permission",
            Self::QueryTimeout { .. } => "error.query.timeout",
            Self::QueryCancelled => "error.query.cancelled",
            Self::QueryFailed(_) => "error.query.failed",
            Self::IntrospectionFailed(_) => "error.introspection.failed",
            Self::SchemaFailed(_) => "error.schema.failed",
            Self::Unsupported(_) => "error.operation.unsupported",
            Self::DataFailed(_) => "error.data.failed",
            Self::ConstraintViolation { .. } => "error.data.constraint_violation",
            Self::NotFound(_) => "error.not_found",
            Self::Validation(_) => "error.validation",
            Self::ReadOnlyViolation(_) => "error.safety.read_only",
            Self::Io(_) => "error.io",
            Self::EncryptionFailed(_) => "error.encryption.failed",
            Self::Internal(_) => "error.internal",
        }
    }

    /// Whether the operation can be retried.
    /// Frontend can use this to show a "Retry" button.
    pub fn retryable(&self) -> bool {
        matches!(
            self,
            Self::ConnectionRefused(_)
                | Self::ConnectionTimeout(_)
                | Self::QueryTimeout { .. }
                | Self::ConnectionLost(_)
                | Self::Io(_)
        )
    }

    /// Broad error category for grouping.
    pub fn category(&self) -> ErrorCategory {
        match self {
            Self::AuthFailed(_)
            | Self::ConnectionRefused(_)
            | Self::ConnectionFailed(_)
            | Self::ConnectionLost(_)
            | Self::ConnectionTimeout(_)
            | Self::DatabaseNotFound(_)
            | Self::SslError(_) => ErrorCategory::Connection,

            Self::QuerySyntax(_)
            | Self::PermissionDenied(_)
            | Self::QueryTimeout { .. }
            | Self::QueryCancelled
            | Self::QueryFailed(_) => ErrorCategory::Query,

            Self::IntrospectionFailed(_) | Self::SchemaFailed(_) | Self::Unsupported(_) => ErrorCategory::Schema,

            Self::DataFailed(_) | Self::ConstraintViolation { .. } | Self::NotFound(_) => ErrorCategory::Data,

            Self::Validation(_) => ErrorCategory::Validation,

            Self::ReadOnlyViolation(_) => ErrorCategory::Safety,

            Self::Io(_) | Self::EncryptionFailed(_) | Self::Internal(_) => ErrorCategory::Internal,
        }
    }
}

/// Broad error categories for frontend grouping/display.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ErrorCategory {
    Connection,
    Query,
    Schema,
    Data,
    Validation,
    Safety,
    Internal,
}

// ── Conversions ─────────────────────────────────────────────────

impl From<QueryError> for DbError {
    fn from(err: QueryError) -> Self {
        match err {
            QueryError::ConnectionNotFound { connection_id } => {
                DbError::ConnectionFailed(format!("connection {connection_id} is not active"))
            }
            QueryError::SyntaxError { line, message } => DbError::QuerySyntax(format!("line {line}: {message}")),
            QueryError::PermissionDenied { table } => DbError::PermissionDenied(format!("table {table}")),
            QueryError::Timeout { timeout_ms } => DbError::QueryTimeout { timeout_ms },
            QueryError::ConnectionLost => DbError::ConnectionLost("connection lost during query".into()),
            QueryError::MultiStatementDisabled => DbError::Validation("multi-statement execution is disabled".into()),
            QueryError::UnsupportedParameterType(t) => DbError::Unsupported(format!("parameter type: {t}")),
            QueryError::Validation(msg) => DbError::Validation(msg),
            QueryError::Internal(msg) => DbError::Internal(msg),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn error_codes_are_unique_and_stable() {
        let errors: Vec<DbError> = vec![
            DbError::AuthFailed("test".into()),
            DbError::ConnectionRefused("test".into()),
            DbError::ConnectionFailed("test".into()),
            DbError::ConnectionLost("test".into()),
            DbError::ConnectionTimeout("test".into()),
            DbError::DatabaseNotFound("test".into()),
            DbError::SslError("test".into()),
            DbError::QuerySyntax("test".into()),
            DbError::PermissionDenied("test".into()),
            DbError::QueryTimeout { timeout_ms: 1000 },
            DbError::QueryCancelled,
            DbError::QueryFailed("test".into()),
            DbError::IntrospectionFailed("test".into()),
            DbError::SchemaFailed("test".into()),
            DbError::Unsupported("test".into()),
            DbError::DataFailed("test".into()),
            DbError::ConstraintViolation {
                constraint_type: ConstraintType::Unique,
                constraint: "test".into(),
                table: "test".into(),
                column: None,
                message: "test".into(),
            },
            DbError::NotFound("test".into()),
            DbError::Validation("test".into()),
            DbError::ReadOnlyViolation("test".into()),
            DbError::Io("test".into()),
            DbError::EncryptionFailed("test".into()),
            DbError::Internal("test".into()),
        ];

        let mut codes: Vec<&str> = errors.iter().map(|e| e.code()).collect();
        let original_len = codes.len();
        codes.sort();
        codes.dedup();
        assert_eq!(codes.len(), original_len, "duplicate error codes detected");
    }

    #[test]
    fn retryable_errors() {
        assert!(DbError::ConnectionRefused("test".into()).retryable());
        assert!(DbError::ConnectionTimeout("test".into()).retryable());
        assert!(DbError::QueryTimeout { timeout_ms: 1000 }.retryable());
        assert!(DbError::ConnectionLost("test".into()).retryable());
        assert!(DbError::Io("test".into()).retryable());
        assert!(!DbError::AuthFailed("test".into()).retryable());
        assert!(!DbError::QuerySyntax("test".into()).retryable());
        assert!(!DbError::Validation("test".into()).retryable());
        assert!(!DbError::ReadOnlyViolation("test".into()).retryable());
    }

    #[test]
    fn query_error_converts_to_correct_db_error() {
        let qe = QueryError::SyntaxError {
            line: 1,
            message: "unexpected token".into(),
        };
        let de = DbError::from(qe);
        assert!(matches!(de, DbError::QuerySyntax(_)));
        assert_eq!(de.code(), "QUERY_SYNTAX_ERROR");

        let qe = QueryError::Timeout { timeout_ms: 5000 };
        let de = DbError::from(qe);
        assert!(matches!(de, DbError::QueryTimeout { timeout_ms: 5000 }));
        assert_eq!(de.code(), "QUERY_TIMEOUT");
        assert!(de.retryable());
    }

    #[test]
    fn error_category_grouping() {
        assert_eq!(DbError::AuthFailed("x".into()).category(), ErrorCategory::Connection);
        assert_eq!(DbError::QuerySyntax("x".into()).category(), ErrorCategory::Query);
        assert_eq!(
            DbError::IntrospectionFailed("x".into()).category(),
            ErrorCategory::Schema
        );
        assert_eq!(DbError::DataFailed("x".into()).category(), ErrorCategory::Data);
        assert_eq!(
            DbError::ConstraintViolation {
                constraint_type: ConstraintType::Unique,
                constraint: "x".into(),
                table: "x".into(),
                column: None,
                message: "x".into(),
            }
            .category(),
            ErrorCategory::Data
        );
        assert_eq!(DbError::Validation("x".into()).category(), ErrorCategory::Validation);
        assert_eq!(DbError::ReadOnlyViolation("x".into()).category(), ErrorCategory::Safety);
        assert_eq!(DbError::Internal("x".into()).category(), ErrorCategory::Internal);
    }
}
