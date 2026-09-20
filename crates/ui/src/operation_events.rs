//! Runtime orchestration for database-management and mutation events.

use super::*;
use crate::RequestId;

impl DbProApp {
    pub(super) fn on_operation_progress(&mut self, operation: String, status: String) {
        management_events::on_operation_progress(&mut self.feedback, operation, status);
    }

    pub(super) fn on_backup_completed(&mut self, output_path: String, size_bytes: u64) {
        management_events::on_backup_completed(&mut self.feedback, output_path, size_bytes);
    }

    pub(super) fn on_monitoring_snapshot_loaded(
        &mut self,
        snapshot: db_pro_core::domain::monitoring::MonitoringSnapshot,
    ) {
        management_events::on_monitoring_snapshot_loaded(&mut self.monitoring, &mut self.feedback, snapshot);
    }

    pub(super) fn on_monitoring_workload_loaded(
        &mut self,
        workload: db_pro_core::domain::monitoring::StatStatementsSnapshot,
    ) {
        management_events::on_monitoring_workload_loaded(&mut self.monitoring, &mut self.feedback, workload);
    }

    pub(super) fn on_audit_page_loaded(&mut self, page: db_pro_core::domain::audit::AuditPage) {
        management_events::on_audit_page_loaded(
            &mut self.audit.audit_page,
            &mut self.audit.audit_error,
            &mut self.feedback,
            page,
        );
    }

    pub(super) fn on_pg_settings_loaded(&mut self, snapshot: db_pro_core::domain::pg_settings::PgSettingsSnapshot) {
        management_events::on_pg_settings_loaded(&mut self.pg_settings, &mut self.feedback, snapshot);
    }

    pub(super) fn on_pg_setting_action_completed(&mut self, action: String, name: String) {
        management_events::on_pg_setting_action_completed(&mut self.feedback, action, name);
        if let Some(connection_id) = self.connection.lifecycle.active_connection_id().map(str::to_owned) {
            let request_id = self.task_bridge.next_request_id();
            self.dispatch_command(self.pg_settings.list_command(request_id, connection_id));
        }
    }

    pub(super) fn on_fdw_inventory_loaded(&mut self, inventory: db_pro_core::domain::fdw::FdwInventory) {
        management_events::on_fdw_inventory_loaded(&mut self.fdw, &mut self.feedback, inventory);
    }

    pub(super) fn on_fdw_action_completed(&mut self, action: String, name: String) {
        management_events::on_fdw_action_completed(&mut self.fdw, &mut self.feedback, action, name);
        if let Some(connection_id) = self.connection.lifecycle.active_connection_id().map(str::to_owned) {
            let request_id = self.task_bridge.next_request_id();
            self.dispatch_command(self.fdw.list_command(request_id, connection_id));
        }
    }

    pub(super) fn on_replication_inventory_loaded(
        &mut self,
        inventory: db_pro_core::domain::replication::ReplicationInventory,
    ) {
        management_events::on_replication_inventory_loaded(&mut self.replication, &mut self.feedback, inventory);
    }

    pub(super) fn on_replication_action_completed(&mut self, action: String, name: String) {
        management_events::on_replication_action_completed(&mut self.replication, &mut self.feedback, action, name);
        if let Some(connection_id) = self.connection.lifecycle.active_connection_id().map(str::to_owned) {
            let request_id = self.task_bridge.next_request_id();
            self.dispatch_command(self.replication.list_command(request_id, connection_id));
        }
    }

    pub(super) fn on_event_trigger_inventory_loaded(
        &mut self,
        inventory: db_pro_core::domain::event_trigger::EventTriggerInventory,
    ) {
        management_events::on_event_trigger_inventory_loaded(&mut self.event_trigger, &mut self.feedback, inventory);
    }

    pub(super) fn on_event_trigger_action_completed(&mut self, action: String, name: String) {
        management_events::on_event_trigger_action_completed(&mut self.event_trigger, &mut self.feedback, action, name);
        if let Some(connection_id) = self.connection.lifecycle.active_connection_id().map(str::to_owned) {
            let request_id = self.task_bridge.next_request_id();
            self.dispatch_command(self.event_trigger.list_command(request_id, connection_id));
        }
    }

    pub(super) fn on_monitoring_action_completed(&mut self, action: String, backend_id: i64, succeeded: bool) {
        management_events::on_monitoring_action_completed(
            &mut self.monitoring,
            &mut self.feedback,
            action,
            backend_id,
            succeeded,
        );
        if let Some(connection_id) = self.connection.lifecycle.active_connection_id().map(str::to_owned) {
            let request_id = self.task_bridge.next_request_id();
            self.dispatch_command(self.monitoring.snapshot_command(request_id, connection_id));
        }
    }

    pub(super) fn on_users_loaded(&mut self, users: Vec<db_pro_core::domain::user::DatabaseUser>) {
        management_events::on_users_loaded(&mut self.security, &mut self.feedback, users);
    }

    pub(super) fn on_privileges_loaded(
        &mut self,
        role_name: String,
        privileges: Vec<db_pro_core::domain::user::Privilege>,
    ) {
        management_events::on_privileges_loaded(&mut self.security, role_name, privileges);
    }

    pub(super) fn on_memberships_loaded(
        &mut self,
        member: String,
        memberships: Vec<db_pro_core::domain::user::RoleMembership>,
    ) {
        management_events::on_memberships_loaded(&mut self.security, member, memberships);
    }

    pub(super) fn on_table_rls_loaded(&mut self, state: db_pro_core::domain::rls::TableRlsState) {
        management_events::on_table_rls_loaded(&mut self.security, &mut self.feedback, state);
    }

    pub(super) fn on_data_diff_loaded(&mut self, diff: db_pro_core::domain::cross_connection::DataDiff) {
        management_events::on_data_diff_loaded(&mut self.schema_compare, &mut self.feedback, diff);
    }

    /// A native file picker returned (or was cancelled).
    pub(super) fn on_file_picked(&mut self, kind: &str, path: Option<String>) {
        if let Some(folder) = file_picker_events::on_file_picked(
            &mut self.connection.dialog,
            &mut self.overlay,
            &mut self.feedback,
            kind,
            path,
        ) {
            self.open_workspace_folder(folder);
        }
    }

    /// DDL applied; re-introspect so the tree and table view pick up the change.
    pub(super) fn on_ddl_completed(&mut self, request_id: RequestId, affected_rows: u64) {
        if let Some(transition) = ddl_events::on_ddl_completed(
            &mut self.table.state,
            &mut self.security,
            &mut self.feedback,
            request_id,
            affected_rows,
            self.schema_explorer.selected_table.is_some(),
        ) {
            if transition.refresh_schema {
                if let Some(connection_id) = self.connection.lifecycle.active_connection_id().map(str::to_owned) {
                    self.request_schema_introspection(connection_id, true);
                }
            }
            if transition.refresh_rls {
                self.request_security_rls();
            }
        }
    }

    /// Generic completion for connection, table-row and query operations.
    pub(super) fn on_operation_completed(&mut self, request_id: RequestId, operation: String) {
        let pending_connection_request = self.connection.lifecycle.pending_request() == Some(request_id);
        if matches!(
            operation.as_str(),
            "connection.created" | "connection.updated" | "connection.deleted" | "connection.tested"
        ) && !pending_connection_request
        {
            return;
        }
        self.feedback.set_runtime_message(operation.clone());
        if pending_connection_request {
            self.connection.lifecycle.clear_pending_request();
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
            self.connection.lifecycle.clear_connections_requested();
            self.request_connections_once();
        }
        if operation == "connection.tested" && pending_connection_request {
            self.connection.dialog.clear_error();
            self.refresh_connection_diagnostics(true, "Authentication succeeded");
            if self.connection.dialog.test_draft() == Some(self.connection.dialog.draft()) {
                self.connection.dialog.set_test_valid(true);
                self.feedback.set_runtime_message(
                    self.connection
                        .dialog
                        .diagnostics()
                        .map(|result| result.summary())
                        .unwrap_or_else(|| "Connection test succeeded".to_owned()),
                );
            } else {
                self.connection.dialog.set_test_valid(false);
                self.feedback
                    .set_runtime_message("Connection changed · test again before saving");
            }
        }
        if operation.starts_with("table-row.") || operation == "table-changes.applied" {
            self.on_table_row_operation_completed(request_id);
        }
        if operation == "connection.created" || operation == "connection.updated" {
            self.connection
                .dialog
                .transition(super::connection::state::ConnectionDialogAction::Close);
            self.connection.dialog.set_editing_connection_id(None);
        }
        if operation.starts_with("query") || operation.starts_with("query-folder") {
            self.request_saved_queries_refresh();
        }
        if operation == "connection.deleted" {
            // `pending_connection_id` is the delete target (set by the confirm dialog).
            let deleted_id = self.connection.lifecycle.take_pending_connection_id();
            if let Some(ref id) = deleted_id {
                self.connection.lifecycle.clear_connection_error(id);
            }
            let deleted_was_active = deleted_id
                .as_ref()
                .is_some_and(|id| self.connection.lifecycle.active_connection_id() == Some(id.as_str()));
            // Only tear down the live session when the deleted connection was active.
            // Deleting a sibling must not force a reconnect / schema reload of the open one.
            if deleted_was_active {
                *self.connection.lifecycle.active_connection_id_mut() = None;
                self.connection.lifecycle.set_connected(false);
            }
        }
    }

    /// Row mutation follow-up: finalise a staged apply, or refresh the grid.
    fn on_table_row_operation_completed(&mut self, request_id: RequestId) {
        self.table.editing.data_editing_cell = None;
        self.table.editing.data_edit_value.clear();
        self.table.editing.data_edit_error = None;
        self.table.editing.data_delete_confirmation = false;
        if self.table.mutation.staged_apply_request == Some(request_id) {
            self.staged_apply_completed();
        } else if self.table.mutation.table_mutation_request == Some(request_id) {
            self.table.mutation.table_mutation_request = None;
            self.table.data_query.result = None;
            self.table.data_query.total_rows = None;
            self.table.data_query.error = None;
            self.table.data_query.request = None;
            if self.workspace.active_tab == WorkspaceTab::Table {
                self.request_table_data();
            }
        }
    }

    /// Re-reads saved queries for the active connection.
    fn request_saved_queries_refresh(&mut self) {
        if let Some(connection_id) = self.connection.lifecycle.active_connection_id().map(str::to_owned) {
            let request_id = self.task_bridge.next_request_id();
            self.dispatch_command(self.query.library.list_queries_command(request_id, connection_id));
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
        if self.table.mutation.staged_apply_request == Some(request_id) {
            self.staged_apply_failed(statement_index, &code, &message, rolled_back);
        }
    }
}
