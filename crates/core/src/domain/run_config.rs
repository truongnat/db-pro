use serde::{Deserialize, Serialize};

use crate::domain::connection::ConnectionId;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunConfig {
    pub id: uuid::Uuid,
    pub connection_id: ConnectionId,
    pub name: String,
    pub sql: String,
    pub timeout_ms: u64,
    pub max_rows: u64,
    pub created_at: chrono::DateTime<chrono::Utc>,
}
