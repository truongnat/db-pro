use async_trait::async_trait;

use crate::domain::connection::{Connection, ConnectionConfig, ConnectionId};
use crate::domain::error::DbError;

#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait ConnectionRepository: Send + Sync {
    async fn save(&self, connection: &Connection) -> Result<(), DbError>;

    async fn get(&self, id: &ConnectionId) -> Result<Option<Connection>, DbError>;

    async fn get_config(&self, id: &ConnectionId) -> Result<Option<ConnectionConfig>, DbError>;

    async fn list(&self) -> Result<Vec<Connection>, DbError>;

    async fn delete(&self, id: &ConnectionId) -> Result<(), DbError>;
}
