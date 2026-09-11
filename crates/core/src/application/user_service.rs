use std::sync::Arc;

use crate::domain::connection::{ConnectionConfig, ConnectionId, DriverType};
use crate::domain::error::DbError;
use crate::domain::user::{DatabaseUser, Privilege};
use crate::ports::{ConnectionRepository, UserManager};

use super::registry::ConnectionRegistry;

pub struct UserService {
    manager: Box<dyn UserManager>,
    registry: Arc<ConnectionRegistry>,
    connections: Box<dyn ConnectionRepository>,
}

impl UserService {
    pub fn new(
        manager: Box<dyn UserManager>,
        registry: Arc<ConnectionRegistry>,
        connections: Box<dyn ConnectionRepository>,
    ) -> Self {
        Self {
            manager,
            registry,
            connections,
        }
    }

    /// Ensure the connection allows write operations.
    async fn ensure_writable(&self, connection_id: &ConnectionId) -> Result<(), DbError> {
        let config = self.connection_config(connection_id).await?;
        ensure_server_sessions_config(&config)?;
        if config.readonly {
            return Err(DbError::QueryFailed(
                "connection is read-only — user management operations are not allowed".into(),
            ));
        }
        Ok(())
    }

    async fn ensure_server_sessions(&self, connection_id: &ConnectionId) -> Result<(), DbError> {
        let config = self.connection_config(connection_id).await?;
        ensure_server_sessions_config(&config)
    }

    async fn connection_config(&self, connection_id: &ConnectionId) -> Result<ConnectionConfig, DbError> {
        self.connections
            .get_config(connection_id)
            .await?
            .ok_or_else(|| DbError::ConnectionFailed(format!("connection {connection_id} not found")))
    }

    pub async fn list_users(&self, connection_id: &ConnectionId) -> Result<Vec<DatabaseUser>, DbError> {
        self.ensure_server_sessions(connection_id).await?;
        let handle = self
            .registry
            .get(connection_id)
            .ok_or_else(|| DbError::ConnectionFailed(format!("connection {connection_id} is not active")))?;
        self.manager.list_users(&handle).await
    }

    pub async fn create_role(&self, connection_id: &ConnectionId, name: &str, login: bool) -> Result<(), DbError> {
        self.ensure_writable(connection_id).await?;
        let handle = self
            .registry
            .get(connection_id)
            .ok_or_else(|| DbError::ConnectionFailed(format!("connection {connection_id} is not active")))?;
        self.manager.create_role(&handle, name, login).await
    }

    pub async fn drop_role(&self, connection_id: &ConnectionId, name: &str) -> Result<(), DbError> {
        self.ensure_writable(connection_id).await?;
        let handle = self
            .registry
            .get(connection_id)
            .ok_or_else(|| DbError::ConnectionFailed(format!("connection {connection_id} is not active")))?;
        self.manager.drop_role(&handle, name).await
    }

    pub async fn list_privileges(
        &self,
        connection_id: &ConnectionId,
        role_name: &str,
    ) -> Result<Vec<Privilege>, DbError> {
        self.ensure_server_sessions(connection_id).await?;
        let handle = self
            .registry
            .get(connection_id)
            .ok_or_else(|| DbError::ConnectionFailed(format!("connection {connection_id} is not active")))?;
        self.manager.list_privileges(&handle, role_name).await
    }

    pub async fn grant_privilege(
        &self,
        connection_id: &ConnectionId,
        role_name: &str,
        schema: &str,
        table: &str,
        privilege: &str,
    ) -> Result<(), DbError> {
        self.ensure_writable(connection_id).await?;
        let handle = self
            .registry
            .get(connection_id)
            .ok_or_else(|| DbError::ConnectionFailed(format!("connection {connection_id} is not active")))?;
        self.manager
            .grant_privilege(&handle, role_name, schema, table, privilege)
            .await
    }

    pub async fn revoke_privilege(
        &self,
        connection_id: &ConnectionId,
        role_name: &str,
        schema: &str,
        table: &str,
        privilege: &str,
    ) -> Result<(), DbError> {
        self.ensure_writable(connection_id).await?;
        let handle = self
            .registry
            .get(connection_id)
            .ok_or_else(|| DbError::ConnectionFailed(format!("connection {connection_id} is not active")))?;
        self.manager
            .revoke_privilege(&handle, role_name, schema, table, privilege)
            .await
    }
}

fn ensure_server_sessions_config(config: &ConnectionConfig) -> Result<(), DbError> {
    if config.driver != DriverType::Postgres {
        return Err(DbError::Unsupported(
            "user management requires a PostgreSQL connection".into(),
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::connection::{ConnectionConfig, SslMode};
    use crate::ports::{MockConnectionRepository, MockUserManager};

    #[tokio::test]
    async fn user_management_rejects_sqlite_before_provider_call() {
        let config = ConnectionConfig {
            name: "local-db".into(),
            host: String::new(),
            port: 0,
            database: "/tmp/db-pro-test.sqlite".into(),
            username: String::new(),
            driver: DriverType::SQLite,
            ssl_mode: SslMode::Disable,
            ssh_tunnel: None,
            query_timeout_ms: 30_000,
            max_rows: 500,
            color: None,
            tags: vec![],
            group: None,
            readonly: false,
        };
        assert!(config.validate().is_ok());

        let mut connections = MockConnectionRepository::new();
        connections
            .expect_get_config()
            .returning(move |_| Ok(Some(config.clone())));

        let service = UserService::new(
            Box::new(MockUserManager::new()),
            Arc::new(ConnectionRegistry::new()),
            Box::new(connections),
        );
        let error = service
            .list_users(&ConnectionId::new())
            .await
            .expect_err("SQLite must not enter PostgreSQL user-management provider");

        assert!(matches!(error, DbError::Unsupported(message) if message.contains("PostgreSQL")));
    }
}
