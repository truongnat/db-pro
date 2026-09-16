//! PostgreSQL monitoring via the shared [`DbConnector`] (never called from UI).

use async_trait::async_trait;
use std::sync::Arc;

use db_pro_core::domain::connection::ConnectionHandle;
use db_pro_core::domain::error::DbError;
use db_pro_core::domain::monitoring::{LocalMonitorState, MonitorSession};
use db_pro_core::domain::query::{CellValue, QueryParam};
use db_pro_core::ports::{DbConnector, MonitoringPort};

const LIST_SESSIONS_SQL: &str = r#"
SELECT
    pid,
    datname,
    usename,
    NULLIF(application_name, '') AS application_name,
    host(client_addr)::text AS client_addr,
    state,
    wait_event_type,
    wait_event,
    LEFT(query, 4000) AS query,
    CASE
        WHEN query_start IS NULL THEN NULL
        ELSE GREATEST(0, FLOOR(EXTRACT(EPOCH FROM (now() - query_start)) * 1000))::bigint
    END AS query_duration_ms,
    backend_start::text,
    xact_start::text,
    query_start::text,
    (pid = pg_backend_pid()) AS is_current
FROM pg_catalog.pg_stat_activity
WHERE backend_type = 'client backend'
ORDER BY query_start NULLS LAST, pid
"#;

pub struct PostgresMonitoringPort {
    connector: Arc<dyn DbConnector>,
}

impl PostgresMonitoringPort {
    pub fn new(connector: Arc<dyn DbConnector>) -> Self {
        Self { connector }
    }
}

fn cell_text(cell: &CellValue) -> Option<String> {
    match cell {
        CellValue::Null => None,
        CellValue::Text(s) => Some(s.clone()),
        CellValue::Int64(v) => Some(v.to_string()),
        CellValue::Bool(v) => Some(v.to_string()),
        CellValue::Float64(v) => Some(v.to_string()),
        other => Some(format!("{other:?}")),
    }
}

fn cell_i64(cell: &CellValue) -> Option<i64> {
    match cell {
        CellValue::Int64(v) => Some(*v),
        CellValue::Text(s) => s.parse().ok(),
        _ => None,
    }
}

fn cell_bool(cell: &CellValue) -> bool {
    matches!(cell, CellValue::Bool(true)) || matches!(cell, CellValue::Text(s) if s == "t" || s == "true")
}

#[async_trait]
impl MonitoringPort for PostgresMonitoringPort {
    async fn list_sessions(&self, handle: &ConnectionHandle) -> Result<Vec<MonitorSession>, DbError> {
        let result = self.connector.query(handle, LIST_SESSIONS_SQL, &[]).await?;
        let mut sessions = Vec::with_capacity(result.rows.len());
        for row in result.rows {
            let cells = &row.0;
            let backend_id = cells
                .first()
                .and_then(cell_i64)
                .ok_or_else(|| DbError::Internal("pg_stat_activity row missing pid".into()))?;
            sessions.push(MonitorSession {
                backend_id,
                database: cells.get(1).and_then(cell_text),
                username: cells.get(2).and_then(cell_text),
                application_name: cells.get(3).and_then(cell_text),
                client_addr: cells.get(4).and_then(cell_text),
                state: cells.get(5).and_then(cell_text),
                wait_event_type: cells.get(6).and_then(cell_text),
                wait_event: cells.get(7).and_then(cell_text),
                query_text: cells.get(8).and_then(cell_text),
                query_duration_ms: cells.get(9).and_then(cell_i64).map(|v| v as u64),
                backend_start: cells.get(10).and_then(cell_text),
                xact_start: cells.get(11).and_then(cell_text),
                query_start: cells.get(12).and_then(cell_text),
                is_current: cells.get(13).map(cell_bool).unwrap_or(false),
            });
        }
        Ok(sessions)
    }

    async fn local_state(&self, _handle: &ConnectionHandle) -> Result<Option<LocalMonitorState>, DbError> {
        Ok(None)
    }

    async fn cancel_backend(&self, handle: &ConnectionHandle, backend_id: i64) -> Result<bool, DbError> {
        let result = self
            .connector
            .query(handle, "SELECT pg_cancel_backend($1)", &[QueryParam::Int64(backend_id)])
            .await?;
        Ok(result
            .rows
            .first()
            .and_then(|row| row.0.first())
            .map(cell_bool)
            .unwrap_or(false))
    }

    async fn terminate_backend(&self, handle: &ConnectionHandle, backend_id: i64) -> Result<bool, DbError> {
        let result = self
            .connector
            .query(
                handle,
                "SELECT pg_terminate_backend($1)",
                &[QueryParam::Int64(backend_id)],
            )
            .await?;
        Ok(result
            .rows
            .first()
            .and_then(|row| row.0.first())
            .map(cell_bool)
            .unwrap_or(false))
    }
}
