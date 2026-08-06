use serde::{Deserialize, Serialize};

use super::query::QueryError;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorEnvelope {
    pub code: String,
    pub message_id: String,
    pub message: String,
    pub details: Option<serde_json::Value>,
    pub request_id: Option<String>,
}

impl From<&QueryError> for ErrorEnvelope {
    fn from(err: &QueryError) -> Self {
        Self {
            code: err.code().into(),
            message_id: err.message_id().into(),
            message: err.to_string(),
            details: None,
            request_id: None,
        }
    }
}

impl ErrorEnvelope {
    pub fn with_request_id(mut self, request_id: Option<String>) -> Self {
        self.request_id = request_id;
        self
    }
}

#[derive(thiserror::Error, Debug)]
pub enum DbError {
    #[error("connection failed: {0}")]
    ConnectionFailed(String),

    #[error("query failed: {0}")]
    QueryFailed(String),

    #[error("not found: {0}")]
    NotFound(String),

    #[error("authentication failed: {0}")]
    AuthFailed(String),

    #[error("timeout: {0}")]
    Timeout(String),

    #[error("query cancelled: {0}")]
    Cancelled(String),

    #[error("io error: {0}")]
    Io(String),

    #[error("introspection failed: {0}")]
    IntrospectionFailed(String),

    #[error("encryption failed: {0}")]
    EncryptionFailed(String),

    #[error("validation: {0}")]
    Validation(String),

    #[error("internal: {0}")]
    Internal(String),
}

impl From<QueryError> for DbError {
    fn from(err: QueryError) -> Self {
        match err {
            QueryError::ConnectionNotFound { .. } | QueryError::ConnectionLost => {
                DbError::ConnectionFailed(err.to_string())
            }
            QueryError::Timeout { .. } => DbError::Timeout(err.to_string()),
            QueryError::Validation(_) => DbError::QueryFailed(err.to_string()),
            _ => DbError::QueryFailed(err.to_string()),
        }
    }
}
