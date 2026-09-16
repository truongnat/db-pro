use std::sync::Arc;

use crate::domain::connection::{ConnectionConfig, ConnectionId, DriverType};
use crate::domain::error::DbError;
use crate::domain::rls::TableRlsState;
use crate::ports::{ConnectionRepository, RlsManager};

use super::registry::ConnectionRegistry;

pub struct RlsService {
    manager: Box<dyn RlsManager>,
    registry: Arc<ConnectionRegistry>,
    connections: Box<dyn ConnectionRepository>,
}

impl RlsService {
    pub fn new(
        manager: Box<dyn RlsManager>,
        registry: Arc<ConnectionRegistry>,
        connections: Box<dyn ConnectionRepository>,
    ) -> Self {
        Self {
            manager,
            registry,
            connections,
        }
    }

    async fn connection_config(&self, connection_id: &ConnectionId) -> Result<ConnectionConfig, DbError> {
        self.connections
            .get_config(connection_id)
            .await?
            .ok_or_else(|| DbError::ConnectionFailed(format!("connection {connection_id} not found")))
    }

    async fn ensure_postgres(&self, connection_id: &ConnectionId) -> Result<(), DbError> {
        let config = self.connection_config(connection_id).await?;
        if config.driver != DriverType::Postgres {
            return Err(DbError::Unsupported(
                "row-level security requires a PostgreSQL connection".into(),
            ));
        }
        Ok(())
    }

    pub async fn table_rls_state(
        &self,
        connection_id: &ConnectionId,
        schema: &str,
        table: &str,
    ) -> Result<TableRlsState, DbError> {
        self.ensure_postgres(connection_id).await?;
        let handle = self
            .registry
            .get(connection_id)
            .ok_or_else(|| DbError::ConnectionFailed(format!("connection {connection_id} is not active")))?;
        self.manager.table_rls_state(&handle, schema, table).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::connection::SslMode;
    use crate::ports::{MockConnectionRepository, MockRlsManager};

    #[tokio::test]
    async fn rls_rejects_sqlite_before_provider_call() {
        let config = ConnectionConfig {
            name: "local".into(),
            host: String::new(),
            port: 0,
            database: "/tmp/t.sqlite".into(),
            username: String::new(),
            driver: DriverType::SQLite,
            ssl_mode: SslMode::Disable,
            ssh_tunnel: None,
            ssh_profile_id: None,
            ssl_root_cert_path: None,
            ssl_client_cert_path: None,
            ssl_client_key_path: None,
            query_timeout_ms: 30_000,
            max_rows: 500,
            color: None,
            tags: vec![],
            group: None,
            favorite: false,
            environment: Default::default(),
            readonly: false,
        };
        let mut connections = MockConnectionRepository::new();
        connections
            .expect_get_config()
            .returning(move |_| Ok(Some(config.clone())));
        let service = RlsService::new(
            Box::new(MockRlsManager::new()),
            Arc::new(ConnectionRegistry::new()),
            Box::new(connections),
        );
        let err = service
            .table_rls_state(&ConnectionId::new(), "public", "t")
            .await
            .expect_err("sqlite gated");
        assert!(matches!(err, DbError::Unsupported(_)));
    }
}
