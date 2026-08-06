use async_trait::async_trait;

use crate::domain::connection::ConnectionId;
use crate::domain::error::DbError;
use crate::domain::run_config::RunConfig;

#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait RunConfigRepository: Send + Sync {
    async fn save(&self, config: &RunConfig) -> Result<(), DbError>;
    async fn list(&self, connection_id: &ConnectionId) -> Result<Vec<RunConfig>, DbError>;
    async fn delete(&self, id: &uuid::Uuid) -> Result<(), DbError>;
}
