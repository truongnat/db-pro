//! SQLite local monitoring — pragma/file state only (no fake sessions).

use async_trait::async_trait;
use std::sync::Arc;

use db_pro_core::domain::connection::ConnectionHandle;
use db_pro_core::domain::error::DbError;
use db_pro_core::domain::monitoring::{
    LocalMonitorState, MaintenanceAction, MonitorLock, MonitorSession, RelationSizeStat, ServerSummary,
};
use db_pro_core::domain::query::CellValue;
use db_pro_core::ports::{DbConnector, MonitoringPort};

pub struct SqliteMonitoringPort {
    connector: Arc<dyn DbConnector>,
}

impl SqliteMonitoringPort {
    pub fn new(connector: Arc<dyn DbConnector>) -> Self {
        Self { connector }
    }

    async fn pragma_cell(&self, handle: &ConnectionHandle, sql: &str) -> Result<Option<CellValue>, DbError> {
        let result = self.connector.query(handle, sql, &[]).await?;
        Ok(result.rows.first().and_then(|row| row.0.first()).cloned())
    }

    async fn pragma_text(&self, handle: &ConnectionHandle, sql: &str) -> Result<Option<String>, DbError> {
        Ok(match self.pragma_cell(handle, sql).await? {
            Some(CellValue::Text(s)) => Some(s),
            Some(CellValue::Int64(v)) => Some(v.to_string()),
            Some(CellValue::Float64(v)) => Some(v.to_string()),
            _ => None,
        })
    }

    async fn pragma_u64(&self, handle: &ConnectionHandle, sql: &str) -> Result<Option<u64>, DbError> {
        Ok(match self.pragma_cell(handle, sql).await? {
            Some(CellValue::Int64(v)) if v >= 0 => Some(v as u64),
            Some(CellValue::Text(s)) => s.parse().ok(),
            _ => None,
        })
    }
}

#[async_trait]
impl MonitoringPort for SqliteMonitoringPort {
    async fn list_sessions(&self, _handle: &ConnectionHandle) -> Result<Vec<MonitorSession>, DbError> {
        Ok(Vec::new())
    }

    async fn local_state(&self, handle: &ConnectionHandle) -> Result<Option<LocalMonitorState>, DbError> {
        let journal_mode = self.pragma_text(handle, "PRAGMA journal_mode").await?;
        let page_count = self.pragma_u64(handle, "PRAGMA page_count").await?;
        let page_size = self.pragma_u64(handle, "PRAGMA page_size").await?;
        let freelist_count = self.pragma_u64(handle, "PRAGMA freelist_count").await?;
        let file_size_bytes = match (page_count, page_size) {
            (Some(pages), Some(size)) => Some(pages.saturating_mul(size)),
            _ => None,
        };

        Ok(Some(LocalMonitorState {
            journal_mode,
            page_count,
            page_size,
            freelist_count,
            file_size_bytes,
            note: "SQLite exposes local file/pragma state only; there are no server sessions.".into(),
        }))
    }

    async fn cancel_backend(&self, _handle: &ConnectionHandle, _backend_id: i64) -> Result<bool, DbError> {
        Err(DbError::Unsupported(
            "SQLite has no backend sessions to cancel; use query interrupt on the active connection".into(),
        ))
    }

    async fn terminate_backend(&self, _handle: &ConnectionHandle, _backend_id: i64) -> Result<bool, DbError> {
        Err(DbError::Unsupported(
            "SQLite has no backend sessions to terminate".into(),
        ))
    }

    async fn list_locks(&self, _handle: &ConnectionHandle) -> Result<Vec<MonitorLock>, DbError> {
        Ok(Vec::new())
    }

    async fn relation_sizes(
        &self,
        _handle: &ConnectionHandle,
        _limit: usize,
    ) -> Result<Vec<RelationSizeStat>, DbError> {
        Ok(Vec::new())
    }

    async fn server_summary(&self, _handle: &ConnectionHandle) -> Result<Option<ServerSummary>, DbError> {
        Ok(None)
    }

    async fn run_maintenance(
        &self,
        handle: &ConnectionHandle,
        _schema: Option<String>,
        table: Option<String>,
        action: MaintenanceAction,
    ) -> Result<(), DbError> {
        let sql = match (action, table.as_deref()) {
            (MaintenanceAction::Vacuum | MaintenanceAction::VacuumAnalyze, Some(table)) => {
                let table = quote_ident(table)?;
                format!("VACUUM {table}")
            }
            (MaintenanceAction::Vacuum | MaintenanceAction::VacuumAnalyze, None) => "VACUUM".into(),
            (MaintenanceAction::Analyze, Some(table)) => {
                let table = quote_ident(table)?;
                format!("ANALYZE {table}")
            }
            (MaintenanceAction::Analyze, None) => "ANALYZE".into(),
        };
        self.connector.execute(handle, &sql, &[]).await?;
        Ok(())
    }
}

fn quote_ident(value: &str) -> Result<String, DbError> {
    if value.trim().is_empty() || value.contains('\0') {
        return Err(DbError::Validation("invalid identifier for maintenance".into()));
    }
    Ok(format!("\"{}\"", value.replace('"', "\"\"")))
}
