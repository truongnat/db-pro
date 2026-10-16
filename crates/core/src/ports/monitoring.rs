use async_trait::async_trait;

use crate::domain::connection::ConnectionHandle;
use crate::domain::error::DbError;
use crate::domain::monitoring::{
    LocalMonitorState, MaintenanceAction, MonitorLock, MonitorSession, RelationSizeStat, ServerSummary,
    StatStatementSort, StatStatementsSnapshot,
};

#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait MonitoringPort: Send + Sync {
    /// List sessions / backends for the active connection handle.
    async fn list_sessions(&self, handle: &ConnectionHandle) -> Result<Vec<MonitorSession>, DbError>;

    /// Optional local/file state (SQLite). Return `None` for server engines.
    async fn local_state(&self, handle: &ConnectionHandle) -> Result<Option<LocalMonitorState>, DbError>;

    /// Cancel the query running on the snapshot backend (PostgreSQL `pg_cancel_backend`).
    /// The adapter must re-verify `backend_id` and `backend_start` together so a
    /// recycled PID cannot cancel a different session (#501).
    async fn cancel_backend(
        &self,
        handle: &ConnectionHandle,
        backend_id: i64,
        backend_start: &str,
    ) -> Result<bool, DbError>;

    /// Terminate the backend session (PostgreSQL `pg_terminate_backend`).
    ///
    /// `backend_start` is the snapshot identity captured from
    /// `pg_stat_activity.backend_start`. The adapter must re-verify pid **and**
    /// `backend_start` against `pg_stat_activity` in the same statement that
    /// calls `pg_terminate_backend`, so a pid recycled by the OS between the
    /// snapshot and the confirmation cannot kill an unrelated session (#501).
    async fn terminate_backend(
        &self,
        handle: &ConnectionHandle,
        backend_id: i64,
        backend_start: &str,
    ) -> Result<bool, DbError>;

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

    /// `pg_stat_statements` workload (#251). Non-PG adapters return a missing-extension snapshot.
    async fn stat_statements(
        &self,
        handle: &ConnectionHandle,
        sort: StatStatementSort,
        limit: usize,
    ) -> Result<StatStatementsSnapshot, DbError>;

    /// Reset `pg_stat_statements` counters. Caller must enforce admin confirmation.
    async fn reset_stat_statements(&self, handle: &ConnectionHandle) -> Result<(), DbError>;
}
