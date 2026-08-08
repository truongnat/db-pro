use std::sync::Arc;

use crate::domain::connection::ConnectionId;
use crate::domain::error::DbError;
use crate::domain::safety::ConnectionSafetyPolicy;
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

    /// Build the safety policy for a connection based on its persisted config.
    async fn safety_policy_for(&self, connection_id: &ConnectionId) -> Result<ConnectionSafetyPolicy, DbError> {
        let config = self
            .connections
            .get_config(connection_id)
            .await?
            .ok_or_else(|| DbError::ConnectionFailed(format!("connection {connection_id} not found")))?;
        if config.readonly {
            Ok(ConnectionSafetyPolicy::read_only())
        } else {
            Ok(ConnectionSafetyPolicy::full_access())
        }
    }

    /// Ensure the connection allows write operations.
    async fn ensure_writable(&self, connection_id: &ConnectionId) -> Result<(), DbError> {
        let policy = self.safety_policy_for(connection_id).await?;
        if policy.read_only {
            return Err(DbError::QueryFailed(
                "connection is read-only — user management operations are not allowed".into(),
            ));
        }
        Ok(())
    }

    pub async fn list_users(&self, connection_id: &ConnectionId) -> Result<Vec<DatabaseUser>, DbError> {
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
