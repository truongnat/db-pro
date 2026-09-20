//! Feature-owned state for database-management surfaces.

#[derive(Default)]
pub(super) struct RoutineState {
    pub(super) routine_source_draft: String,
    pub(super) routine_param_values: Vec<String>,
    pub(super) routine_param_nulls: Vec<bool>,
    pub(super) routine_ddl_preview: Option<String>,
    pub(super) routine_drop_confirm: bool,
}

#[derive(Default)]
pub(super) struct TransferState {
    pub(super) transfer_jobs: Vec<db_pro_core::domain::transfer::TransferJob>,
}

pub(super) struct SyntheticDataState {
    pub(super) synthetic_table: String,
    pub(super) synthetic_row_count: String,
    pub(super) synthetic_seed: String,
    pub(super) synthetic_null_pct: String,
    pub(super) synthetic_preview: Option<db_pro_core::domain::synthetic_data::SyntheticPreview>,
    pub(super) synthetic_error: Option<String>,
    pub(super) synthetic_production_confirm: bool,
}

impl Default for SyntheticDataState {
    fn default() -> Self {
        Self {
            synthetic_table: String::new(),
            synthetic_row_count: "10".to_owned(),
            synthetic_seed: "42".to_owned(),
            synthetic_null_pct: "0".to_owned(),
            synthetic_preview: None,
            synthetic_error: None,
            synthetic_production_confirm: false,
        }
    }
}

pub(super) struct MaskingState {
    pub(super) masking_columns_csv: String,
    pub(super) masking_rule: db_pro_core::domain::masking::MaskRule,
    pub(super) masking_keyed: bool,
    pub(super) masking_preview: Option<db_pro_core::domain::masking::MaskingPreview>,
    pub(super) masking_error: Option<String>,
}

impl Default for MaskingState {
    fn default() -> Self {
        Self {
            masking_columns_csv: "email,phone".to_owned(),
            masking_rule: db_pro_core::domain::masking::MaskRule::PartialReveal,
            masking_keyed: true,
            masking_preview: None,
            masking_error: None,
        }
    }
}

#[derive(Default)]
pub(super) struct PgSettingsState {
    pub(super) pg_settings: Option<db_pro_core::domain::pg_settings::PgSettingsSnapshot>,
    pub(super) pg_settings_filter: String,
    pub(super) pg_settings_edit_name: String,
    pub(super) pg_settings_edit_value: String,
    pub(super) pg_settings_preview: Option<db_pro_core::domain::pg_settings::PgSettingPreviewSql>,
    pub(super) pg_settings_error: Option<String>,
}

pub(super) struct FdwState {
    pub(super) fdw_inventory: Option<db_pro_core::domain::fdw::FdwInventory>,
    pub(super) fdw_error: Option<String>,
    pub(super) fdw_create_name: String,
    pub(super) fdw_create_wrapper: String,
    pub(super) fdw_create_host: String,
    pub(super) fdw_create_dbname: String,
    pub(super) fdw_create_port: String,
    pub(super) fdw_ddl_preview: Option<String>,
    pub(super) fdw_drop_confirm: Option<String>,
}

impl Default for FdwState {
    fn default() -> Self {
        Self {
            fdw_inventory: None,
            fdw_error: None,
            fdw_create_name: String::new(),
            fdw_create_wrapper: "postgres_fdw".to_owned(),
            fdw_create_host: String::new(),
            fdw_create_dbname: String::new(),
            fdw_create_port: "5432".to_owned(),
            fdw_ddl_preview: None,
            fdw_drop_confirm: None,
        }
    }
}

#[derive(Default)]
pub(super) struct ReplicationState {
    pub(super) replication_inventory: Option<db_pro_core::domain::replication::ReplicationInventory>,
    pub(super) replication_error: Option<String>,
    pub(super) replication_create_name: String,
    pub(super) replication_ddl_preview: Option<String>,
    pub(super) replication_drop_publication: Option<String>,
    pub(super) replication_drop_subscription: Option<String>,
}

pub(super) struct EventTriggerState {
    pub(super) event_trigger_inventory: Option<db_pro_core::domain::event_trigger::EventTriggerInventory>,
    pub(super) event_trigger_error: Option<String>,
    pub(super) event_trigger_create_name: String,
    pub(super) event_trigger_create_event: String,
    pub(super) event_trigger_create_function: String,
    pub(super) event_trigger_create_tags: String,
    pub(super) event_trigger_ddl_preview: Option<String>,
    pub(super) event_trigger_drop_confirm: Option<String>,
}

impl Default for EventTriggerState {
    fn default() -> Self {
        Self {
            event_trigger_inventory: None,
            event_trigger_error: None,
            event_trigger_create_name: String::new(),
            event_trigger_create_event: "ddl_command_end".to_owned(),
            event_trigger_create_function: String::new(),
            event_trigger_create_tags: String::new(),
            event_trigger_ddl_preview: None,
            event_trigger_drop_confirm: None,
        }
    }
}

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

#[cfg(test)]
mod tests {
    use super::{SecurityState, SyntheticDataState};

    #[test]
    fn destructive_management_defaults_are_safe() {
        let synthetic = SyntheticDataState::default();
        let security = SecurityState::default();

        assert!(!synthetic.synthetic_production_confirm);
        assert!(security.security_users.is_empty());
        assert_eq!(security.security_rls_schema, "public");
    }
}
