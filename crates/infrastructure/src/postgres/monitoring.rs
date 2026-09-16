//! PostgreSQL monitoring via the shared [`DbConnector`] (never called from UI).

use async_trait::async_trait;
use std::sync::Arc;

use db_pro_core::domain::connection::ConnectionHandle;
use db_pro_core::domain::error::DbError;
use db_pro_core::domain::monitoring::{
    LocalMonitorState, MaintenanceAction, MonitorLock, MonitorSession, RelationSizeStat, ServerSummary, StatStatement,
    StatStatementSort, StatStatementsSnapshot,
};
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
    CASE
        WHEN xact_start IS NULL THEN NULL
        ELSE GREATEST(0, FLOOR(EXTRACT(EPOCH FROM (now() - xact_start)) * 1000))::bigint
    END AS xact_age_ms,
    CASE
        WHEN backend_start IS NULL THEN NULL
        ELSE GREATEST(0, FLOOR(EXTRACT(EPOCH FROM (now() - backend_start)) * 1000))::bigint
    END AS backend_age_ms,
    (state IS NOT NULL AND state ILIKE 'idle in transaction%') AS idle_in_transaction,
    (pid = pg_backend_pid()) AS is_current
FROM pg_catalog.pg_stat_activity
WHERE backend_type = 'client backend'
ORDER BY
    CASE WHEN state ILIKE 'idle in transaction%' THEN 0 ELSE 1 END,
    xact_start NULLS LAST,
    query_start NULLS LAST,
    pid
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
                xact_age_ms: cells.get(13).and_then(cell_i64).map(|v| v.max(0) as u64),
                backend_age_ms: cells.get(14).and_then(cell_i64).map(|v| v.max(0) as u64),
                idle_in_transaction: cells.get(15).map(cell_bool).unwrap_or(false),
                is_current: cells.get(16).map(cell_bool).unwrap_or(false),
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

    async fn list_locks(&self, handle: &ConnectionHandle) -> Result<Vec<MonitorLock>, DbError> {
        let result = self
            .connector
            .query(
                handle,
                r#"
                SELECT
                    a.pid AS locked_pid,
                    blocked.pid AS blocker_pid,
                    COALESCE(c.relname, l.locktype::text) AS relation,
                    l.locktype::text,
                    l.mode::text,
                    l.granted,
                    CASE
                        WHEN a.state = 'active' AND a.wait_event_type = 'Lock' AND a.query_start IS NOT NULL
                        THEN GREATEST(0, FLOOR(EXTRACT(EPOCH FROM (now() - a.query_start)) * 1000))::bigint
                        ELSE NULL
                    END AS waiting_ms
                FROM pg_catalog.pg_locks l
                JOIN pg_catalog.pg_stat_activity a ON a.pid = l.pid
                LEFT JOIN pg_catalog.pg_class c ON c.oid = l.relation
                LEFT JOIN LATERAL (
                    SELECT DISTINCT blocking.pid
                    FROM pg_catalog.pg_locks waiting
                    JOIN pg_catalog.pg_locks blocking
                      ON waiting.locktype = blocking.locktype
                     AND waiting.database IS NOT DISTINCT FROM blocking.database
                     AND waiting.relation IS NOT DISTINCT FROM blocking.relation
                     AND waiting.page IS NOT DISTINCT FROM blocking.page
                     AND waiting.tuple IS NOT DISTINCT FROM blocking.tuple
                     AND waiting.virtualxid IS NOT DISTINCT FROM blocking.virtualxid
                     AND waiting.transactionid IS NOT DISTINCT FROM blocking.transactionid
                     AND waiting.classid IS NOT DISTINCT FROM blocking.classid
                     AND waiting.objid IS NOT DISTINCT FROM blocking.objid
                     AND waiting.objsubid IS NOT DISTINCT FROM blocking.objsubid
                     AND waiting.pid <> blocking.pid
                     AND waiting.granted = false
                     AND blocking.granted = true
                    WHERE waiting.pid = a.pid
                    LIMIT 1
                ) blocked ON true
                WHERE a.backend_type = 'client backend'
                ORDER BY l.granted, a.pid
                LIMIT 200
                "#,
                &[],
            )
            .await?;
        let mut locks = Vec::with_capacity(result.rows.len());
        for row in result.rows {
            let cells = &row.0;
            let locked_pid = cells
                .first()
                .and_then(cell_i64)
                .ok_or_else(|| DbError::Internal("lock row missing pid".into()))?;
            locks.push(MonitorLock {
                locked_pid,
                blocker_pid: cells.get(1).and_then(cell_i64),
                relation: cells.get(2).and_then(cell_text),
                lock_type: cells.get(3).and_then(cell_text),
                mode: cells.get(4).and_then(cell_text),
                granted: cells.get(5).map(cell_bool).unwrap_or(false),
                waiting_ms: cells.get(6).and_then(cell_i64).map(|v| v as u64),
            });
        }
        Ok(locks)
    }

    async fn relation_sizes(&self, handle: &ConnectionHandle, limit: usize) -> Result<Vec<RelationSizeStat>, DbError> {
        let limit = limit.clamp(1, 200) as i64;
        let result = self
            .connector
            .query(
                handle,
                r#"
                SELECT
                    n.nspname AS schema,
                    c.relname AS name,
                    CASE c.relkind
                        WHEN 'r' THEN 'table'
                        WHEN 'i' THEN 'index'
                        WHEN 'm' THEN 'matview'
                        ELSE c.relkind::text
                    END AS kind,
                    pg_total_relation_size(c.oid)::bigint AS total_bytes,
                    pg_relation_size(c.oid)::bigint AS table_bytes,
                    GREATEST(0, pg_total_relation_size(c.oid) - pg_relation_size(c.oid))::bigint AS index_bytes,
                    s.seq_scan,
                    s.idx_scan,
                    s.n_live_tup,
                    s.n_dead_tup
                FROM pg_catalog.pg_class c
                JOIN pg_catalog.pg_namespace n ON n.oid = c.relnamespace
                LEFT JOIN pg_catalog.pg_stat_all_tables s
                  ON s.relid = c.oid
                WHERE c.relkind IN ('r', 'm', 'i')
                  AND n.nspname NOT IN ('pg_catalog', 'information_schema', 'pg_toast')
                ORDER BY pg_total_relation_size(c.oid) DESC
                LIMIT $1
                "#,
                &[QueryParam::Int64(limit)],
            )
            .await?;
        let mut out = Vec::with_capacity(result.rows.len());
        for row in result.rows {
            let cells = &row.0;
            out.push(RelationSizeStat {
                schema: cells.first().and_then(cell_text).unwrap_or_default(),
                name: cells.get(1).and_then(cell_text).unwrap_or_default(),
                kind: cells.get(2).and_then(cell_text).unwrap_or_else(|| "table".into()),
                total_bytes: cells.get(3).and_then(cell_i64).unwrap_or(0).max(0) as u64,
                table_bytes: cells.get(4).and_then(cell_i64).unwrap_or(0).max(0) as u64,
                index_bytes: cells.get(5).and_then(cell_i64).unwrap_or(0).max(0) as u64,
                seq_scan: cells.get(6).and_then(cell_i64).map(|v| v.max(0) as u64),
                idx_scan: cells.get(7).and_then(cell_i64).map(|v| v.max(0) as u64),
                n_live_tup: cells.get(8).and_then(cell_i64).map(|v| v.max(0) as u64),
                n_dead_tup: cells.get(9).and_then(cell_i64).map(|v| v.max(0) as u64),
            });
        }
        Ok(out)
    }

    async fn server_summary(&self, handle: &ConnectionHandle) -> Result<Option<ServerSummary>, DbError> {
        let result = self
            .connector
            .query(
                handle,
                r#"
                SELECT
                    version(),
                    current_database(),
                    (SELECT setting::bigint FROM pg_catalog.pg_settings WHERE name = 'max_connections'),
                    (SELECT count(*)::bigint FROM pg_catalog.pg_stat_activity WHERE backend_type = 'client backend'),
                    pg_database_size(current_database())::bigint
                "#,
                &[],
            )
            .await?;
        let Some(row) = result.rows.first() else {
            return Ok(None);
        };
        let cells = &row.0;
        Ok(Some(ServerSummary {
            version: cells.first().and_then(cell_text),
            current_database: cells.get(1).and_then(cell_text),
            max_connections: cells.get(2).and_then(cell_i64).map(|v| v.max(0) as u64),
            current_connections: cells.get(3).and_then(cell_i64).map(|v| v.max(0) as u64),
            database_size_bytes: cells.get(4).and_then(cell_i64).map(|v| v.max(0) as u64),
        }))
    }

    async fn run_maintenance(
        &self,
        handle: &ConnectionHandle,
        schema: Option<String>,
        table: Option<String>,
        action: MaintenanceAction,
    ) -> Result<(), DbError> {
        let sql = match (schema.as_deref(), table.as_deref()) {
            (Some(schema), Some(table)) => {
                let schema = quote_ident(schema)?;
                let table = quote_ident(table)?;
                match action {
                    MaintenanceAction::Vacuum => format!("VACUUM {schema}.{table}"),
                    MaintenanceAction::Analyze => format!("ANALYZE {schema}.{table}"),
                    MaintenanceAction::VacuumAnalyze => format!("VACUUM ANALYZE {schema}.{table}"),
                }
            }
            _ => match action {
                MaintenanceAction::Vacuum => "VACUUM".into(),
                MaintenanceAction::Analyze => "ANALYZE".into(),
                MaintenanceAction::VacuumAnalyze => "VACUUM ANALYZE".into(),
            },
        };
        self.connector.execute(handle, &sql, &[]).await?;
        Ok(())
    }

    async fn stat_statements(
        &self,
        handle: &ConnectionHandle,
        sort: StatStatementSort,
        limit: usize,
    ) -> Result<StatStatementsSnapshot, DbError> {
        let fetched_at_ms = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_millis() as u64)
            .unwrap_or(0);

        let ext = self
            .connector
            .query(
                handle,
                "SELECT extversion FROM pg_catalog.pg_extension WHERE extname = 'pg_stat_statements'",
                &[],
            )
            .await?;
        let extension_version = ext.rows.first().and_then(|row| row.0.first()).and_then(cell_text);
        if extension_version.is_none() {
            return Ok(StatStatementsSnapshot {
                extension_present: false,
                extension_version: None,
                message: "Extension pg_stat_statements is not installed. Ask a DBA to CREATE EXTENSION \
                    pg_stat_statements (never auto-installed by DB Pro)."
                    .into(),
                statements: Vec::new(),
                sort,
                fetched_at_ms,
            });
        }

        let order = match sort {
            StatStatementSort::TotalTime => "s.total_exec_time DESC NULLS LAST",
            StatStatementSort::MeanTime => "s.mean_exec_time DESC NULLS LAST",
            StatStatementSort::Calls => "s.calls DESC NULLS LAST",
            StatStatementSort::Rows => "s.rows DESC NULLS LAST",
        };
        let limit = limit.clamp(1, 500);
        let sql = format!(
            r#"
SELECT
    s.queryid::bigint,
    s.userid::bigint,
    s.dbid::bigint,
    d.datname::text,
    r.rolname::text,
    LEFT(s.query, 4000),
    s.calls::bigint,
    s.total_exec_time,
    s.mean_exec_time,
    s.min_exec_time,
    s.max_exec_time,
    s.rows::bigint,
    s.shared_blks_hit::bigint,
    s.shared_blks_read::bigint,
    s.shared_blks_dirtied::bigint,
    s.shared_blks_written::bigint,
    s.local_blks_hit::bigint,
    s.local_blks_read::bigint,
    s.temp_blks_read::bigint,
    s.temp_blks_written::bigint
FROM pg_stat_statements s
LEFT JOIN pg_catalog.pg_database d ON d.oid = s.dbid
LEFT JOIN pg_catalog.pg_roles r ON r.oid = s.userid
ORDER BY {order}
LIMIT {limit}
"#
        );

        let result = match self.connector.query(handle, &sql, &[]).await {
            Ok(r) => r,
            Err(err) => {
                // PG < 13 used total_time / mean_time column names.
                let legacy = format!(
                    r#"
SELECT
    s.queryid::bigint,
    s.userid::bigint,
    s.dbid::bigint,
    d.datname::text,
    r.rolname::text,
    LEFT(s.query, 4000),
    s.calls::bigint,
    s.total_time,
    s.mean_time,
    s.min_time,
    s.max_time,
    s.rows::bigint,
    s.shared_blks_hit::bigint,
    s.shared_blks_read::bigint,
    s.shared_blks_dirtied::bigint,
    s.shared_blks_written::bigint,
    s.local_blks_hit::bigint,
    s.local_blks_read::bigint,
    s.temp_blks_read::bigint,
    s.temp_blks_written::bigint
FROM pg_stat_statements s
LEFT JOIN pg_catalog.pg_database d ON d.oid = s.dbid
LEFT JOIN pg_catalog.pg_roles r ON r.oid = s.userid
ORDER BY {}
LIMIT {limit}
"#,
                    match sort {
                        StatStatementSort::TotalTime => "s.total_time DESC NULLS LAST",
                        StatStatementSort::MeanTime => "s.mean_time DESC NULLS LAST",
                        StatStatementSort::Calls => "s.calls DESC NULLS LAST",
                        StatStatementSort::Rows => "s.rows DESC NULLS LAST",
                    }
                );
                match self.connector.query(handle, &legacy, &[]).await {
                    Ok(r) => r,
                    Err(_) => {
                        return Ok(StatStatementsSnapshot {
                            extension_present: true,
                            extension_version,
                            message: format!("pg_stat_statements is installed but could not be queried: {err}"),
                            statements: Vec::new(),
                            sort,
                            fetched_at_ms,
                        });
                    }
                }
            }
        };

        let mut statements = Vec::with_capacity(result.rows.len());
        for row in result.rows {
            let c = &row.0;
            let query = c.get(5).and_then(cell_text).unwrap_or_default();
            if query.is_empty() {
                continue;
            }
            statements.push(StatStatement {
                queryid: c.first().and_then(cell_i64),
                userid: c.get(1).and_then(cell_i64),
                dbid: c.get(2).and_then(cell_i64),
                database: c.get(3).and_then(cell_text),
                username: c.get(4).and_then(cell_text),
                query,
                calls: c.get(6).and_then(cell_i64).map(|v| v.max(0) as u64).unwrap_or(0),
                total_time_ms: c.get(7).map(cell_f64).unwrap_or(0.0),
                mean_time_ms: c.get(8).map(cell_f64).unwrap_or(0.0),
                min_time_ms: c.get(9).map(cell_f64).unwrap_or(0.0),
                max_time_ms: c.get(10).map(cell_f64).unwrap_or(0.0),
                rows: c.get(11).and_then(cell_i64).map(|v| v.max(0) as u64).unwrap_or(0),
                shared_blks_hit: c.get(12).and_then(cell_i64).map(|v| v.max(0) as u64).unwrap_or(0),
                shared_blks_read: c.get(13).and_then(cell_i64).map(|v| v.max(0) as u64).unwrap_or(0),
                shared_blks_dirtied: c.get(14).and_then(cell_i64).map(|v| v.max(0) as u64).unwrap_or(0),
                shared_blks_written: c.get(15).and_then(cell_i64).map(|v| v.max(0) as u64).unwrap_or(0),
                local_blks_hit: c.get(16).and_then(cell_i64).map(|v| v.max(0) as u64).unwrap_or(0),
                local_blks_read: c.get(17).and_then(cell_i64).map(|v| v.max(0) as u64).unwrap_or(0),
                temp_blks_read: c.get(18).and_then(cell_i64).map(|v| v.max(0) as u64).unwrap_or(0),
                temp_blks_written: c.get(19).and_then(cell_i64).map(|v| v.max(0) as u64).unwrap_or(0),
            });
        }

        Ok(StatStatementsSnapshot {
            extension_present: true,
            extension_version,
            message: format!("{} statement(s) by {}", statements.len(), sort.as_label()),
            statements,
            sort,
            fetched_at_ms,
        })
    }

    async fn reset_stat_statements(&self, handle: &ConnectionHandle) -> Result<(), DbError> {
        self.connector
            .execute(handle, "SELECT pg_stat_statements_reset()", &[])
            .await?;
        Ok(())
    }
}

fn cell_f64(cell: &CellValue) -> f64 {
    match cell {
        CellValue::Float64(v) => *v,
        CellValue::Int64(v) => *v as f64,
        CellValue::Text(s) => s.parse().unwrap_or(0.0),
        _ => 0.0,
    }
}

fn quote_ident(value: &str) -> Result<String, DbError> {
    if value.trim().is_empty() || value.contains('\0') {
        return Err(DbError::Validation("invalid identifier for maintenance".into()));
    }
    Ok(format!("\"{}\"", value.replace('"', "\"\"")))
}
