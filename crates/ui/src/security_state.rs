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
mod tests {
    use super::SecurityState;

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
            state.table_rls_target(),
            Err("Schema and table are required for RLS inspect".to_owned())
        );
    }

    #[test]
    fn empty_rls_preview_cannot_dispatch_ddl() {
        let state = SecurityState::default();

        assert!(state.rls_preview_sql().is_none());

        let state = SecurityState {
            security_rls_preview_sql: " ALTER TABLE users ENABLE ROW LEVEL SECURITY ".to_owned(),
            ..SecurityState::default()
        };
        assert_eq!(state.rls_preview_sql().as_deref(), Some("ALTER TABLE users ENABLE ROW LEVEL SECURITY"));
    }
}
