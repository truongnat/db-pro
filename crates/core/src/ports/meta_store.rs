use async_trait::async_trait;

use crate::domain::connection::{Connection, ConnectionConfig, ConnectionId};
use crate::domain::error::DbError;
use crate::domain::history::{QueryHistory, SavedQuery, Settings, Workspace};
use crate::domain::query::QueryResult;
use crate::domain::schema::IntrospectResult;

#[async_trait]
pub trait MetaStore: Send + Sync {
    async fn save_connection(&self, connection: &Connection) -> Result<(), DbError>;

    async fn get_connection(&self, id: &ConnectionId) -> Result<Option<Connection>, DbError>;

    async fn get_connection_config(&self, id: &ConnectionId) -> Result<Option<ConnectionConfig>, DbError>;

    async fn list_connections(&self) -> Result<Vec<Connection>, DbError>;

    async fn delete_connection(&self, id: &ConnectionId) -> Result<(), DbError>;

    async fn save_query_history(
        &self,
        connection_id: &ConnectionId,
        sql: &str,
        result: &QueryResult,
    ) -> Result<(), DbError>;

    async fn list_query_history(&self, connection_id: &ConnectionId, limit: u32) -> Result<Vec<QueryHistory>, DbError>;

    async fn save_saved_query(&self, query: &SavedQuery) -> Result<(), DbError>;

    async fn list_saved_queries(&self, connection_id: &ConnectionId) -> Result<Vec<SavedQuery>, DbError>;

    async fn save_workspace(&self, workspace: &Workspace) -> Result<(), DbError>;

    async fn list_workspaces(&self) -> Result<Vec<Workspace>, DbError>;

    async fn save_settings(&self, settings: &Settings) -> Result<(), DbError>;

    async fn load_settings(&self) -> Result<Settings, DbError>;

    async fn save_introspection_cache(
        &self,
        connection_id: &ConnectionId,
        result: &IntrospectResult,
    ) -> Result<(), DbError>;

    async fn get_introspection_cache(&self, connection_id: &ConnectionId) -> Result<Option<IntrospectResult>, DbError>;
}
