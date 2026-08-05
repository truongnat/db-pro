use async_trait::async_trait;

use crate::domain::connection::ConnectionId;
use crate::domain::error::DbError;
use crate::domain::history::SavedQuery;

#[async_trait]
pub trait SavedQueryRepository: Send + Sync {
    async fn save(&self, query: &SavedQuery) -> Result<(), DbError>;

    async fn list(&self, connection_id: &ConnectionId) -> Result<Vec<SavedQuery>, DbError>;

    async fn delete(&self, id: &uuid::Uuid) -> Result<(), DbError>;
}
