//! Audit / activity event service (#258).
//!
//! Reads explicitly configured PostgreSQL CSV logs. Never enables server
//! logging or installs extensions. Distinguishes DB audit from app logs.

use std::sync::Arc;

use crate::domain::audit::{
    csvlog_path_from_tags, export_events_csv, read_csvlog_page, unavailable_guidance, validate_csvlog_path, AuditEvent,
    AuditFilter, AuditPage, AuditSourceKind, AuditSourceStatus,
};
use crate::domain::connection::{ConnectionId, DriverType};
use crate::domain::error::DbError;
use crate::ports::{ConnectionRepository, DbConnector};

use super::registry::ConnectionRegistry;

const DEFAULT_LIMIT: usize = 100;
const DEFAULT_MAX_SCAN: u64 = 2 * 1024 * 1024;

pub struct AuditService {
    registry: Arc<ConnectionRegistry>,
    connections: Box<dyn ConnectionRepository>,
    connector: Arc<dyn DbConnector>,
}

impl AuditService {
    pub fn new(
        registry: Arc<ConnectionRegistry>,
        connections: Box<dyn ConnectionRepository>,
        connector: Arc<dyn DbConnector>,
    ) -> Self {
        Self {
            registry,
            connections,
            connector,
        }
    }

    async fn driver_and_tags(&self, connection_id: &ConnectionId) -> Result<(DriverType, Vec<String>), DbError> {
        let config = self
            .connections
            .get_config(connection_id)
            .await?
            .ok_or_else(|| DbError::ConnectionFailed(format!("connection {connection_id} not found")))?;
        Ok((config.driver, config.tags))
    }

    async fn pgaudit_present(&self, connection_id: &ConnectionId, driver: DriverType) -> bool {
        if driver != DriverType::Postgres {
            return false;
        }
        let Some(handle) = self.registry.get(connection_id) else {
            return false;
        };
        match self
            .connector
            .query(
                &handle,
                "SELECT 1 FROM pg_catalog.pg_extension WHERE extname = 'pgaudit' LIMIT 1",
                &[],
            )
            .await
        {
            Ok(result) => !result.rows.is_empty(),
            Err(_) => false,
        }
    }

    pub async fn source_status(&self, connection_id: &ConnectionId) -> Result<AuditSourceStatus, DbError> {
        let (driver, tags) = self.driver_and_tags(connection_id).await?;
        let pgaudit = self.pgaudit_present(connection_id, driver).await;
        let label = match driver {
            DriverType::Postgres => "PostgreSQL",
            DriverType::SQLite => "SQLite",
            DriverType::Mysql => "MySQL",
        };
        if driver != DriverType::Postgres {
            return Ok(AuditSourceStatus::unavailable(format!(
                "{label} has no canonical server audit log reader in DB Pro yet. \
                 Application logs remain on the Diagnostics surface."
            )));
        }
        match csvlog_path_from_tags(&tags) {
            Some(path) => match validate_csvlog_path(&path) {
                Ok(()) => Ok(AuditSourceStatus {
                    kind: AuditSourceKind::PostgresCsvLog,
                    path: Some(path.display().to_string()),
                    pgaudit_extension_present: pgaudit,
                    guidance: if pgaudit {
                        "Configured PostgreSQL CSV log is readable. pgaudit extension is present (records share this log stream)."
                            .into()
                    } else {
                        "Configured PostgreSQL CSV log is readable.".into()
                    },
                }),
                Err(err) => Ok(AuditSourceStatus {
                    kind: AuditSourceKind::Unavailable,
                    path: Some(path.display().to_string()),
                    pgaudit_extension_present: pgaudit,
                    guidance: format!("{err}. {}", unavailable_guidance(label, pgaudit)),
                }),
            },
            None => Ok(AuditSourceStatus {
                kind: AuditSourceKind::Unavailable,
                path: None,
                pgaudit_extension_present: pgaudit,
                guidance: unavailable_guidance(label, pgaudit),
            }),
        }
    }

    pub async fn load_page(
        &self,
        connection_id: &ConnectionId,
        filter: AuditFilter,
        limit: Option<usize>,
    ) -> Result<AuditPage, DbError> {
        let status = self.source_status(connection_id).await?;
        if status.kind != AuditSourceKind::PostgresCsvLog {
            return Ok(AuditPage {
                source: status,
                events: Vec::new(),
                scanned_bytes: 0,
                truncated: false,
                export_warning: "No audit events to export until a csvlog source is configured.".into(),
            });
        }
        let path = status
            .path
            .clone()
            .ok_or_else(|| DbError::Internal("audit source missing path".into()))?;
        let limit = limit.unwrap_or(DEFAULT_LIMIT).clamp(1, 500);
        let mut page = read_csvlog_page(std::path::Path::new(&path), &filter, limit, DEFAULT_MAX_SCAN)
            .map_err(DbError::Internal)?;
        page.source.pgaudit_extension_present = status.pgaudit_extension_present;
        if status.pgaudit_extension_present {
            page.source.guidance.push_str(" pgaudit extension detected on server.");
        }
        Ok(page)
    }

    pub fn export_selected(events: &[AuditEvent]) -> String {
        export_events_csv(events)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::audit::{AuditSourceKind, AUDIT_CSVLOG_TAG_PREFIX};
    use crate::domain::connection::{Connection, ConnectionConfig, DriverType};
    use crate::ports::{ConnectionRepository, DbConnector, MockDbConnector};
    use async_trait::async_trait;
    use std::collections::HashMap;
    use std::sync::Mutex;

    struct MemConnections {
        configs: Mutex<HashMap<ConnectionId, ConnectionConfig>>,
    }

    #[async_trait]
    impl ConnectionRepository for MemConnections {
        async fn list(&self) -> Result<Vec<Connection>, DbError> {
            Ok(Vec::new())
        }
        async fn get(&self, _id: &ConnectionId) -> Result<Option<Connection>, DbError> {
            Ok(None)
        }
        async fn get_config(&self, id: &ConnectionId) -> Result<Option<ConnectionConfig>, DbError> {
            Ok(self.configs.lock().unwrap().get(id).cloned())
        }
        async fn save(&self, _connection: &Connection) -> Result<(), DbError> {
            Ok(())
        }
        async fn delete(&self, _id: &ConnectionId) -> Result<(), DbError> {
            Ok(())
        }
    }

    #[tokio::test]
    async fn unavailable_without_tag() {
        let id = ConnectionId::new();
        let mut configs = HashMap::new();
        configs.insert(
            id,
            ConnectionConfig {
                name: "pg".into(),
                driver: DriverType::Postgres,
                ..ConnectionConfig::default()
            },
        );
        let connections: Box<dyn ConnectionRepository> = Box::new(MemConnections {
            configs: Mutex::new(configs),
        });
        let registry = Arc::new(ConnectionRegistry::new());
        let connector: Arc<dyn DbConnector> = Arc::new(MockDbConnector::new());
        let svc = AuditService::new(registry, connections, connector);
        let status = svc.source_status(&id).await.unwrap();
        assert_eq!(status.kind, AuditSourceKind::Unavailable);
        assert!(status.guidance.contains(AUDIT_CSVLOG_TAG_PREFIX));
    }
}
