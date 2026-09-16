//! Provider-aware monitoring service (#196). UI never queries pg_stat_activity.

use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::domain::connection::{ConnectionId, DriverType};
use crate::domain::error::DbError;
use crate::domain::monitoring::MonitoringSnapshot;
use crate::ports::{ConnectionRepository, MonitoringPort};

use super::registry::ConnectionRegistry;

pub struct MonitoringService {
    pg: Box<dyn MonitoringPort>,
    sqlite: Box<dyn MonitoringPort>,
    registry: Arc<ConnectionRegistry>,
    connections: Box<dyn ConnectionRepository>,
}

impl MonitoringService {
    pub fn new(
        pg: Box<dyn MonitoringPort>,
        sqlite: Box<dyn MonitoringPort>,
        registry: Arc<ConnectionRegistry>,
        connections: Box<dyn ConnectionRepository>,
    ) -> Self {
        Self {
            pg,
            sqlite,
            registry,
            connections,
        }
    }

    fn port_for(&self, driver: DriverType) -> Result<&dyn MonitoringPort, DbError> {
        match driver {
            DriverType::Postgres => Ok(self.pg.as_ref()),
            DriverType::SQLite => Ok(self.sqlite.as_ref()),
            DriverType::Mysql => Err(DbError::Unsupported(
                "MySQL session monitoring is not enabled yet".into(),
            )),
        }
    }

    async fn driver(&self, connection_id: &ConnectionId) -> Result<DriverType, DbError> {
        let config = self
            .connections
            .get_config(connection_id)
            .await?
            .ok_or_else(|| DbError::ConnectionFailed(format!("connection {connection_id} not found")))?;
        Ok(config.driver)
    }

    pub async fn snapshot(&self, connection_id: &ConnectionId) -> Result<MonitoringSnapshot, DbError> {
        let driver = self.driver(connection_id).await?;
        let port = self.port_for(driver)?;
        let handle = self
            .registry
            .get(connection_id)
            .ok_or_else(|| DbError::ConnectionFailed(format!("connection {connection_id} is not active")))?;

        let sessions = match driver {
            DriverType::Postgres => port.list_sessions(&handle).await?,
            DriverType::SQLite => Vec::new(),
            DriverType::Mysql => unreachable!("port_for rejects mysql"),
        };
        let local = match driver {
            DriverType::SQLite => port.local_state(&handle).await?,
            DriverType::Postgres => None,
            DriverType::Mysql => None,
        };

        let message = match driver {
            DriverType::Postgres => format!("{} session(s)", sessions.len()),
            DriverType::SQLite => "SQLite local file state (no server sessions)".into(),
            DriverType::Mysql => "MySQL session monitoring is not enabled yet".into(),
        };

        Ok(MonitoringSnapshot {
            connection_id: connection_id.to_string(),
            driver: format!("{driver:?}"),
            sessions,
            local,
            fetched_at_ms: now_ms(),
            message,
        })
    }

    pub async fn cancel_backend(&self, connection_id: &ConnectionId, backend_id: i64) -> Result<bool, DbError> {
        self.ensure_pg(connection_id).await?;
        validate_backend_id(backend_id)?;
        let handle = self.active_handle(connection_id)?;
        self.pg.cancel_backend(&handle, backend_id).await
    }

    pub async fn terminate_backend(&self, connection_id: &ConnectionId, backend_id: i64) -> Result<bool, DbError> {
        self.ensure_pg(connection_id).await?;
        validate_backend_id(backend_id)?;
        let handle = self.active_handle(connection_id)?;
        self.pg.terminate_backend(&handle, backend_id).await
    }

    fn active_handle(
        &self,
        connection_id: &ConnectionId,
    ) -> Result<crate::domain::connection::ConnectionHandle, DbError> {
        self.registry
            .get(connection_id)
            .ok_or_else(|| DbError::ConnectionFailed(format!("connection {connection_id} is not active")))
    }

    async fn ensure_pg(&self, connection_id: &ConnectionId) -> Result<(), DbError> {
        match self.driver(connection_id).await? {
            DriverType::Postgres => Ok(()),
            other => Err(DbError::Unsupported(format!(
                "cancel/terminate backends is PostgreSQL-only (got {other:?})"
            ))),
        }
    }
}

fn validate_backend_id(backend_id: i64) -> Result<(), DbError> {
    if backend_id <= 0 {
        return Err(DbError::Validation(
            "backend id must be a positive PostgreSQL pid".into(),
        ));
    }
    Ok(())
}

fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::connection::{ConnectionConfig, ConnectionHandle, ConnectionId, DriverType, SslMode};
    use crate::domain::monitoring::{LocalMonitorState, MonitorSession};
    use crate::ports::{MockConnectionRepository, MockMonitoringPort};
    use mockall::predicate::eq;
    use std::sync::Arc;

    fn config(driver: DriverType) -> ConnectionConfig {
        ConnectionConfig {
            name: "t".into(),
            host: "127.0.0.1".into(),
            port: 5432,
            database: "db".into(),
            username: "u".into(),
            driver,
            ssl_mode: SslMode::Disable,
            ssh_tunnel: None,
            query_timeout_ms: 30_000,
            max_rows: 500,
            color: None,
            tags: vec![],
            group: None,
            readonly: false,
        }
    }

    #[tokio::test]
    async fn snapshot_lists_postgres_sessions_via_port() {
        let id = ConnectionId::new();
        let handle = ConnectionHandle::new(1);
        let mut repo = MockConnectionRepository::new();
        let cfg = config(DriverType::Postgres);
        repo.expect_get_config().returning(move |_| Ok(Some(cfg.clone())));

        let mut port = MockMonitoringPort::new();
        port.expect_list_sessions().returning(|_| {
            Ok(vec![MonitorSession {
                backend_id: 42,
                database: Some("db".into()),
                username: Some("u".into()),
                application_name: Some("db-pro".into()),
                client_addr: None,
                state: Some("active".into()),
                wait_event_type: None,
                wait_event: None,
                query_text: Some("SELECT 1".into()),
                query_duration_ms: Some(12),
                backend_start: None,
                xact_start: None,
                query_start: None,
                is_current: false,
            }])
        });

        let registry = Arc::new(ConnectionRegistry::new());
        registry.register(id, handle);

        let service = MonitoringService::new(
            Box::new(port),
            Box::new(MockMonitoringPort::new()),
            registry,
            Box::new(repo),
        );
        let snap = service.snapshot(&id).await.unwrap();
        assert_eq!(snap.sessions.len(), 1);
        assert_eq!(snap.sessions[0].backend_id, 42);
        assert_eq!(snap.active_queries().len(), 1);
    }

    #[tokio::test]
    async fn sqlite_snapshot_uses_local_state_not_fake_sessions() {
        let id = ConnectionId::new();
        let handle = ConnectionHandle::new(1);
        let mut repo = MockConnectionRepository::new();
        let cfg = config(DriverType::SQLite);
        repo.expect_get_config().returning(move |_| Ok(Some(cfg.clone())));

        let mut sqlite = MockMonitoringPort::new();
        sqlite.expect_list_sessions().times(0);
        sqlite.expect_local_state().returning(|_| {
            Ok(Some(LocalMonitorState {
                journal_mode: Some("wal".into()),
                page_count: Some(10),
                page_size: Some(4096),
                freelist_count: Some(0),
                file_size_bytes: Some(40960),
                note: "local only".into(),
            }))
        });

        let registry = Arc::new(ConnectionRegistry::new());
        registry.register(id, handle);

        let service = MonitoringService::new(
            Box::new(MockMonitoringPort::new()),
            Box::new(sqlite),
            registry,
            Box::new(repo),
        );
        let snap = service.snapshot(&id).await.unwrap();
        assert!(snap.sessions.is_empty());
        assert_eq!(snap.local.as_ref().unwrap().journal_mode.as_deref(), Some("wal"));
    }

    #[tokio::test]
    async fn cancel_rejects_non_positive_backend_id() {
        let id = ConnectionId::new();
        let mut repo = MockConnectionRepository::new();
        let cfg = config(DriverType::Postgres);
        repo.expect_get_config().returning(move |_| Ok(Some(cfg.clone())));
        let port = MockMonitoringPort::new();
        let registry = Arc::new(ConnectionRegistry::new());
        let service = MonitoringService::new(
            Box::new(port),
            Box::new(MockMonitoringPort::new()),
            registry,
            Box::new(repo),
        );
        let err = service.cancel_backend(&id, 0).await.unwrap_err();
        assert!(err.to_string().contains("positive"));
    }

    #[tokio::test]
    async fn cancel_targets_exact_backend_id() {
        let id = ConnectionId::new();
        let handle = ConnectionHandle::new(7);
        let mut repo = MockConnectionRepository::new();
        let cfg = config(DriverType::Postgres);
        repo.expect_get_config().returning(move |_| Ok(Some(cfg.clone())));

        let mut port = MockMonitoringPort::new();
        port.expect_cancel_backend()
            .with(eq(handle), eq(99_i64))
            .returning(|_, _| Ok(true));

        let registry = Arc::new(ConnectionRegistry::new());
        registry.register(id, handle);
        let service = MonitoringService::new(
            Box::new(port),
            Box::new(MockMonitoringPort::new()),
            registry,
            Box::new(repo),
        );
        assert!(service.cancel_backend(&id, 99).await.unwrap());
    }
}
