// cc-scan:allow-file HUGE_FUNCTION,LONG_FUNCTION,HARD_COMPLEXITY,COMPLEXITY,DEEP_NESTING
// cc-scan:allow-file HARD_PARAMS,TOO_MANY_PARAMS,DUPLICATE_BLOCK
// egui painter/view file: fns are linear layout code; branches are per-state paint variants.
//! Runtime event dispatch table for the native UI.
//!
//! Feature handlers remain in their feature modules; this module only maps
//! runtime events to the owning transition entry point.

use super::*;

impl DbProApp {
    pub(crate) fn apply_runtime_event(&mut self, event: UiEvent) {
        match event {
            UiEvent::ConnectionsLoaded { connections, .. } => self.on_connections_loaded(connections),
            UiEvent::SavedQueriesLoaded { queries, .. } => self.on_saved_queries_loaded(queries),
            UiEvent::QueryFoldersLoaded { folders, .. } => self.on_query_folders_loaded(folders),
            UiEvent::SchemaLoaded { request_id, schema } => self.on_schema_loaded(request_id, schema),
            UiEvent::AgentProviderReady { provider, detail } => self.on_agent_provider_ready(provider, detail),
            UiEvent::AgentFailed { request_id, message } => self.on_agent_failed(request_id, message),
            UiEvent::AgentWorkflow { event, .. } => self.on_agent_workflow_event(event),
            UiEvent::AgentConfigured {
                request_id,
                provider,
                detail,
            } => self.on_agent_configured(request_id, provider, detail),
            UiEvent::AgentForgotten { request_id } => self.on_agent_forgotten(request_id),
            UiEvent::TableInfoLoaded { request_id, table_info } => self.on_table_info_loaded(request_id, table_info),
            UiEvent::TableDdlLoaded { request_id, sql } => self.on_table_ddl_loaded(request_id, sql),
            UiEvent::TableDataLoaded {
                request_id,
                result,
                total_rows,
            } => self.on_table_data_loaded(request_id, result, total_rows),
            UiEvent::FilePicked { kind, path, .. } => self.on_file_picked(&kind, path),
            UiEvent::OperationProgress { operation, status, .. } => self.on_operation_progress(operation, status),
            UiEvent::BackupCompleted {
                request_id,
                output_path,
                size_bytes,
            } => self.on_backup_completed(request_id, output_path, size_bytes),
            UiEvent::MonitoringSnapshotLoaded {
                request_id,
                connection_id,
                snapshot,
            } => self.on_monitoring_snapshot_loaded(request_id, connection_id, snapshot),
            UiEvent::MonitoringWorkloadLoaded {
                request_id,
                connection_id,
                workload,
            } => self.on_monitoring_workload_loaded(request_id, connection_id, workload),
            UiEvent::AuditPageLoaded { request_id, connection_id, page } => {
                self.on_audit_page_loaded(request_id, connection_id, page)
            }
            UiEvent::PgSettingsLoaded { request_id, connection_id, snapshot } => {
                self.on_pg_settings_loaded(request_id, connection_id, snapshot)
            }
            UiEvent::PgSettingActionCompleted { action, name, .. } => self.on_pg_setting_action_completed(action, name),
            UiEvent::FdwInventoryLoaded { request_id, connection_id, inventory } => {
                self.on_fdw_inventory_loaded(request_id, connection_id, inventory)
            }
            UiEvent::FdwActionCompleted { action, name, .. } => self.on_fdw_action_completed(action, name),
            UiEvent::ReplicationInventoryLoaded { request_id, connection_id, inventory } => {
                self.on_replication_inventory_loaded(request_id, connection_id, inventory)
            }
            UiEvent::ReplicationActionCompleted { action, name, .. } => {
                self.on_replication_action_completed(action, name)
            }
            UiEvent::EventTriggerInventoryLoaded { request_id, connection_id, inventory } => {
                self.on_event_trigger_inventory_loaded(request_id, connection_id, inventory)
            }
            UiEvent::EventTriggerActionCompleted { action, name, .. } => {
                self.on_event_trigger_action_completed(action, name)
            }
            UiEvent::MonitoringActionCompleted {
                request_id,
                action,
                backend_id,
                succeeded,
            } => self.on_monitoring_action_completed(request_id, action, backend_id, succeeded),
            UiEvent::UsersLoaded {
                request_id,
                connection_id,
                users,
            } => self.on_users_loaded(request_id, connection_id, users),
            UiEvent::PrivilegesLoaded {
                request_id,
                connection_id,
                role_name,
                privileges,
            } => self.on_privileges_loaded(request_id, connection_id, role_name, privileges),
            UiEvent::MembershipsLoaded {
                request_id,
                connection_id,
                member,
                memberships,
            } => self.on_memberships_loaded(request_id, connection_id, member, memberships),
            UiEvent::TableRlsLoaded { request_id, connection_id, state } => {
                self.on_table_rls_loaded(request_id, connection_id, state)
            }
            UiEvent::DataDiffLoaded { request_id, source_id, target_id, diff } => {
                self.on_data_diff_loaded(request_id, source_id, target_id, diff)
            }
            UiEvent::DdlCompleted {
                request_id,
                affected_rows,
            } => self.on_ddl_completed(request_id, affected_rows),
            UiEvent::OperationCompleted { request_id, operation } => self.on_operation_completed(request_id, operation),
            UiEvent::TableChangesFailed {
                request_id,
                code,
                message,
                statement_index,
                rolled_back,
            } => self.on_table_changes_failed(request_id, code, message, statement_index, rolled_back),
            UiEvent::Connected {
                request_id,
                connection_id,
            } => self.on_connected(request_id, connection_id),
            UiEvent::QueryQueued { request_id } => self.on_query_queued(request_id),
            UiEvent::QueryCompleted { request_id, result } => self.on_query_completed(request_id, result),
            UiEvent::QueryMultiCompleted { request_id, output } => self.on_query_multi_completed(request_id, output),
            UiEvent::QuerySaved { request_id, query } => self.on_query_saved(request_id, query),
            UiEvent::ExplainCompleted { request_id, plan } => self.on_explain_completed(request_id, plan),
            UiEvent::QueryCancelled { request_id } => self.on_query_cancelled(request_id),
            UiEvent::QueryFailed { request_id, message } => self.on_query_failed(request_id, message, None, None),
            UiEvent::QueryFailedDetailed {
                request_id,
                code,
                message,
                position,
            } => self.on_query_failed(request_id, message, position, Some(code)),
            UiEvent::SqlPredictionReady {
                request_id,
                document_id,
                document_version,
                anchor,
                replacement_range,
                prediction,
            } => self.on_sql_prediction_ready(
                request_id,
                document_id,
                document_version,
                anchor,
                replacement_range,
                prediction,
            ),
            UiEvent::SqlPredictionFailed {
                request_id,
                document_id,
                ..
            } => self.on_sql_prediction_failed(request_id, document_id),
        }
    }
}
