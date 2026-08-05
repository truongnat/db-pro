use async_trait::async_trait;

use crate::domain::connection::ConnectionId;
use crate::domain::error::DbError;
use crate::domain::schema::IntrospectResult;

#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait IntrospectionCache: Send + Sync {
    async fn save(&self, connection_id: &ConnectionId, result: &IntrospectResult) -> Result<(), DbError>;

    async fn get(&self, connection_id: &ConnectionId) -> Result<Option<IntrospectResult>, DbError>;

    async fn invalidate(&self, connection_id: &ConnectionId) -> Result<(), DbError>;
}
