//! Runtime event dispatch table for the native UI.
//!
//! Feature handlers remain in their feature modules; this module only maps
//! runtime events to the owning transition entry point.

use super::*;

impl DbProApp {
    pub(crate) fn apply_runtime_event(&mut self, event: UiEvent) {
        match event {
            UiEvent::ConnectionsLoaded { connections, .. } => self.on_connections_loaded(connections),
            UiEvent::SavedQueriesLoaded { queries, .. } => self.query_library.saved_queries = queries,
            UiEvent::QueryFoldersLoaded { folders, .. } => self.query_library.query_folders = folders,
            UiEvent::SchemaLoaded { request_id, schema } => self.on_schema_loaded(request_id, schema),
            UiEvent::AgentCompleted {
                request_id,
                provider,
                message,
            } => self.on_agent_completed(request_id, provider, message),
            UiEvent::AgentProviderReady { provider, detail } => {
                self.agent.provider_label = provider;
                self.agent.provider_detail = detail;
            }
            UiEvent::AgentFailed { request_id, message } => self.on_agent_failed(request_id, message),
            UiEvent::AgentToolCompleted { .. } | UiEvent::AgentToolFailed { .. } => {}
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
            UiEvent::OperationProgress { operation, status, .. } => {
                self.feedback.runtime_message = format!("{operation}: {status}");
            }
            UiEvent::BackupCompleted {
                output_path,
                size_bytes,
                ..
            } => {
                self.feedback.runtime_message = format!("Backup completed · {output_path} · {size_bytes} bytes");
            }
            UiEvent::MonitoringSnapshotLoaded { snapshot, .. } => {
                if let Some(prev) = self.database_operations.monitoring_snapshot.take() {
                    self.database_operations.monitoring_workload_prev = prev.workload;
                }
                self.database_operations.monitoring_snapshot = Some(snapshot.clone());
                self.database_operations.monitoring_error = None;
                self.feedback.runtime_message = format!("Monitor · {}", snapshot.message);
            }
            UiEvent::MonitoringWorkloadLoaded { workload, .. } => {
                if let Some(snap) = self.database_operations.monitoring_snapshot.as_mut() {
                    self.database_operations.monitoring_workload_prev = snap.workload.clone();
                    snap.workload = Some(workload.clone());
                }
                self.database_operations.monitoring_stat_sort = workload.sort;
                self.feedback.runtime_message = format!("Workload · {}", workload.message);
            }
            UiEvent::AuditPageLoaded { page, .. } => {
                self.database_operations.audit_page = Some(page.clone());
                self.database_operations.audit_error = None;
                self.feedback.runtime_message = format!(
                    "Audit · {} event(s) · {}",
                    page.events.len(),
                    page.source.guidance.chars().take(80).collect::<String>()
                );
            }
            UiEvent::PgSettingsLoaded { snapshot, .. } => {
                self.database_operations.pg_settings = Some(snapshot.clone());
                self.database_operations.pg_settings_error = None;
                self.feedback.runtime_message = format!("pg_settings · {}", snapshot.message);
            }
            UiEvent::PgSettingActionCompleted { action, name, .. } => {
                self.feedback.runtime_message = format!("pg_settings {action} `{name}` ok");
                if let Some(connection_id) = self.connection_lifecycle.active_connection_id.clone() {
                    let request_id = self.task_bridge.next_request_id();
                    self.dispatch_command(UiCommand::ListPgSettings {
                        request_id,
                        connection_id,
                    });
                }
            }
            UiEvent::FdwInventoryLoaded { inventory, .. } => {
                self.database_operations.fdw_inventory = Some(inventory.clone());
                self.database_operations.fdw_error = None;
                self.feedback.runtime_message = format!("FDW · {}", inventory.message);
            }
            UiEvent::FdwActionCompleted { action, name, .. } => {
                self.feedback.runtime_message = format!("FDW {action} `{name}` ok");
                self.database_operations.fdw_drop_confirm = None;
                self.database_operations.fdw_ddl_preview = None;
                if let Some(connection_id) = self.connection_lifecycle.active_connection_id.clone() {
                    let request_id = self.task_bridge.next_request_id();
                    self.dispatch_command(UiCommand::ListFdwInventory {
                        request_id,
                        connection_id,
                    });
                }
            }
            UiEvent::ReplicationInventoryLoaded { inventory, .. } => {
                self.database_operations.replication_inventory = Some(inventory.clone());
                self.database_operations.replication_error = None;
                self.feedback.runtime_message = format!("Replication · {}", inventory.message);
            }
            UiEvent::ReplicationActionCompleted { action, name, .. } => {
                self.feedback.runtime_message = format!("Replication {action} `{name}` ok");
                self.database_operations.replication_drop_publication = None;
                self.database_operations.replication_drop_subscription = None;
                self.database_operations.replication_ddl_preview = None;
                if let Some(connection_id) = self.connection_lifecycle.active_connection_id.clone() {
                    let request_id = self.task_bridge.next_request_id();
                    self.dispatch_command(UiCommand::ListReplicationInventory {
                        request_id,
                        connection_id,
                    });
                }
            }
            UiEvent::EventTriggerInventoryLoaded { inventory, .. } => {
                self.database_operations.event_trigger_inventory = Some(inventory.clone());
                self.database_operations.event_trigger_error = None;
                self.feedback.runtime_message = format!("Event triggers · {}", inventory.message);
            }
            UiEvent::EventTriggerActionCompleted { action, name, .. } => {
                self.feedback.runtime_message = format!("Event trigger {action} `{name}` ok");
                self.database_operations.event_trigger_drop_confirm = None;
                self.database_operations.event_trigger_ddl_preview = None;
                if let Some(connection_id) = self.connection_lifecycle.active_connection_id.clone() {
                    let request_id = self.task_bridge.next_request_id();
                    self.dispatch_command(UiCommand::ListEventTriggers {
                        request_id,
                        connection_id,
                    });
                }
            }
            UiEvent::MonitoringActionCompleted {
                action,
                backend_id,
                succeeded,
                ..
            } => {
                self.feedback.runtime_message = format!(
                    "Monitor {action} pid={backend_id} · {}",
                    if succeeded { "ok" } else { "no-op" }
                );
                self.database_operations.monitoring_terminate_confirm = None;
                self.database_operations.monitoring_reset_stats_confirm = false;
                if let Some(connection_id) = self.connection_lifecycle.active_connection_id.clone() {
                    let request_id = self.task_bridge.next_request_id();
                    self.dispatch_command(UiCommand::MonitoringSnapshot {
                        request_id,
                        connection_id,
                    });
                }
            }
            UiEvent::UsersLoaded { users, .. } => {
                self.database_operations.security_users = users;
                self.database_operations.security_error = None;
                self.feedback.runtime_message =
                    format!("Security · {} role(s)", self.database_operations.security_users.len());
            }
            UiEvent::PrivilegesLoaded {
                role_name, privileges, ..
            } => {
                self.database_operations.security_selected_role = Some(role_name);
                self.database_operations.security_privileges = privileges;
            }
            UiEvent::MembershipsLoaded {
                member, memberships, ..
            } => {
                self.database_operations.security_selected_role = Some(member);
                self.database_operations.security_memberships = memberships;
            }
            UiEvent::TableRlsLoaded { state, .. } => {
                self.database_operations.security_rls_state = Some(state);
                self.database_operations.security_error = None;
                self.feedback.runtime_message = "Security · RLS state loaded".into();
            }
            UiEvent::DataDiffLoaded { diff, .. } => {
                self.database_operations.data_diff_result = Some(diff);
                self.feedback.runtime_message = "Data compare ready".into();
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
            UiEvent::QueryQueued { request_id } => {
                self.feedback.runtime_message = format!("Query queued · request {}", request_id.0);
            }
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
                document_version,
                anchor,
                replacement_range,
                message,
            } => self.on_sql_prediction_failed(
                request_id,
                document_id,
                document_version,
                anchor,
                replacement_range,
                message,
            ),
        }
    }
}
