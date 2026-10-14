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
#[path = "tests/rls_service_tests.rs"]
mod tests;
