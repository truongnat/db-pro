//! Tests for `../audit_service.rs` (moved out of the source file).
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
