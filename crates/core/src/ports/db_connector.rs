use async_trait::async_trait;

use crate::domain::connection::{ConnectionConfig, ConnectionHandle};
use crate::domain::error::DbError;
use crate::domain::query::{QueryParam, QueryResult};
use crate::domain::schema::IntrospectResult;

#[async_trait]
pub trait DbConnector: Send + Sync {
    async fn connect(&self, config: &ConnectionConfig, password: &str) -> Result<ConnectionHandle, DbError>;

    async fn disconnect(&self, handle: &ConnectionHandle) -> Result<(), DbError>;

    async fn test_connection(&self, config: &ConnectionConfig, password: &str) -> Result<(), DbError>;

    async fn query(&self, handle: &ConnectionHandle, sql: &str, params: &[QueryParam]) -> Result<QueryResult, DbError>;

    async fn introspect(&self, handle: &ConnectionHandle) -> Result<IntrospectResult, DbError>;

    async fn explain(&self, handle: &ConnectionHandle, sql: &str) -> Result<serde_json::Value, DbError>;
}
