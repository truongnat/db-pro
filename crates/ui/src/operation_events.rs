//! Runtime completion and mutation-failure events.

use super::*;
use crate::RequestId;

impl DbProApp {
    pub(super) fn on_operation_progress(&mut self, operation: String, status: String) {
        self.feedback.runtime_message = format!("{operation}: {status}");
    }

    pub(super) fn on_backup_completed(&mut self, output_path: String, size_bytes: u64) {
        self.feedback.runtime_message = format!("Backup completed · {output_path} · {size_bytes} bytes");
    }

    pub(super) fn on_monitoring_snapshot_loaded(
        &mut self,
        snapshot: db_pro_core::domain::monitoring::MonitoringSnapshot,
    ) {
        if let Some(prev) = self.monitoring.monitoring_snapshot.take() {
            self.monitoring.monitoring_workload_prev = prev.workload;
        }
        self.monitoring.monitoring_snapshot = Some(snapshot.clone());
        self.monitoring.monitoring_error = None;
        self.feedback.runtime_message = format!("Monitor · {}", snapshot.message);
    }

    pub(super) fn on_monitoring_workload_loaded(
        &mut self,
        workload: db_pro_core::domain::monitoring::StatStatementsSnapshot,
    ) {
        if let Some(snapshot) = self.monitoring.monitoring_snapshot.as_mut() {
            self.monitoring.monitoring_workload_prev = snapshot.workload.clone();
            snapshot.workload = Some(workload.clone());
        }
        self.monitoring.monitoring_stat_sort = workload.sort;
        self.feedback.runtime_message = format!("Workload · {}", workload.message);
    }

    pub(super) fn on_audit_page_loaded(&mut self, page: db_pro_core::domain::audit::AuditPage) {
        self.audit.audit_page = Some(page.clone());
        self.audit.audit_error = None;
        self.feedback.runtime_message = format!(
            "Audit · {} event(s) · {}",
            page.events.len(),
            page.source.guidance.chars().take(80).collect::<String>()
        );
    }

    pub(super) fn on_pg_settings_loaded(&mut self, snapshot: db_pro_core::domain::pg_settings::PgSettingsSnapshot) {
        self.pg_settings.pg_settings = Some(snapshot.clone());
        self.pg_settings.pg_settings_error = None;
        self.feedback.runtime_message = format!("pg_settings · {}", snapshot.message);
    }

    pub(super) fn on_pg_setting_action_completed(&mut self, action: String, name: String) {
        self.feedback.runtime_message = format!("pg_settings {action} `{name}` ok");
        if let Some(connection_id) = self.connection_lifecycle.active_connection_id.clone() {
            let request_id = self.task_bridge.next_request_id();
            self.dispatch_command(UiCommand::ListPgSettings {
                request_id,
                connection_id,
            });
        }
    }

    pub(super) fn on_fdw_inventory_loaded(&mut self, inventory: db_pro_core::domain::fdw::FdwInventory) {
        self.fdw.fdw_inventory = Some(inventory.clone());
        self.fdw.fdw_error = None;
        self.feedback.runtime_message = format!("FDW · {}", inventory.message);
    }

    pub(super) fn on_fdw_action_completed(&mut self, action: String, name: String) {
        self.feedback.runtime_message = format!("FDW {action} `{name}` ok");
        self.fdw.fdw_drop_confirm = None;
        self.fdw.fdw_ddl_preview = None;
        if let Some(connection_id) = self.connection_lifecycle.active_connection_id.clone() {
            let request_id = self.task_bridge.next_request_id();
            self.dispatch_command(UiCommand::ListFdwInventory {
                request_id,
                connection_id,
            });
        }
    }

    pub(super) fn on_replication_inventory_loaded(
        &mut self,
        inventory: db_pro_core::domain::replication::ReplicationInventory,
    ) {
        self.replication.replication_inventory = Some(inventory.clone());
        self.replication.replication_error = None;
        self.feedback.runtime_message = format!("Replication · {}", inventory.message);
    }

    pub(super) fn on_replication_action_completed(&mut self, action: String, name: String) {
        self.feedback.runtime_message = format!("Replication {action} `{name}` ok");
        self.replication.replication_drop_publication = None;
        self.replication.replication_drop_subscription = None;
        self.replication.replication_ddl_preview = None;
        if let Some(connection_id) = self.connection_lifecycle.active_connection_id.clone() {
            let request_id = self.task_bridge.next_request_id();
            self.dispatch_command(UiCommand::ListReplicationInventory {
                request_id,
                connection_id,
            });
        }
    }

    pub(super) fn on_event_trigger_inventory_loaded(
        &mut self,
        inventory: db_pro_core::domain::event_trigger::EventTriggerInventory,
    ) {
        self.event_trigger.event_trigger_inventory = Some(inventory.clone());
        self.event_trigger.event_trigger_error = None;
        self.feedback.runtime_message = format!("Event triggers · {}", inventory.message);
    }

    pub(super) fn on_event_trigger_action_completed(&mut self, action: String, name: String) {
        self.feedback.runtime_message = format!("Event trigger {action} `{name}` ok");
        self.event_trigger.event_trigger_drop_confirm = None;
        self.event_trigger.event_trigger_ddl_preview = None;
        if let Some(connection_id) = self.connection_lifecycle.active_connection_id.clone() {
            let request_id = self.task_bridge.next_request_id();
            self.dispatch_command(UiCommand::ListEventTriggers {
                request_id,
                connection_id,
            });
        }
    }

    pub(super) fn on_monitoring_action_completed(&mut self, action: String, backend_id: i64, succeeded: bool) {
        self.feedback.runtime_message = format!(
            "Monitor {action} pid={backend_id} · {}",
            if succeeded { "ok" } else { "no-op" }
        );
        self.monitoring.monitoring_terminate_confirm = None;
        self.monitoring.monitoring_reset_stats_confirm = false;
        if let Some(connection_id) = self.connection_lifecycle.active_connection_id.clone() {
            let request_id = self.task_bridge.next_request_id();
            self.dispatch_command(UiCommand::MonitoringSnapshot {
                request_id,
                connection_id,
            });
        }
    }

    pub(super) fn on_users_loaded(&mut self, users: Vec<db_pro_core::domain::user::DatabaseUser>) {
        self.security.security_users = users;
        self.security.security_error = None;
        self.feedback.runtime_message = format!("Security · {} role(s)", self.security.security_users.len());
    }

    pub(super) fn on_privileges_loaded(
        &mut self,
        role_name: String,
        privileges: Vec<db_pro_core::domain::user::Privilege>,
    ) {
        self.security.security_selected_role = Some(role_name);
        self.security.security_privileges = privileges;
    }

    pub(super) fn on_memberships_loaded(
        &mut self,
        member: String,
        memberships: Vec<db_pro_core::domain::user::RoleMembership>,
    ) {
        self.security.security_selected_role = Some(member);
        self.security.security_memberships = memberships;
    }

    pub(super) fn on_table_rls_loaded(&mut self, state: db_pro_core::domain::rls::TableRlsState) {
        self.security.security_rls_state = Some(state);
        self.security.security_error = None;
        self.feedback.runtime_message = "Security · RLS state loaded".to_owned();
    }

    pub(super) fn on_data_diff_loaded(&mut self, diff: db_pro_core::domain::cross_connection::DataDiff) {
        self.schema_compare.data_diff_result = Some(diff);
        self.feedback.runtime_message = "Data compare ready".to_owned();
    }

    /// A native file picker returned (or was cancelled).
    pub(super) fn on_file_picked(&mut self, kind: &str, path: Option<String>) {
        if let Some(path) = path {
            if kind == "sqlite" {
                self.connection_dialog.draft_mut().database = path;
            } else if kind == "ssh-key" {
                self.connection_dialog.draft_mut().ssh_private_key = path;
            } else if kind == "backup" {
                self.overlay.backup_output_path = path;
            } else if kind == "restore" {
                self.overlay.restore_input_path = path;
            } else if kind == "workspace-folder" {
                self.open_workspace_folder(std::path::PathBuf::from(path));
            }
            self.connection_dialog.clear_error();
            self.connection_dialog.set_test_valid(false);
        } else if kind == "sqlite" || kind == "ssh-key" {
            self.connection_dialog.set_error("File selection was cancelled");
            self.connection_dialog.set_test_valid(false);
        } else if kind == "workspace-folder" {
            self.feedback.runtime_message = "Workspace folder selection was cancelled".to_owned();
        }
    }

    /// DDL applied; re-introspect so the tree and table view pick up the change.
    pub(super) fn on_ddl_completed(&mut self, request_id: RequestId, affected_rows: u64) {
        if self.table_state.ddl_execution_request == Some(request_id) {
            self.table_state.ddl_execution_request = None;
            self.table_state.ddl_execute_confirmation = false;
            self.table_state.table_ddl_error = None;
            self.table_state.refresh_table_info_after_schema = self.schema_explorer.selected_table.is_some();
            self.feedback.runtime_message = format!("DDL applied · {affected_rows} affected rows");
            if let Some(connection_id) = self.connection_lifecycle.active_connection_id.clone() {
                self.request_schema_introspection(connection_id, true);
            }
            if !self.security.security_rls_table.trim().is_empty() {
                self.request_security_rls();
            }
            self.security.security_rls_confirm_apply = false;
        }
    }

    /// Generic completion for connection, table-row and query operations.
    pub(super) fn on_operation_completed(&mut self, request_id: RequestId, operation: String) {
        let pending_connection_request = self.connection_lifecycle.pending_request == Some(request_id);
        if matches!(
            operation.as_str(),
            "connection.created" | "connection.updated" | "connection.deleted" | "connection.tested"
        ) && !pending_connection_request
        {
            return;
        }
        self.feedback.runtime_message = operation.clone();
        if pending_connection_request {
            self.connection_lifecycle.clear_pending_request();
        }
        if matches!(
            operation.as_str(),
            "create_role"
                | "drop_role"
                | "alter_role"
                | "update_role_password"
                | "grant_membership"
                | "revoke_membership"
                | "grant_privilege"
                | "revoke_privilege"
        ) {
            self.security.security_drop_confirm = None;
            self.security.security_password.clear();
            self.request_security_users();
            if let Some(role) = self.security.security_selected_role.clone() {
                self.request_security_role_details(&role);
            }
        }
        if matches!(
            operation.as_str(),
            "connection.created" | "connection.updated" | "connection.deleted"
        ) {
            self.connection_lifecycle.connections_requested = false;
            self.request_connections_once();
        }
        if operation == "connection.tested" && pending_connection_request {
            self.connection_dialog.clear_error();
            self.refresh_connection_diagnostics(true, "Authentication succeeded");
            if self.connection_dialog.test_draft() == Some(self.connection_dialog.draft()) {
                self.connection_dialog.set_test_valid(true);
                self.feedback.runtime_message = self
                    .connection_dialog
                    .diagnostics()
                    .map(|r| r.summary())
                    .unwrap_or_else(|| "Connection test succeeded".to_owned());
            } else {
                self.connection_dialog.set_test_valid(false);
                self.feedback.runtime_message = "Connection changed · test again before saving".to_owned();
            }
        }
        if operation.starts_with("table-row.") || operation == "table-changes.applied" {
            self.on_table_row_operation_completed(request_id);
        }
        if operation == "connection.created" || operation == "connection.updated" {
            self.connection_dialog
                .transition(super::connection::state::ConnectionDialogAction::Close);
            self.connection_dialog.set_editing_connection_id(None);
        }
        if operation.starts_with("query") || operation.starts_with("query-folder") {
            self.request_saved_queries_refresh();
        }
        if operation == "connection.deleted" {
            // `pending_connection_id` is the delete target (set by the confirm dialog).
            let deleted_id = self.connection_lifecycle.pending_connection_id.take();
            if let Some(ref id) = deleted_id {
                self.connection_lifecycle.clear_connection_error(id);
            }
            let deleted_was_active = deleted_id
                .as_ref()
                .is_some_and(|id| self.connection_lifecycle.active_connection_id.as_deref() == Some(id.as_str()));
            // Only tear down the live session when the deleted connection was active.
            // Deleting a sibling must not force a reconnect / schema reload of the open one.
            if deleted_was_active {
                self.connection_lifecycle.active_connection_id = None;
                self.connection_lifecycle.connected = false;
            }
        }
    }

    /// Row mutation follow-up: finalise a staged apply, or refresh the grid.
    fn on_table_row_operation_completed(&mut self, request_id: RequestId) {
        self.table_data.data_editing_cell = None;
        self.table_data.data_edit_value.clear();
        self.table_data.data_edit_error = None;
        self.table_data.data_delete_confirmation = false;
        if self.table_mutation.staged_apply_request == Some(request_id) {
            self.staged_apply_completed();
        } else if self.table_mutation.table_mutation_request == Some(request_id) {
            self.table_mutation.table_mutation_request = None;
            self.table_state.table_data_result = None;
            self.table_state.table_data_total_rows = None;
            self.table_state.table_data_error = None;
            self.table_state.table_data_request = None;
            if self.workspace.active_tab == WorkspaceTab::Table {
                self.request_table_data();
            }
        }
    }

    /// Re-reads saved queries for the active connection.
    fn request_saved_queries_refresh(&mut self) {
        if let Some(connection_id) = self.connection_lifecycle.active_connection_id.clone() {
            let request_id = self.task_bridge.next_request_id();
            self.dispatch_command(UiCommand::ListSavedQueries {
                request_id,
                connection_id,
            });
        }
    }

    pub(super) fn on_table_changes_failed(
        &mut self,
        request_id: RequestId,
        code: String,
        message: String,
        statement_index: usize,
        rolled_back: bool,
    ) {
        if self.table_mutation.staged_apply_request == Some(request_id) {
            self.staged_apply_failed(statement_index, &code, &message, rolled_back);
        }
    }
}
