use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorEnvelope {
    pub code: String,
    pub message_id: String,
    pub message: String,
    pub details: Option<serde_json::Value>,
    pub request_id: Option<String>,
}

#[derive(Debug, thiserror::Error)]
pub enum DomainError {
    #[error("validation: {0}")]
    Validation(String),
    #[error("connection: {0}")]
    Connection(String),
    #[error("query: {0}")]
    Query(String),
}

impl DomainError {
    pub fn to_envelope(&self, request_id: Option<String>) -> ErrorEnvelope {
        match self {
            DomainError::Validation(msg) => ErrorEnvelope {
                code: "VALIDATION_ERROR".into(),
                message_id: "error.validation".into(),
                message: msg.clone(),
                details: None,
                request_id,
            },
            DomainError::Connection(msg) => ErrorEnvelope {
                code: "CONNECTION_ERROR".into(),
                message_id: "error.connection".into(),
                message: msg.clone(),
                details: None,
                request_id,
            },
            DomainError::Query(msg) => ErrorEnvelope {
                code: "QUERY_ERROR".into(),
                message_id: "error.query".into(),
                message: msg.clone(),
                details: None,
                request_id,
            },
        }
    }
}
