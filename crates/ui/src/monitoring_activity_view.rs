use super::*;

impl DbProApp {
    pub(super) fn draw_monitor_activity(&mut self, ui: &mut egui::Ui) {
        section_label(ui, "MONITOR", self.theme);
        ui.add_space(SPACE_SM);

        let connected =
            self.connection.lifecycle.is_connected() && self.connection.lifecycle.active_connection_id().is_some();
        let driver = self.active_driver().to_owned();
        let name = self.active_connection_name().to_owned();
        let header_actions = monitoring_header_view::MonitoringHeaderContext {
            theme: self.theme,
            connected,
            driver: &driver,
            connection_name: &name,
            poll: &mut self.management.monitoring.monitoring_poll,
        }
        .draw(ui);
        if header_actions
            .into_iter()
            .any(|action| matches!(action, monitoring_header_view::MonitoringHeaderAction::Refresh))
        {
            self.request_monitoring_snapshot();
        }

        if connected && self.management.monitoring.monitoring_poll {
            let due = self
                .management
                .monitoring
                .monitoring_last_poll
                .map(|t| t.elapsed() >= std::time::Duration::from_secs(5))
                .unwrap_or(true);
            if due {
                self.request_monitoring_snapshot();
            }
        }

        ui.add_space(SPACE_MD);
        if let Some(error) = &self.management.monitoring.monitoring_error {
            ui.colored_label(self.theme.warning, error);
            ui.add_space(SPACE_SM);
        }

        self.draw_monitor_snapshot(ui, connected);

        let confirmation_actions = monitoring_confirmation_view::MonitoringConfirmationContext {
            theme: self.theme,
            terminate_backend_id: self.management.monitoring.monitoring_terminate_confirm,
            maintenance_action: self.management.monitoring.monitoring_maintenance_confirm,
            reset_statistics: self.management.monitoring.monitoring_reset_stats_confirm,
        }
        .draw(ui.ctx());
        self.apply_monitoring_confirmation_actions(confirmation_actions);
    }

    fn draw_monitor_snapshot(&mut self, ui: &mut egui::Ui, connected: bool) {
        if let Some(snapshot) = self.management.monitoring.monitoring_snapshot.clone() {
            self.draw_monitor_health_and_local(ui, &snapshot);
            let session_actions = monitoring_sessions_view::MonitoringSessionsContext {
                theme: self.theme,
                snapshot: &snapshot,
                filter_active_only: &mut self.management.monitoring.monitoring_filter_active_only,
            }
            .draw(ui);
            self.apply_monitoring_sessions_actions(session_actions);
            self.draw_monitor_server_stats(ui, &snapshot);
            let workload_actions = monitoring_workload_view::MonitoringWorkloadContext {
                theme: self.theme,
                workload: snapshot.workload.as_ref(),
                previous: self.management.monitoring.monitoring_workload_prev.as_ref(),
                sort: self.management.monitoring.monitoring_stat_sort,
                filter: &mut self.management.monitoring.monitoring_workload_filter,
            }
            .draw(ui);
            self.apply_monitoring_workload_actions(workload_actions);

            self.draw_audit_activity(ui);

            self.draw_fdw_activity(ui);

            self.draw_replication_activity(ui);

            self.draw_event_trigger_activity(ui);

            self.draw_pg_settings_activity(ui);

            if let Some(action) = maintenance_activity_view::draw_maintenance_activity(ui, self.theme) {
                self.management.monitoring.monitoring_maintenance_confirm = Some(action);
            }
        } else if connected {
            ui.label(
                RichText::new("Refresh to load sessions (or wait for auto-refresh).")
                    .small()
                    .color(self.theme.text_muted),
            );
        }
    }
    fn draw_monitor_health_and_local(
        &mut self,
        ui: &mut egui::Ui,
        snapshot: &db_pro_core::domain::monitoring::MonitoringSnapshot,
    ) {
        monitoring_snapshot_view::draw_health_and_local(ui, self.theme, snapshot);
    }

    fn draw_monitor_server_stats(
        &mut self,
        ui: &mut egui::Ui,
        snapshot: &db_pro_core::domain::monitoring::MonitoringSnapshot,
    ) {
        monitoring_snapshot_view::draw_server_stats(ui, self.theme, snapshot);
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
        let request_id = self.task_bridge.next_request_id();
        self.dispatch_command(self.management.monitoring.terminate_backend_command(
            request_id,
            connection_id,
            backend_id,
        ));
    }

    fn run_monitoring_maintenance(&mut self, action: db_pro_core::domain::monitoring::MaintenanceAction) {
        let Some(connection_id) = self.connection.lifecycle.active_connection_id().map(str::to_owned) else {
            return;
        };
        let request_id = self.task_bridge.next_request_id();
        self.dispatch_command(
            self.management
                .monitoring
                .maintenance_command(request_id, connection_id, action),
        );
    }

    fn reset_monitoring_statistics(&mut self) {
        let Some(connection_id) = self.connection.lifecycle.active_connection_id().map(str::to_owned) else {
            return;
        };
        let request_id = self.task_bridge.next_request_id();
        self.dispatch_command(
            self.management
                .monitoring
                .reset_statements_command(request_id, connection_id),
        );
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
        let request_id = self.task_bridge.next_request_id();
        self.dispatch_command(
            self.management
                .monitoring
                .cancel_backend_command(request_id, connection_id, backend_id),
        );
    }

    fn request_monitoring_workload(&mut self) {
        let Some(connection_id) = self.connection.lifecycle.active_connection_id().map(str::to_owned) else {
            return;
        };
        let request_id = self.task_bridge.next_request_id();
        self.dispatch_command(self.management.monitoring.workload_command(request_id, connection_id));
    }

    fn request_monitoring_snapshot(&mut self) {
        let Some(connection_id) = self.connection.lifecycle.active_connection_id().map(str::to_owned) else {
            return;
        };
        let request_id = self.task_bridge.next_request_id();
        if self.dispatch_command(self.management.monitoring.snapshot_command(request_id, connection_id)) {
            self.management.monitoring.monitoring_last_poll = Some(std::time::Instant::now());
        }
    }
}
