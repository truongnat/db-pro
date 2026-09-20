//! State owned by the security administration surface.

use super::{RequestId, UiCommand};

pub(super) struct SecurityState {
    pub(super) security_users: Vec<db_pro_core::domain::user::DatabaseUser>,
    pub(super) security_selected_role: Option<String>,
    pub(super) security_privileges: Vec<db_pro_core::domain::user::Privilege>,
    pub(super) security_memberships: Vec<db_pro_core::domain::user::RoleMembership>,
    pub(super) security_new_role: String,
    pub(super) security_new_role_login: bool,
    pub(super) security_membership_role: String,
    pub(super) security_password: String,
    pub(super) security_grant_kind: db_pro_core::domain::user::PrivilegeObjectKind,
    pub(super) security_grant_schema: String,
    pub(super) security_grant_object: String,
    pub(super) security_grant_privilege: String,
    pub(super) security_rls_schema: String,
    pub(super) security_rls_table: String,
    pub(super) security_rls_state: Option<db_pro_core::domain::rls::TableRlsState>,
    pub(super) security_rls_policy_name: String,
    pub(super) security_rls_command: String,
    pub(super) security_rls_roles: String,
    pub(super) security_rls_using: String,
    pub(super) security_rls_with_check: String,
    pub(super) security_rls_preview_sql: String,
    pub(super) security_rls_confirm_apply: bool,
    pub(super) security_drop_confirm: Option<String>,
    pub(super) security_error: Option<String>,
}

impl Default for SecurityState {
    fn default() -> Self {
        Self {
            security_users: Vec::new(),
            security_selected_role: None,
            security_privileges: Vec::new(),
            security_memberships: Vec::new(),
            security_new_role: String::new(),
            security_new_role_login: true,
            security_membership_role: String::new(),
            security_password: String::new(),
            security_grant_kind: db_pro_core::domain::user::PrivilegeObjectKind::Table,
            security_grant_schema: String::new(),
            security_grant_object: String::new(),
            security_grant_privilege: "SELECT".to_owned(),
            security_rls_schema: "public".to_owned(),
            security_rls_table: String::new(),
            security_rls_state: None,
            security_rls_policy_name: String::new(),
            security_rls_command: "SELECT".to_owned(),
            security_rls_roles: String::new(),
            security_rls_using: String::new(),
            security_rls_with_check: String::new(),
            security_rls_preview_sql: String::new(),
            security_rls_confirm_apply: false,
            security_drop_confirm: None,
            security_error: None,
        }
    }
}

impl SecurityState {
    pub(super) fn list_users_command(&self, request_id: RequestId, connection_id: String) -> UiCommand {
        UiCommand::ListUsers {
            request_id,
            connection_id,
        }
    }

    pub(super) fn list_privileges_command(
        &self,
        request_id: RequestId,
        connection_id: String,
        role_name: String,
    ) -> UiCommand {
        UiCommand::ListPrivileges {
            request_id,
            connection_id,
            role_name,
        }
    }

    pub(super) fn list_memberships_command(
        &self,
        request_id: RequestId,
        connection_id: String,
        member: String,
    ) -> UiCommand {
        UiCommand::ListMemberships {
            request_id,
            connection_id,
            member,
        }
    }

    pub(super) fn list_table_rls_command(
        &self,
        request_id: RequestId,
        connection_id: String,
    ) -> Result<UiCommand, String> {
        let schema = self.security_rls_schema.trim();
        if schema.is_empty() {
            return Err("Schema and table are required for RLS inspect".to_owned());
        }
        let table = self.security_rls_table.trim();
        if table.is_empty() {
            return Err("Schema and table are required for RLS inspect".to_owned());
        }
        Ok(UiCommand::ListTableRls {
            request_id,
            connection_id,
            schema: schema.to_owned(),
            table: table.to_owned(),
        })
    }

    pub(super) fn create_role_command(&self, request_id: RequestId, connection_id: String, name: String) -> UiCommand {
        UiCommand::CreateRole {
            request_id,
            connection_id,
            name,
            login: self.security_new_role_login,
        }
    }

    pub(super) fn drop_role_command(&self, request_id: RequestId, connection_id: String, name: String) -> UiCommand {
        UiCommand::DropRole {
            request_id,
            connection_id,
            name,
        }
    }

    pub(super) fn alter_role_command(
        &self,
        request_id: RequestId,
        connection_id: String,
        name: String,
        attributes: db_pro_core::domain::user::RoleAttributes,
    ) -> UiCommand {
        UiCommand::AlterRole {
            request_id,
            connection_id,
            name,
            attributes,
        }
    }

    pub(super) fn update_password_command(
        &self,
        request_id: RequestId,
        connection_id: String,
        name: String,
        password: String,
    ) -> UiCommand {
        UiCommand::UpdateRolePassword {
            request_id,
            connection_id,
            name,
            password,
        }
    }

    pub(super) fn grant_membership_command(
        &self,
        request_id: RequestId,
        connection_id: String,
        role: String,
        member: String,
    ) -> UiCommand {
        UiCommand::GrantMembership {
            request_id,
            connection_id,
            role,
            member,
        }
    }

    pub(super) fn revoke_membership_command(
        &self,
        request_id: RequestId,
        connection_id: String,
        role: String,
        member: String,
    ) -> UiCommand {
        UiCommand::RevokeMembership {
            request_id,
            connection_id,
            role,
            member,
        }
    }

    pub(super) fn grant_privilege_command(
        &self,
        request_id: RequestId,
        connection_id: String,
        role_name: String,
    ) -> UiCommand {
        UiCommand::GrantPrivilege {
            request_id,
            connection_id,
            role_name,
            object_kind: self.security_grant_kind,
            schema: self.security_grant_schema.trim().to_owned(),
            object_name: self.security_grant_object.trim().to_owned(),
            privilege: self.security_grant_privilege.trim().to_owned(),
        }
    }

    pub(super) fn revoke_privilege_command(
        &self,
        request_id: RequestId,
        connection_id: String,
        role_name: String,
        privilege: db_pro_core::domain::user::Privilege,
    ) -> UiCommand {
        UiCommand::RevokePrivilege {
            request_id,
            connection_id,
            role_name,
            object_kind: privilege.object_kind,
            schema: privilege.schema,
            object_name: privilege.object_name,
            privilege: privilege.privilege_type,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{RequestId, SecurityState};

    #[test]
    fn defaults_keep_security_scope_explicit() {
        let security = SecurityState::default();

        assert!(security.security_users.is_empty());
        assert_eq!(security.security_rls_schema, "public");
        assert!(!security.security_rls_confirm_apply);
    }

    #[test]
    fn rls_inspection_requires_schema_and_table() {
        let state = SecurityState::default();

        assert_eq!(
            state.list_table_rls_command(RequestId(1), "source".to_owned()),
            Err("Schema and table are required for RLS inspect".to_owned())
        );
    }
}
