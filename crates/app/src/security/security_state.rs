//! State owned by the security administration surface.

pub(super) struct SecurityState {
    pub(super) security_users: Vec<db_pro_core::domain::user::DatabaseUser>,
    pub(super) security_selected_role: Option<String>,
    pub(super) security_privileges: Vec<db_pro_core::domain::user::Privilege>,
    pub(super) security_memberships: Vec<db_pro_core::domain::user::RoleMembership>,
    pub(super) security_new_role: String,
    pub(super) security_new_role_login: bool,
    pub(super) security_membership_role: String,
    pub(super) security_password: String,
    pub(super) security_show_password: bool,
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
    pub(super) security_rls_restrictive: bool,
    pub(super) security_drop_confirm: Option<String>,
    pub(super) security_pending: Option<PendingSecurityAction>,
    pub(super) security_privilege_request: Option<u64>,
    pub(super) security_error: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum PendingSecurityAction {
    EnableAttribute {
        role: String,
        attributes: db_pro_core::domain::user::RoleAttributes,
    },
    RevokeMembership {
        role: String,
        member_role: String,
    },
    RevokePrivilege {
        role: String,
        privilege: db_pro_core::domain::user::Privilege,
    },
}

impl PendingSecurityAction {
    pub(super) fn prompt(&self) -> String {
        match self {
            Self::EnableAttribute { role, attributes } => {
                let flag = if attributes.createdb == Some(true) {
                    "CREATEDB"
                } else {
                    "CREATEROLE"
                };
                format!("Enable {flag} for `{role}`?")
            }
            Self::RevokeMembership { role, member_role } => {
                format!("Revoke membership of `{member_role}` from `{role}`?")
            }
            Self::RevokePrivilege { role, privilege } => format!(
                "Revoke {} on {} from `{role}`?",
                privilege.privilege_type, privilege.object_name
            ),
        }
    }
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
            security_show_password: false,
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
            security_rls_restrictive: false,
            security_drop_confirm: None,
            security_pending: None,
            security_privilege_request: None,
            security_error: None,
        }
    }
}

impl SecurityState {
    pub(super) fn note_privilege_failure(&mut self, request_id: u64, message: &str) -> bool {
        if self.security_privilege_request != Some(request_id) {
            return false;
        }
        self.security_privilege_request = None;
        self.security_error = Some(message.to_owned());
        self.security_privileges.clear();
        true
    }
}

impl SecurityState {
    pub(super) fn table_rls_target(&self) -> Result<(String, String), String> {
        let schema = self.security_rls_schema.trim();
        if schema.is_empty() {
            return Err("Schema and table are required for RLS inspect".to_owned());
        }
        let table = self.security_rls_table.trim();
        if table.is_empty() {
            return Err("Schema and table are required for RLS inspect".to_owned());
        }
        Ok((schema.to_owned(), table.to_owned()))
    }

    pub(super) fn rls_preview_sql(&self) -> Option<String> {
        let sql = self.security_rls_preview_sql.trim();
        if sql.is_empty() {
            return None;
        }
        Some(sql.to_owned())
    }
}

#[cfg(test)]
#[path = "tests/security_state_tests.rs"]
mod tests;

