//! Feature-owned state for database-management surfaces.

#[derive(Default)]
pub(crate) struct RoutineState {
    pub(crate) routine_source_draft: String,
    pub(crate) routine_param_values: Vec<String>,
    pub(crate) routine_param_nulls: Vec<bool>,
    pub(crate) routine_ddl_preview: Option<String>,
    pub(crate) routine_drop_confirm: bool,
}

#[derive(Default)]
pub(crate) struct TransferState {
    pub(crate) transfer_jobs: Vec<db_pro_core::domain::transfer::TransferJob>,
}

pub(crate) struct SyntheticDataState {
    pub(crate) synthetic_table: String,
    pub(crate) synthetic_row_count: String,
    pub(crate) synthetic_seed: String,
    pub(crate) synthetic_null_pct: String,
    pub(crate) synthetic_preview: Option<db_pro_core::domain::synthetic_data::SyntheticPreview>,
    pub(crate) synthetic_error: Option<String>,
    pub(crate) synthetic_production_confirm: bool,
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

pub(crate) struct MaskingState {
    pub(crate) masking_columns_csv: String,
    pub(crate) masking_rule: db_pro_core::domain::masking::MaskRule,
    pub(crate) masking_keyed: bool,
    pub(crate) masking_preview: Option<db_pro_core::domain::masking::MaskingPreview>,
    pub(crate) masking_error: Option<String>,
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

pub(crate) struct MonitoringState {
    pub(crate) monitoring_snapshot: Option<db_pro_core::domain::monitoring::MonitoringSnapshot>,
    pub(crate) monitoring_error: Option<String>,
    pub(crate) monitoring_poll: bool,
    pub(crate) monitoring_last_poll: Option<std::time::Instant>,
    pub(crate) monitoring_terminate_confirm: Option<i64>,
    pub(crate) monitoring_filter_active_only: bool,
    pub(crate) monitoring_maintenance_confirm: Option<db_pro_core::domain::monitoring::MaintenanceAction>,
    pub(crate) monitoring_stat_sort: db_pro_core::domain::monitoring::StatStatementSort,
    pub(crate) monitoring_reset_stats_confirm: bool,
    pub(crate) monitoring_workload_prev: Option<db_pro_core::domain::monitoring::StatStatementsSnapshot>,
    pub(crate) monitoring_workload_filter: String,
}

impl Default for MonitoringState {
    fn default() -> Self {
        Self {
            monitoring_snapshot: None,
            monitoring_error: None,
            monitoring_poll: true,
            monitoring_last_poll: None,
            monitoring_terminate_confirm: None,
            monitoring_filter_active_only: true,
            monitoring_maintenance_confirm: None,
            monitoring_stat_sort: db_pro_core::domain::monitoring::StatStatementSort::TotalTime,
            monitoring_reset_stats_confirm: false,
            monitoring_workload_prev: None,
            monitoring_workload_filter: String::new(),
        }
    }
}

#[derive(Default)]
pub(crate) struct AuditState {
    pub(crate) audit_page: Option<db_pro_core::domain::audit::AuditPage>,
    pub(crate) audit_error: Option<String>,
    pub(crate) audit_filter_text: String,
    pub(crate) audit_filter_database: String,
    pub(crate) audit_filter_username: String,
    pub(crate) audit_filter_severity: String,
    pub(crate) audit_bookmarks: std::collections::HashSet<String>,
    pub(crate) audit_selected: std::collections::HashSet<String>,
    pub(crate) audit_export_preview: Option<String>,
}

#[derive(Default)]
pub(crate) struct PgSettingsState {
    pub(crate) pg_settings: Option<db_pro_core::domain::pg_settings::PgSettingsSnapshot>,
    pub(crate) pg_settings_filter: String,
    pub(crate) pg_settings_edit_name: String,
    pub(crate) pg_settings_edit_value: String,
    pub(crate) pg_settings_preview: Option<db_pro_core::domain::pg_settings::PgSettingPreviewSql>,
    pub(crate) pg_settings_error: Option<String>,
}

pub(crate) struct FdwState {
    pub(crate) fdw_inventory: Option<db_pro_core::domain::fdw::FdwInventory>,
    pub(crate) fdw_error: Option<String>,
    pub(crate) fdw_create_name: String,
    pub(crate) fdw_create_wrapper: String,
    pub(crate) fdw_create_host: String,
    pub(crate) fdw_create_dbname: String,
    pub(crate) fdw_create_port: String,
    pub(crate) fdw_ddl_preview: Option<String>,
    pub(crate) fdw_drop_confirm: Option<String>,
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
pub(crate) struct ReplicationState {
    pub(crate) replication_inventory: Option<db_pro_core::domain::replication::ReplicationInventory>,
    pub(crate) replication_error: Option<String>,
    pub(crate) replication_create_name: String,
    pub(crate) replication_ddl_preview: Option<String>,
    pub(crate) replication_drop_publication: Option<String>,
    pub(crate) replication_drop_subscription: Option<String>,
}

pub(crate) struct EventTriggerState {
    pub(crate) event_trigger_inventory: Option<db_pro_core::domain::event_trigger::EventTriggerInventory>,
    pub(crate) event_trigger_error: Option<String>,
    pub(crate) event_trigger_create_name: String,
    pub(crate) event_trigger_create_event: String,
    pub(crate) event_trigger_create_function: String,
    pub(crate) event_trigger_create_tags: String,
    pub(crate) event_trigger_ddl_preview: Option<String>,
    pub(crate) event_trigger_drop_confirm: Option<String>,
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

pub(crate) struct SecurityState {
    pub(crate) security_users: Vec<db_pro_core::domain::user::DatabaseUser>,
    pub(crate) security_selected_role: Option<String>,
    pub(crate) security_privileges: Vec<db_pro_core::domain::user::Privilege>,
    pub(crate) security_memberships: Vec<db_pro_core::domain::user::RoleMembership>,
    pub(crate) security_new_role: String,
    pub(crate) security_new_role_login: bool,
    pub(crate) security_membership_role: String,
    pub(crate) security_password: String,
    pub(crate) security_grant_kind: db_pro_core::domain::user::PrivilegeObjectKind,
    pub(crate) security_grant_schema: String,
    pub(crate) security_grant_object: String,
    pub(crate) security_grant_privilege: String,
    pub(crate) security_rls_schema: String,
    pub(crate) security_rls_table: String,
    pub(crate) security_rls_state: Option<db_pro_core::domain::rls::TableRlsState>,
    pub(crate) security_rls_policy_name: String,
    pub(crate) security_rls_command: String,
    pub(crate) security_rls_roles: String,
    pub(crate) security_rls_using: String,
    pub(crate) security_rls_with_check: String,
    pub(crate) security_rls_preview_sql: String,
    pub(crate) security_rls_confirm_apply: bool,
    pub(crate) security_drop_confirm: Option<String>,
    pub(crate) security_error: Option<String>,
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
    use super::{MonitoringState, SecurityState, SyntheticDataState};

    #[test]
    fn destructive_management_defaults_are_safe() {
        let synthetic = SyntheticDataState::default();
        let security = SecurityState::default();

        assert!(!synthetic.synthetic_production_confirm);
        assert!(security.security_users.is_empty());
        assert_eq!(security.security_rls_schema, "public");
    }

    #[test]
    fn monitoring_starts_with_bounded_default_filters() {
        let monitoring = MonitoringState::default();

        assert!(monitoring.monitoring_poll);
        assert!(monitoring.monitoring_filter_active_only);
        assert!(monitoring.monitoring_snapshot.is_none());
    }
}
