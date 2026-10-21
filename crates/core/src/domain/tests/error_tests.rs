//! Tests for `../error.rs` (moved out of the source file).
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
            details: Box::new(ConstraintViolationDetails {
                constraint_type: ConstraintType::Unique,
                constraint: "test".into(),
                table: "test".into(),
                column: None,
                message: "test".into(),
            }),
        },
        DbError::NotFound("test".into()),
        DbError::Conflict("test".into()),
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
            details: Box::new(ConstraintViolationDetails {
                constraint_type: ConstraintType::Unique,
                constraint: "x".into(),
                table: "x".into(),
                column: None,
                message: "x".into(),
            }),
        }
        .category(),
        ErrorCategory::Data
    );
    assert_eq!(DbError::Validation("x".into()).category(), ErrorCategory::Validation);
    assert_eq!(DbError::ReadOnlyViolation("x".into()).category(), ErrorCategory::Safety);
    assert_eq!(DbError::Internal("x".into()).category(), ErrorCategory::Internal);
}
