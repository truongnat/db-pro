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

    /// Cancel the query running on `backend_id` (PostgreSQL `pg_cancel_backend`).
    /// `expected_backend_start` is the `pg_stat_activity.backend_start` captured
    /// in the monitoring snapshot; implementations MUST refuse the signal when
    /// the pid's current `backend_start` differs, so a recycled pid can never
    /// hit an innocent session (#501).
    async fn cancel_backend(
        &self,
        handle: &ConnectionHandle,
        backend_id: i64,
        expected_backend_start: &str,
    ) -> Result<bool, DbError>;

    /// Terminate the backend session (PostgreSQL `pg_terminate_backend`).
    /// Same recycled-pid contract as [`Self::cancel_backend`] (#501).
    async fn terminate_backend(
        &self,
        handle: &ConnectionHandle,
        backend_id: i64,
        expected_backend_start: &str,
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
