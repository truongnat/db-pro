use async_trait::async_trait;

use crate::domain::connection::ConnectionId;
use crate::domain::error::DbError;
use crate::domain::history::QueryHistory;
use crate::domain::query::QueryResult;

#[async_trait]
pub trait QueryHistoryRepository: Send + Sync {
    async fn save(&self, connection_id: &ConnectionId, sql: &str, result: &QueryResult) -> Result<(), DbError>;

    async fn list(&self, connection_id: &ConnectionId, limit: u32) -> Result<Vec<QueryHistory>, DbError>;
}
