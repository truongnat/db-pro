//! Provider-aware monitoring service (#196). UI never queries pg_stat_activity.

use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::domain::connection::{ConnectionId, DriverType};
use crate::domain::error::DbError;
use crate::domain::monitoring::MonitoringSnapshot;
use crate::ports::{ConnectionRepository, MonitoringPort};

use super::registry::ConnectionRegistry;

#[path = "monitoring_snapshot.rs"]
mod monitoring_snapshot;

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
            DriverType::Mysql => Err(DbError::Unsupported("MySQL has no sessions".into())),
            DriverType::SqlServer => Err(DbError::Unsupported("SQL Server has no sessions".into())),
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

    pub async fn snapshot(
        &self,
        connection_id: &ConnectionId,
        sort: crate::domain::monitoring::StatStatementSort,
    ) -> Result<MonitoringSnapshot, DbError> {
        monitoring_snapshot::execute(self, connection_id, sort).await
    }

    /// Load a sorted/bounded `pg_stat_statements` slice (#251).
    pub async fn stat_statements(
        &self,
        connection_id: &ConnectionId,
        sort: crate::domain::monitoring::StatStatementSort,
        limit: usize,
    ) -> Result<crate::domain::monitoring::StatStatementsSnapshot, DbError> {
        self.ensure_pg(connection_id).await?;
        let handle = self.active_handle(connection_id)?;
        let limit = limit.clamp(1, 500);
        self.pg.stat_statements(&handle, sort, limit).await
    }

    /// Reset statement stats — requires explicit admin confirmation (#251).
    pub async fn reset_stat_statements(&self, connection_id: &ConnectionId, confirmed: bool) -> Result<(), DbError> {
        if !confirmed {
            return Err(DbError::Validation(
                "resetting pg_stat_statements requires administrative confirmation".into(),
            ));
        }
        self.ensure_pg(connection_id).await?;
        let config = self
            .connections
            .get_config(connection_id)
            .await?
            .ok_or_else(|| DbError::ConnectionFailed(format!("connection {connection_id} not found")))?;
        if config.readonly {
            return Err(DbError::QueryFailed(
                "connection is read-only — pg_stat_statements reset is not allowed".into(),
            ));
        }
        let handle = self.active_handle(connection_id)?;
        self.pg.reset_stat_statements(&handle).await
    }

    /// Deterministic health report from a fresh snapshot (#259). Does not mutate the DB.
    pub async fn health_report(
        &self,
        connection_id: &ConnectionId,
        config: &crate::domain::health_advisor::HealthAdvisorConfig,
    ) -> Result<crate::domain::health_advisor::HealthReport, DbError> {
        let snapshot = self
            .snapshot(connection_id, crate::domain::monitoring::StatStatementSort::TotalTime)
            .await?;
        let workload = snapshot.workload.clone();
        Ok(crate::domain::health_advisor::analyze_health(
            &snapshot,
            workload.as_ref(),
            config,
            now_ms(),
        ))
    }

    pub async fn run_maintenance(
        &self,
        connection_id: &ConnectionId,
        schema: Option<&str>,
        table: Option<&str>,
        action: crate::domain::monitoring::MaintenanceAction,
        confirmed: bool,
    ) -> Result<(), DbError> {
        if !confirmed {
            return Err(DbError::Validation(format!(
                "maintenance action {} requires explicit confirmation",
                action.as_label()
            )));
        }
        let driver = self.driver(connection_id).await?;
        let config = self
            .connections
            .get_config(connection_id)
            .await?
            .ok_or_else(|| DbError::ConnectionFailed(format!("connection {connection_id} not found")))?;
        if config.readonly {
            return Err(DbError::QueryFailed(
                "connection is read-only — maintenance actions are not allowed".into(),
            ));
        }
        let port = self.port_for(driver)?;
        let handle = self.active_handle(connection_id)?;
        port.run_maintenance(&handle, schema.map(str::to_owned), table.map(str::to_owned), action)
            .await
    }

    pub async fn cancel_backend(
        &self,
        connection_id: &ConnectionId,
        backend_id: i64,
        backend_start: Option<&str>,
    ) -> Result<bool, DbError> {
        self.ensure_pg(connection_id).await?;
        validate_backend_id(backend_id)?;
        let backend_start = validate_backend_start(backend_start)?;
        let handle = self.active_handle(connection_id)?;
        self.pg.cancel_backend(&handle, backend_id, backend_start).await
    }

    pub async fn terminate_backend(
        &self,
        connection_id: &ConnectionId,
        backend_id: i64,
        backend_start: Option<&str>,
    ) -> Result<bool, DbError> {
        self.ensure_pg(connection_id).await?;
        validate_backend_id(backend_id)?;
        let backend_start = validate_backend_start(backend_start)?;
        let handle = self.active_handle(connection_id)?;
        self.pg.terminate_backend(&handle, backend_id, backend_start).await
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

/// A terminate must carry the snapshot `backend_start` identity so the adapter
/// can prove the pid still belongs to the confirmed session (#501).
fn validate_backend_start(backend_start: Option<&str>) -> Result<&str, DbError> {
    backend_start
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .ok_or_else(|| DbError::Validation("backend action requires its snapshot backend_start identity".into()))
}

fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}

#[cfg(test)]
#[path = "tests/monitoring_service_tests.rs"]
mod tests;
