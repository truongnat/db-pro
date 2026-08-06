use std::sync::Arc;

use crate::domain::connection::ConnectionId;
use crate::domain::error::DbError;
use crate::domain::user::{DatabaseUser, Privilege};
use crate::ports::UserManager;

use super::registry::ConnectionRegistry;

pub struct UserService {
    manager: Box<dyn UserManager>,
    registry: Arc<ConnectionRegistry>,
}

impl UserService {
    pub fn new(manager: Box<dyn UserManager>, registry: Arc<ConnectionRegistry>) -> Self {
        Self { manager, registry }
    }

    pub async fn list_users(&self, connection_id: &ConnectionId) -> Result<Vec<DatabaseUser>, DbError> {
        let handle = self.registry.get(connection_id)
            .ok_or_else(|| DbError::ConnectionFailed(format!("connection {connection_id} is not active")))?;
        self.manager.list_users(&handle).await
    }

    pub async fn create_role(&self, connection_id: &ConnectionId, name: &str, login: bool) -> Result<(), DbError> {
        let handle = self.registry.get(connection_id)
            .ok_or_else(|| DbError::ConnectionFailed(format!("connection {connection_id} is not active")))?;
        self.manager.create_role(&handle, name, login).await
    }

    pub async fn drop_role(&self, connection_id: &ConnectionId, name: &str) -> Result<(), DbError> {
        let handle = self.registry.get(connection_id)
            .ok_or_else(|| DbError::ConnectionFailed(format!("connection {connection_id} is not active")))?;
        self.manager.drop_role(&handle, name).await
    }

    pub async fn list_privileges(&self, connection_id: &ConnectionId, role_name: &str) -> Result<Vec<Privilege>, DbError> {
        let handle = self.registry.get(connection_id)
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
        let handle = self.registry.get(connection_id)
            .ok_or_else(|| DbError::ConnectionFailed(format!("connection {connection_id} is not active")))?;
        self.manager.grant_privilege(&handle, role_name, schema, table, privilege).await
    }

    pub async fn revoke_privilege(
        &self,
        connection_id: &ConnectionId,
        role_name: &str,
        schema: &str,
        table: &str,
        privilege: &str,
    ) -> Result<(), DbError> {
        let handle = self.registry.get(connection_id)
            .ok_or_else(|| DbError::ConnectionFailed(format!("connection {connection_id} is not active")))?;
        self.manager.revoke_privilege(&handle, role_name, schema, table, privilege).await
    }
}
