//! Database-management event reducers over feature-owned state.

use super::database_feature_states::{EventTriggerState, FdwState, PgSettingsState, ReplicationState, SecurityState};
use super::monitoring_state::MonitoringState;
use super::schema_compare_state::SchemaCompareState;
use super::FeedbackState;

pub(super) fn on_operation_progress(feedback: &mut FeedbackState, operation: String, status: String) {
    feedback.set_runtime_message(format!("{operation}: {status}"));
}

pub(super) fn on_backup_completed(feedback: &mut FeedbackState, output_path: String, size_bytes: u64) {
    feedback.set_runtime_message(format!("Backup completed · {output_path} · {size_bytes} bytes"));
}

pub(super) fn on_monitoring_snapshot_loaded(
    monitoring: &mut MonitoringState,
    feedback: &mut FeedbackState,
    snapshot: db_pro_core::domain::monitoring::MonitoringSnapshot,
) {
    if let Some(previous) = monitoring.monitoring_snapshot.take() {
        monitoring.monitoring_workload_prev = previous.workload;
    }
    monitoring.monitoring_snapshot = Some(snapshot.clone());
    monitoring.monitoring_error = None;
    feedback.set_runtime_message(format!("Monitor · {}", snapshot.message));
}

pub(super) fn on_monitoring_workload_loaded(
    monitoring: &mut MonitoringState,
    feedback: &mut FeedbackState,
    workload: db_pro_core::domain::monitoring::StatStatementsSnapshot,
) {
    if let Some(snapshot) = monitoring.monitoring_snapshot.as_mut() {
        monitoring.monitoring_workload_prev = snapshot.workload.clone();
        snapshot.workload = Some(workload.clone());
    }
    monitoring.monitoring_stat_sort = workload.sort;
    feedback.set_runtime_message(format!("Workload · {}", workload.message));
}

pub(super) fn on_audit_page_loaded(
    audit_page: &mut Option<db_pro_core::domain::audit::AuditPage>,
    audit_error: &mut Option<String>,
    feedback: &mut FeedbackState,
    page: db_pro_core::domain::audit::AuditPage,
) {
    *audit_page = Some(page.clone());
    *audit_error = None;
    feedback.set_runtime_message(format!(
        "Audit · {} event(s) · {}",
        page.events.len(),
        page.source.guidance.chars().take(80).collect::<String>()
    ));
}

pub(super) fn on_pg_settings_loaded(
    pg_settings: &mut PgSettingsState,
    feedback: &mut FeedbackState,
    snapshot: db_pro_core::domain::pg_settings::PgSettingsSnapshot,
) {
    pg_settings.pg_settings = Some(snapshot.clone());
    pg_settings.pg_settings_error = None;
    feedback.set_runtime_message(format!("pg_settings · {}", snapshot.message));
}

pub(super) fn on_fdw_inventory_loaded(
    fdw: &mut FdwState,
    feedback: &mut FeedbackState,
    inventory: db_pro_core::domain::fdw::FdwInventory,
) {
    fdw.fdw_inventory = Some(inventory.clone());
    fdw.fdw_error = None;
    feedback.set_runtime_message(format!("FDW · {}", inventory.message));
}

pub(super) fn on_replication_inventory_loaded(
    replication: &mut ReplicationState,
    feedback: &mut FeedbackState,
    inventory: db_pro_core::domain::replication::ReplicationInventory,
) {
    replication.replication_inventory = Some(inventory.clone());
    replication.replication_error = None;
    feedback.set_runtime_message(format!("Replication · {}", inventory.message));
}

pub(super) fn on_event_trigger_inventory_loaded(
    event_trigger: &mut EventTriggerState,
    feedback: &mut FeedbackState,
    inventory: db_pro_core::domain::event_trigger::EventTriggerInventory,
) {
    event_trigger.event_trigger_inventory = Some(inventory.clone());
    event_trigger.event_trigger_error = None;
    feedback.set_runtime_message(format!("Event triggers · {}", inventory.message));
}

pub(super) fn on_users_loaded(
    security: &mut SecurityState,
    feedback: &mut FeedbackState,
    users: Vec<db_pro_core::domain::user::DatabaseUser>,
) {
    security.security_users = users;
    security.security_error = None;
    feedback.set_runtime_message(format!("Security · {} role(s)", security.security_users.len()));
}

pub(super) fn on_privileges_loaded(
    security: &mut SecurityState,
    role_name: String,
    privileges: Vec<db_pro_core::domain::user::Privilege>,
) {
    security.security_selected_role = Some(role_name);
    security.security_privileges = privileges;
}

pub(super) fn on_memberships_loaded(
    security: &mut SecurityState,
    member: String,
    memberships: Vec<db_pro_core::domain::user::RoleMembership>,
) {
    security.security_selected_role = Some(member);
    security.security_memberships = memberships;
}

pub(super) fn on_table_rls_loaded(
    security: &mut SecurityState,
    feedback: &mut FeedbackState,
    state: db_pro_core::domain::rls::TableRlsState,
) {
    security.security_rls_state = Some(state);
    security.security_error = None;
    feedback.set_runtime_message("Security · RLS state loaded");
}

pub(super) fn on_data_diff_loaded(
    schema_compare: &mut SchemaCompareState,
    feedback: &mut FeedbackState,
    diff: db_pro_core::domain::cross_connection::DataDiff,
) {
    schema_compare.data_diff_result = Some(diff);
    feedback.set_runtime_message("Data compare ready");
}

pub(super) fn on_pg_setting_action_completed(feedback: &mut FeedbackState, action: String, name: String) {
    feedback.set_runtime_message(format!("pg_settings {action} `{name}` ok"));
}

pub(super) fn on_fdw_action_completed(fdw: &mut FdwState, feedback: &mut FeedbackState, action: String, name: String) {
    feedback.set_runtime_message(format!("FDW {action} `{name}` ok"));
    fdw.fdw_drop_confirm = None;
    fdw.fdw_ddl_preview = None;
}

pub(super) fn on_replication_action_completed(
    replication: &mut ReplicationState,
    feedback: &mut FeedbackState,
    action: String,
    name: String,
) {
    feedback.set_runtime_message(format!("Replication {action} `{name}` ok"));
    replication.replication_drop_publication = None;
    replication.replication_drop_subscription = None;
    replication.replication_ddl_preview = None;
}

pub(super) fn on_event_trigger_action_completed(
    event_trigger: &mut EventTriggerState,
    feedback: &mut FeedbackState,
    action: String,
    name: String,
) {
    feedback.set_runtime_message(format!("Event trigger {action} `{name}` ok"));
    event_trigger.event_trigger_drop_confirm = None;
    event_trigger.event_trigger_ddl_preview = None;
}

pub(super) fn on_monitoring_action_completed(
    monitoring: &mut MonitoringState,
    feedback: &mut FeedbackState,
    action: String,
    backend_id: i64,
    succeeded: bool,
) {
    feedback.set_runtime_message(format!(
        "Monitor {action} pid={backend_id} · {}",
        if succeeded { "ok" } else { "no-op" }
    ));
    monitoring.monitoring_terminate_confirm = None;
    monitoring.monitoring_reset_stats_confirm = false;
}
