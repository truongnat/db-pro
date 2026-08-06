use std::sync::Arc;

use async_trait::async_trait;

use crate::domain::connection::{ConnectionConfig, ConnectionHandle};
use crate::domain::error::DbError;
use crate::domain::query::{QueryParam, QueryResult};
use crate::domain::schema::IntrospectResult;

#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait DbConnector: Send + Sync {
    async fn connect(&self, config: &ConnectionConfig, password: &str) -> Result<ConnectionHandle, DbError>;

    async fn disconnect(&self, handle: &ConnectionHandle) -> Result<(), DbError>;

    async fn test_connection(&self, config: &ConnectionConfig, password: &str) -> Result<(), DbError>;

    async fn query(&self, handle: &ConnectionHandle, sql: &str, params: &[QueryParam]) -> Result<QueryResult, DbError>;

    async fn introspect(&self, handle: &ConnectionHandle) -> Result<IntrospectResult, DbError>;

    async fn explain(&self, handle: &ConnectionHandle, sql: &str) -> Result<serde_json::Value, DbError>;
}

#[async_trait]
impl<T: DbConnector + ?Sized> DbConnector for Arc<T> {
    async fn connect(&self, config: &ConnectionConfig, password: &str) -> Result<ConnectionHandle, DbError> {
        self.as_ref().connect(config, password).await
    }

    async fn disconnect(&self, handle: &ConnectionHandle) -> Result<(), DbError> {
        self.as_ref().disconnect(handle).await
    }

    async fn test_connection(&self, config: &ConnectionConfig, password: &str) -> Result<(), DbError> {
        self.as_ref().test_connection(config, password).await
    }

    async fn query(&self, handle: &ConnectionHandle, sql: &str, params: &[QueryParam]) -> Result<QueryResult, DbError> {
        self.as_ref().query(handle, sql, params).await
    }

    async fn introspect(&self, handle: &ConnectionHandle) -> Result<IntrospectResult, DbError> {
        self.as_ref().introspect(handle).await
    }

    async fn explain(&self, handle: &ConnectionHandle, sql: &str) -> Result<serde_json::Value, DbError> {
        self.as_ref().explain(handle, sql).await
    }
}
