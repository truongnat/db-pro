use std::sync::Arc;

use crate::domain::connection::{ConnectionConfig, ConnectionId, DriverType};
use crate::domain::error::DbError;
use crate::domain::user::{DatabaseUser, Privilege, PrivilegeObjectKind, RoleAttributes, RoleMembership};
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

    async fn active_handle(
        &self,
        connection_id: &ConnectionId,
    ) -> Result<crate::domain::connection::ConnectionHandle, DbError> {
        self.registry
            .get(connection_id)
            .ok_or_else(|| DbError::ConnectionFailed(format!("connection {connection_id} is not active")))
    }

    pub async fn list_users(&self, connection_id: &ConnectionId) -> Result<Vec<DatabaseUser>, DbError> {
        self.ensure_server_sessions(connection_id).await?;
        let handle = self.active_handle(connection_id).await?;
        self.manager.list_users(&handle).await
    }

    pub async fn create_role(&self, connection_id: &ConnectionId, name: &str, login: bool) -> Result<(), DbError> {
        self.ensure_writable(connection_id).await?;
        let handle = self.active_handle(connection_id).await?;
        self.manager.create_role(&handle, name, login).await
    }

    pub async fn drop_role(&self, connection_id: &ConnectionId, name: &str) -> Result<(), DbError> {
        self.ensure_writable(connection_id).await?;
        let handle = self.active_handle(connection_id).await?;
        self.manager.drop_role(&handle, name).await
    }

    pub async fn alter_role(
        &self,
        connection_id: &ConnectionId,
        name: &str,
        attributes: RoleAttributes,
    ) -> Result<(), DbError> {
        self.ensure_writable(connection_id).await?;
        let handle = self.active_handle(connection_id).await?;
        self.manager.alter_role(&handle, name, &attributes).await
    }

    /// Update a role password. The password value must never be logged by callers.
    pub async fn update_password(
        &self,
        connection_id: &ConnectionId,
        name: &str,
        password: &str,
    ) -> Result<(), DbError> {
        self.ensure_writable(connection_id).await?;
        let handle = self.active_handle(connection_id).await?;
        self.manager.update_password(&handle, name, password).await
    }

    pub async fn list_memberships(
        &self,
        connection_id: &ConnectionId,
        member: &str,
    ) -> Result<Vec<RoleMembership>, DbError> {
        self.ensure_server_sessions(connection_id).await?;
        let handle = self.active_handle(connection_id).await?;
        self.manager.list_memberships(&handle, member).await
    }

    pub async fn grant_membership(
        &self,
        connection_id: &ConnectionId,
        role: &str,
        member: &str,
    ) -> Result<(), DbError> {
        self.ensure_writable(connection_id).await?;
        let handle = self.active_handle(connection_id).await?;
        self.manager.grant_membership(&handle, role, member).await
    }

    pub async fn revoke_membership(
        &self,
        connection_id: &ConnectionId,
        role: &str,
        member: &str,
    ) -> Result<(), DbError> {
        self.ensure_writable(connection_id).await?;
        let handle = self.active_handle(connection_id).await?;
        self.manager.revoke_membership(&handle, role, member).await
    }

    pub async fn list_privileges(
        &self,
        connection_id: &ConnectionId,
        role_name: &str,
    ) -> Result<Vec<Privilege>, DbError> {
        self.ensure_server_sessions(connection_id).await?;
        let handle = self.active_handle(connection_id).await?;
        self.manager.list_privileges(&handle, role_name).await
    }

    pub async fn grant_privilege(
        &self,
        connection_id: &ConnectionId,
        role_name: &str,
        object_kind: PrivilegeObjectKind,
        schema: &str,
        object_name: &str,
        privilege: &str,
    ) -> Result<(), DbError> {
        self.ensure_writable(connection_id).await?;
        let handle = self.active_handle(connection_id).await?;
        self.manager
            .grant_privilege(&handle, role_name, object_kind, schema, object_name, privilege)
            .await
    }

    pub async fn revoke_privilege(
        &self,
        connection_id: &ConnectionId,
        role_name: &str,
        object_kind: PrivilegeObjectKind,
        schema: &str,
        object_name: &str,
        privilege: &str,
    ) -> Result<(), DbError> {
        self.ensure_writable(connection_id).await?;
        let handle = self.active_handle(connection_id).await?;
        self.manager
            .revoke_privilege(&handle, role_name, object_kind, schema, object_name, privilege)
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
