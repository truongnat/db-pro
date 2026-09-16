use async_trait::async_trait;

use crate::domain::connection::ConnectionHandle;
use crate::domain::error::DbError;
use crate::domain::user::{DatabaseUser, Privilege, PrivilegeObjectKind, RoleAttributes, RoleMembership};

#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait UserManager: Send + Sync {
    async fn list_users(&self, handle: &ConnectionHandle) -> Result<Vec<DatabaseUser>, DbError>;
    async fn create_role(&self, handle: &ConnectionHandle, name: &str, login: bool) -> Result<(), DbError>;
    async fn drop_role(&self, handle: &ConnectionHandle, name: &str) -> Result<(), DbError>;
    async fn alter_role(
        &self,
        handle: &ConnectionHandle,
        name: &str,
        attributes: &RoleAttributes,
    ) -> Result<(), DbError>;
    /// Update a role password. Implementations must never log the password value.
    async fn update_password(&self, handle: &ConnectionHandle, name: &str, password: &str) -> Result<(), DbError>;

    async fn list_memberships(&self, handle: &ConnectionHandle, member: &str) -> Result<Vec<RoleMembership>, DbError>;
    async fn grant_membership(&self, handle: &ConnectionHandle, role: &str, member: &str) -> Result<(), DbError>;
    async fn revoke_membership(&self, handle: &ConnectionHandle, role: &str, member: &str) -> Result<(), DbError>;

    async fn list_privileges(&self, handle: &ConnectionHandle, role_name: &str) -> Result<Vec<Privilege>, DbError>;
    async fn grant_privilege(
        &self,
        handle: &ConnectionHandle,
        role_name: &str,
        object_kind: PrivilegeObjectKind,
        schema: &str,
        object_name: &str,
        privilege: &str,
    ) -> Result<(), DbError>;
    async fn revoke_privilege(
        &self,
        handle: &ConnectionHandle,
        role_name: &str,
        object_kind: PrivilegeObjectKind,
        schema: &str,
        object_name: &str,
        privilege: &str,
    ) -> Result<(), DbError>;
}
