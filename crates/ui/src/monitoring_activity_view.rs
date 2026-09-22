use super::*;
use super::{monitoring_state::MonitoringState, RequestId, UiCommand};

impl DbProApp {
    pub(super) fn draw_monitor_activity(&mut self, ui: &mut egui::Ui) {
        section_label(ui, "MONITOR", self.theme);
        ui.add_space(SPACE_SM);

        let connected =
            self.connection.lifecycle.is_connected() && self.connection.lifecycle.active_connection_id().is_some();
        let driver = self.active_driver().to_owned();
        let name = self.active_connection_name().to_owned();
        let actions = monitoring_surface_view::MonitoringSurfaceContext {
            theme: self.theme,
            connected,
            driver: &driver,
            connection_name: &name,
            state: &mut self.management.monitoring,
        }
        .draw(ui);
        for action in actions {
            match action {
                monitoring_surface_view::MonitoringSurfaceAction::Refresh => {
                    self.request_monitoring_snapshot();
                }
                monitoring_surface_view::MonitoringSurfaceAction::Sessions(action) => {
                    self.apply_monitoring_sessions_actions(vec![action]);
                }
                monitoring_surface_view::MonitoringSurfaceAction::Workload(action) => {
                    self.apply_monitoring_workload_actions(vec![action]);
                }
            }
        }

        self.request_monitoring_poll_if_due(connected);

        if self.management.monitoring.monitoring_snapshot.is_some() {
            self.draw_monitor_auxiliary_surfaces(ui);
        }

        let confirmation_actions = monitoring_confirmation_view::MonitoringConfirmationContext {
            theme: self.theme,
            terminate_backend_id: self.management.monitoring.monitoring_terminate_confirm,
            maintenance_action: self.management.monitoring.monitoring_maintenance_confirm,
            reset_statistics: self.management.monitoring.monitoring_reset_stats_confirm,
        }
        .draw(ui.ctx());
        self.apply_monitoring_confirmation_actions(confirmation_actions);
    }

    fn request_monitoring_poll_if_due(&mut self, connected: bool) {
        if !connected || !self.management.monitoring.monitoring_poll {
            return;
        }
        let due = self
            .management
            .monitoring
            .monitoring_last_poll
            .map(|time| time.elapsed() >= std::time::Duration::from_secs(5))
            .unwrap_or(true);
        if due {
            self.request_monitoring_snapshot();
        }
    }

    fn draw_monitor_auxiliary_surfaces(&mut self, ui: &mut egui::Ui) {
        self.draw_audit_activity(ui);
        self.draw_fdw_activity(ui);
        self.draw_replication_activity(ui);
        self.draw_event_trigger_activity(ui);
        self.draw_pg_settings_activity(ui);
        if let Some(action) = maintenance_activity_view::draw_maintenance_activity(ui, self.theme) {
            self.management.monitoring.monitoring_maintenance_confirm = Some(action);
        }
    }

    fn apply_monitoring_confirmation_actions(
        &mut self,
        actions: Vec<monitoring_confirmation_view::MonitoringConfirmationAction>,
    ) {
        for action in actions {
            match action {
                monitoring_confirmation_view::MonitoringConfirmationAction::ConfirmTerminate(backend_id) => {
                    self.terminate_monitoring_backend(backend_id);
                    self.management.monitoring.monitoring_terminate_confirm = None;
                }
                monitoring_confirmation_view::MonitoringConfirmationAction::CancelTerminate => {
                    self.management.monitoring.monitoring_terminate_confirm = None;
                }
                monitoring_confirmation_view::MonitoringConfirmationAction::ConfirmMaintenance(action) => {
                    self.run_monitoring_maintenance(action);
                    self.management.monitoring.monitoring_maintenance_confirm = None;
                }
                monitoring_confirmation_view::MonitoringConfirmationAction::CancelMaintenance => {
                    self.management.monitoring.monitoring_maintenance_confirm = None;
                }
                monitoring_confirmation_view::MonitoringConfirmationAction::ConfirmResetStatistics => {
                    self.reset_monitoring_statistics();
                    self.management.monitoring.monitoring_reset_stats_confirm = false;
                }
                monitoring_confirmation_view::MonitoringConfirmationAction::CancelResetStatistics => {
                    self.management.monitoring.monitoring_reset_stats_confirm = false;
                }
            }
        }
    }

    fn terminate_monitoring_backend(&mut self, backend_id: i64) {
        let Some(connection_id) = self.connection.lifecycle.active_connection_id().map(str::to_owned) else {
            return;
        };
        let request_id = self.next_request_id();
        self.dispatch_command(terminate_backend_command(
            request_id,
            connection_id,
            backend_id,
        ));
    }

    fn run_monitoring_maintenance(&mut self, action: db_pro_core::domain::monitoring::MaintenanceAction) {
        let Some(connection_id) = self.connection.lifecycle.active_connection_id().map(str::to_owned) else {
            return;
        };
        let request_id = self.next_request_id();
        self.dispatch_command(maintenance_command(request_id, connection_id, action));
    }

    fn reset_monitoring_statistics(&mut self) {
        let Some(connection_id) = self.connection.lifecycle.active_connection_id().map(str::to_owned) else {
            return;
        };
        let request_id = self.next_request_id();
        self.dispatch_command(reset_statements_command(request_id, connection_id));
    }

    fn apply_monitoring_sessions_actions(&mut self, actions: Vec<monitoring_sessions_view::MonitoringSessionsAction>) {
        for action in actions {
            match action {
                monitoring_sessions_view::MonitoringSessionsAction::OpenSql(query) => {
                    self.set_active_query_text(query);
                    self.workspace.active_tab = WorkspaceTab::Query;
                    self.workspace.activity = Activity::Explorer;
                }
                monitoring_sessions_view::MonitoringSessionsAction::Cancel(backend_id) => {
                    self.cancel_monitoring_backend(backend_id);
                }
                monitoring_sessions_view::MonitoringSessionsAction::RequestTerminate(backend_id) => {
                    self.management.monitoring.monitoring_terminate_confirm = Some(backend_id);
                }
            }
        }
    }

    fn apply_monitoring_workload_actions(&mut self, actions: Vec<monitoring_workload_view::MonitoringWorkloadAction>) {
        for action in actions {
            match action {
                monitoring_workload_view::MonitoringWorkloadAction::SortChanged(sort) => {
                    self.management.monitoring.monitoring_stat_sort = sort;
                    self.request_monitoring_workload();
                }
                monitoring_workload_view::MonitoringWorkloadAction::ResetStatistics => {
                    self.management.monitoring.monitoring_reset_stats_confirm = true;
                }
                monitoring_workload_view::MonitoringWorkloadAction::OpenSql(query) => {
                    self.set_active_query_text(query);
                    self.workspace.active_tab = WorkspaceTab::Query;
                    self.workspace.activity = Activity::Explorer;
                }
            }
        }
    }

    fn cancel_monitoring_backend(&mut self, backend_id: i64) {
        let Some(connection_id) = self.connection.lifecycle.active_connection_id().map(str::to_owned) else {
            return;
        };
        let request_id = self.next_request_id();
        self.dispatch_command(cancel_backend_command(request_id, connection_id, backend_id));
    }

    fn request_monitoring_workload(&mut self) {
        let Some(connection_id) = self.connection.lifecycle.active_connection_id().map(str::to_owned) else {
            return;
        };
        let request_id = self.next_request_id();
        self.dispatch_command(workload_command(
            &self.management.monitoring,
            request_id,
            connection_id,
        ));
    }

    fn request_monitoring_snapshot(&mut self) {
        let Some(connection_id) = self.connection.lifecycle.active_connection_id().map(str::to_owned) else {
            return;
        };
        let request_id = self.next_request_id();
        if self.dispatch_command(snapshot_command(request_id, connection_id)) {
            self.management.monitoring.monitoring_last_poll = Some(std::time::Instant::now());
        }
    }
}

pub(super) fn snapshot_command(request_id: RequestId, connection_id: String) -> UiCommand {
    UiCommand::MonitoringSnapshot {
        request_id,
        connection_id,
    }
}

fn workload_command(state: &MonitoringState, request_id: RequestId, connection_id: String) -> UiCommand {
    UiCommand::MonitoringStatStatements {
        request_id,
        connection_id,
        sort: state.monitoring_stat_sort,
        limit: 100,
    }
}

fn cancel_backend_command(request_id: RequestId, connection_id: String, backend_id: i64) -> UiCommand {
    UiCommand::MonitoringCancelBackend {
        request_id,
        connection_id,
        backend_id,
    }
}

fn terminate_backend_command(request_id: RequestId, connection_id: String, backend_id: i64) -> UiCommand {
    UiCommand::MonitoringTerminateBackend {
        request_id,
        connection_id,
        backend_id,
    }
}

fn maintenance_command(
    request_id: RequestId,
    connection_id: String,
    action: db_pro_core::domain::monitoring::MaintenanceAction,
) -> UiCommand {
    UiCommand::MonitoringMaintenance {
        request_id,
        connection_id,
        schema: None,
        table: None,
        action,
        confirmed: true,
    }
}

fn reset_statements_command(request_id: RequestId, connection_id: String) -> UiCommand {
    UiCommand::MonitoringResetStatStatements {
        request_id,
        connection_id,
        confirmed: true,
    }
}

#[cfg(test)]
mod tests {
    use super::{maintenance_command, workload_command, MonitoringState, RequestId, UiCommand};

    #[test]
    fn workload_command_uses_state_sort_and_bounded_limit() {
        let state = MonitoringState {
            monitoring_stat_sort: db_pro_core::domain::monitoring::StatStatementSort::Calls,
            ..MonitoringState::default()
        };

        assert!(matches!(
            workload_command(&state, RequestId(7), "source".to_owned()),
            UiCommand::MonitoringStatStatements {
                request_id: RequestId(7),
                connection_id,
                sort: db_pro_core::domain::monitoring::StatStatementSort::Calls,
                limit: 100,
            } if connection_id == "source"
        ));
    }

    #[test]
    fn maintenance_command_is_explicitly_confirmed() {
        let action = db_pro_core::domain::monitoring::MaintenanceAction::Vacuum;

        assert!(matches!(
            maintenance_command(RequestId(8), "source".to_owned(), action),
            UiCommand::MonitoringMaintenance {
                request_id: RequestId(8),
                connection_id,
                action: db_pro_core::domain::monitoring::MaintenanceAction::Vacuum,
                confirmed: true,
                ..
            } if connection_id == "source"
        ));
    }
}
