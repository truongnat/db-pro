use async_trait::async_trait;

use crate::domain::connection::ConnectionHandle;
use crate::domain::error::DbError;
use crate::domain::monitoring::{
    LocalMonitorState, MaintenanceAction, MonitorLock, MonitorSession, RelationSizeStat, ServerSummary,
};

#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait MonitoringPort: Send + Sync {
    /// List sessions / backends for the active connection handle.
    async fn list_sessions(&self, handle: &ConnectionHandle) -> Result<Vec<MonitorSession>, DbError>;

    /// Optional local/file state (SQLite). Return `None` for server engines.
    async fn local_state(&self, handle: &ConnectionHandle) -> Result<Option<LocalMonitorState>, DbError>;

    /// Cancel the query running on `backend_id` (PostgreSQL `pg_cancel_backend`).
    async fn cancel_backend(&self, handle: &ConnectionHandle, backend_id: i64) -> Result<bool, DbError>;

    /// Terminate the backend session (PostgreSQL `pg_terminate_backend`).
    async fn terminate_backend(&self, handle: &ConnectionHandle, backend_id: i64) -> Result<bool, DbError>;

    /// Lock + blocker relationships (#197).
    async fn list_locks(&self, handle: &ConnectionHandle) -> Result<Vec<MonitorLock>, DbError>;

    /// Top relation sizes / scan stats (#197).
    async fn relation_sizes(&self, handle: &ConnectionHandle, limit: usize) -> Result<Vec<RelationSizeStat>, DbError>;

    /// Server / database summary (#197).
    async fn server_summary(&self, handle: &ConnectionHandle) -> Result<Option<ServerSummary>, DbError>;

    /// Run a maintenance action. SQL must be constructed only inside the adapter.
    async fn run_maintenance(
        &self,
        handle: &ConnectionHandle,
        schema: Option<String>,
        table: Option<String>,
        action: MaintenanceAction,
    ) -> Result<(), DbError>;
}
