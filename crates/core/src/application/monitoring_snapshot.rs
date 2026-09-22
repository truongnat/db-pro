use crate::domain::connection::{ConnectionId, DriverType};
use crate::domain::error::DbError;
use crate::domain::monitoring::MonitoringSnapshot;

use super::{now_ms, MonitoringService};

pub(super) async fn execute(
    service: &MonitoringService,
    connection_id: &ConnectionId,
) -> Result<MonitoringSnapshot, DbError> {
    let driver = service.driver(connection_id).await?;
    let port = service.port_for(driver)?;
    let handle = service
        .registry
        .get(connection_id)
        .ok_or_else(|| DbError::ConnectionFailed(format!("connection {connection_id} is not active")))?;

    let sessions = match driver {
        DriverType::Postgres => port.list_sessions(&handle).await?,
        DriverType::SQLite => Vec::new(),
        DriverType::Mysql => unreachable!("port_for rejects mysql"),
        DriverType::SqlServer => unreachable!("port_for rejects sql server"),
    };
    let local = match driver {
        DriverType::SQLite => port.local_state(&handle).await?,
        DriverType::Postgres => None,
        DriverType::Mysql => None,
        DriverType::SqlServer => None,
    };
    let (locks, relation_sizes, server) = match driver {
        DriverType::Postgres => {
            // The monitoring panel must still render when a section query fails, so
            // each section falls back to empty values; the error is logged with the connection
            // instead of being silently swallowed (error-handling.md §3).
            let locks = match port.list_locks(&handle).await {
                Ok(locks) => locks,
                Err(error) => {
                    tracing::warn!(connection_id = %connection_id, %error, "monitoring: list_locks failed — panel renders without lock data");
                    Vec::new()
                }
            };
            let relation_sizes = match port.relation_sizes(&handle, 40).await {
                Ok(sizes) => sizes,
                Err(error) => {
                    tracing::warn!(connection_id = %connection_id, %error, "monitoring: relation_sizes failed — panel renders without size data");
                    Vec::new()
                }
            };
            let server = port.server_summary(&handle).await.unwrap_or(None);
            (locks, relation_sizes, server)
        }
        DriverType::SQLite => (Vec::new(), Vec::new(), None),
        DriverType::Mysql => unreachable!("port_for rejects mysql"),
        DriverType::SqlServer => unreachable!("port_for rejects sql server"),
    };

    let workload = match driver {
        DriverType::Postgres => Some(
            port.stat_statements(&handle, crate::domain::monitoring::StatStatementSort::TotalTime, 100)
                .await
                .unwrap_or_else(|err| crate::domain::monitoring::StatStatementsSnapshot {
                    extension_present: false,
                    extension_version: None,
                    message: format!("Could not load pg_stat_statements: {err}"),
                    statements: Vec::new(),
                    sort: crate::domain::monitoring::StatStatementSort::TotalTime,
                    fetched_at_ms: now_ms(),
                }),
        ),
        DriverType::SQLite | DriverType::Mysql => None,
        DriverType::SqlServer => None,
    };

    let message = match driver {
        DriverType::Postgres => format!(
            "{} session(s) · {} lock row(s) · {} relation(s) · workload {}",
            sessions.len(),
            locks.len(),
            relation_sizes.len(),
            workload
                .as_ref()
                .map(|w| {
                    if w.extension_present {
                        format!("{} statement(s)", w.statements.len())
                    } else {
                        "extension missing".into()
                    }
                })
                .unwrap_or_else(|| "n/a".into())
        ),
        DriverType::SQLite => "SQLite local file state (no server sessions)".into(),
        DriverType::Mysql => "MySQL session monitoring is not enabled yet".into(),
        DriverType::SqlServer => "SQL Server session monitoring is not enabled yet".into(),
    };

    Ok(MonitoringSnapshot {
        connection_id: connection_id.to_string(),
        driver: format!("{driver:?}"),
        sessions,
        locks,
        relation_sizes,
        server,
        local,
        workload,
        fetched_at_ms: now_ms(),
        message,
    })
}
