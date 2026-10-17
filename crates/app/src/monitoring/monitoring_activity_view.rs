use super::command_dispatch::RuntimeCommandDispatcher;
use super::monitoring_state::MonitoringState;
use super::*;

pub(super) enum MonitoringActivityEffect {
    OpenQuery(String),
}

pub(super) struct MonitoringActivityContext<'a, 'bridge> {
    pub(super) theme: DbProTheme,
    pub(super) state: &'a mut MonitoringState,
    pub(super) connected: bool,
    pub(super) driver: &'a str,
    pub(super) connection_name: &'a str,
    pub(super) connection_id: Option<&'a str>,
    pub(super) command_dispatcher: &'a mut RuntimeCommandDispatcher<'bridge>,
    pub(super) feedback: &'a mut FeedbackState,
}

impl MonitoringActivityContext<'_, '_> {
    pub(super) fn draw(&mut self, ui: &mut egui::Ui) -> Vec<MonitoringActivityEffect> {
        section_label(ui, "MONITOR", self.theme);
        ui.add_space(SPACE_SM);

        let actions = monitoring_surface_view::MonitoringSurfaceContext {
            theme: self.theme,
            connected: self.connected,
            driver: self.driver,
            connection_name: self.connection_name,
            state: self.state,
        }
        .draw(ui);
        let mut effects = Vec::new();
        for action in actions {
            match action {
                monitoring_surface_view::MonitoringSurfaceAction::Refresh => {
                    self.request_monitoring_snapshot();
                }
                monitoring_surface_view::MonitoringSurfaceAction::Sessions(action) => {
                    self.apply_monitoring_sessions_action(*action, &mut effects);
                }
                monitoring_surface_view::MonitoringSurfaceAction::Workload(action) => {
                    self.apply_monitoring_workload_action(action, &mut effects);
                }
            }
        }

        self.request_monitoring_poll_if_due();

        let confirmation_actions = monitoring_confirmation_view::MonitoringConfirmationContext {
            theme: self.theme,
            terminate_session: self.state.monitoring_terminate_confirm.as_ref(),
            cancel_session: self.state.monitoring_cancel_confirm.as_ref(),
            maintenance_action: self.state.monitoring_maintenance_confirm,
            maintenance_target: maintenance_target_label(self.state),
            reset_statistics: self.state.monitoring_reset_stats_confirm,
        }
        .draw(ui.ctx());
        self.apply_monitoring_confirmation_actions(confirmation_actions);
        effects
    }

    fn request_monitoring_poll_if_due(&mut self) {
        if !self.connected || !self.state.monitoring_poll {
            return;
        }
        if monitoring_poll_remaining(self.state.monitoring_last_poll, std::time::Instant::now()).is_zero() {
            self.request_monitoring_snapshot();
        }
    }

    fn apply_monitoring_confirmation_actions(
        &mut self,
        actions: Vec<monitoring_confirmation_view::MonitoringConfirmationAction>,
    ) {
        for action in actions {
            match action {
                monitoring_confirmation_view::MonitoringConfirmationAction::ConfirmTerminate {
                    backend_id,
                    backend_start,
                } => {
                    if self.terminate_monitoring_backend(backend_id, backend_start) {
                        self.state.monitoring_terminate_confirm = None;
                    }
                }
                monitoring_confirmation_view::MonitoringConfirmationAction::CancelTerminate => {
                    self.state.monitoring_terminate_confirm = None;
                }
                monitoring_confirmation_view::MonitoringConfirmationAction::ConfirmCancelBackend {
                    backend_id,
                    backend_start,
                } => {
                    if self.cancel_monitoring_backend(backend_id, backend_start) {
                        self.state.monitoring_cancel_confirm = None;
                    }
                }
                monitoring_confirmation_view::MonitoringConfirmationAction::DismissCancelBackend => {
                    self.state.monitoring_cancel_confirm = None;
                }
                monitoring_confirmation_view::MonitoringConfirmationAction::ConfirmMaintenance(action) => {
                    if self.run_monitoring_maintenance(action) {
                        self.state.monitoring_maintenance_confirm = None;
                    }
                }
                monitoring_confirmation_view::MonitoringConfirmationAction::CancelMaintenance => {
                    self.state.monitoring_maintenance_confirm = None;
                }
                monitoring_confirmation_view::MonitoringConfirmationAction::ConfirmResetStatistics => {
                    if self.reset_monitoring_statistics() {
                        self.state.monitoring_reset_stats_confirm = false;
                    }
                }
                monitoring_confirmation_view::MonitoringConfirmationAction::CancelResetStatistics => {
                    self.state.monitoring_reset_stats_confirm = false;
                }
            }
        }
    }

    fn terminate_monitoring_backend(&mut self, backend_id: i64, backend_start: Option<String>) -> bool {
        let Some(connection_id) = self.connection_id else {
            self.feedback.set_runtime_message("Connect a database first");
            return false;
        };
        let Some(backend_start) = backend_start else {
            // No snapshot timestamp → the pid is ambiguous; never signal a
            // possibly-recycled pid (#501).
            self.feedback
                .set_runtime_message("Cannot terminate: session start time is unknown — refresh and retry");
            return false;
        };
        let request_id = self.command_dispatcher.next_request_id();
        self.dispatch(terminate_backend_command(
            request_id,
            connection_id.to_owned(),
            backend_id,
            backend_start,
        ))
    }

    fn run_monitoring_maintenance(&mut self, action: db_pro_core::domain::monitoring::MaintenanceAction) -> bool {
        let Some(connection_id) = self.connection_id else {
            self.feedback.set_runtime_message("Connect a database first");
            return false;
        };
        let request_id = self.command_dispatcher.next_request_id();
        self.dispatch(maintenance_command(
            self.state,
            request_id,
            connection_id.to_owned(),
            action,
        ))
    }

    fn reset_monitoring_statistics(&mut self) -> bool {
        let Some(connection_id) = self.connection_id else {
            self.feedback.set_runtime_message("Connect a database first");
            return false;
        };
        let request_id = self.command_dispatcher.next_request_id();
        self.dispatch(reset_statements_command(request_id, connection_id.to_owned()))
    }

    fn apply_monitoring_sessions_action(
        &mut self,
        action: monitoring_sessions_view::MonitoringSessionsAction,
        effects: &mut Vec<MonitoringActivityEffect>,
    ) {
        match action {
            monitoring_sessions_view::MonitoringSessionsAction::OpenSql(query) => {
                effects.push(MonitoringActivityEffect::OpenQuery(query));
            }
            monitoring_sessions_view::MonitoringSessionsAction::Cancel(session) => {
                self.state.monitoring_cancel_confirm = Some(session);
            }
            monitoring_sessions_view::MonitoringSessionsAction::RequestTerminate(session) => {
                self.state.monitoring_terminate_confirm = Some(session);
            }
        }
    }

    fn apply_monitoring_workload_action(
        &mut self,
        action: monitoring_workload_view::MonitoringWorkloadAction,
        effects: &mut Vec<MonitoringActivityEffect>,
    ) {
        match action {
            monitoring_workload_view::MonitoringWorkloadAction::SortChanged(sort) => {
                self.state.monitoring_stat_sort = sort;
                self.request_monitoring_workload();
            }
            monitoring_workload_view::MonitoringWorkloadAction::ResetStatistics => {
                self.state.monitoring_reset_stats_confirm = true;
            }
            monitoring_workload_view::MonitoringWorkloadAction::OpenSql(query) => {
                effects.push(MonitoringActivityEffect::OpenQuery(query));
            }
        }
    }

    fn cancel_monitoring_backend(&mut self, backend_id: i64, backend_start: Option<String>) -> bool {
        let Some(connection_id) = self.connection_id else {
            self.feedback.set_runtime_message("Connect a database first");
            return false;
        };
        let Some(backend_start) = backend_start else {
            self.feedback
                .set_runtime_message("Cannot cancel: session start time is unknown — refresh and retry");
            return false;
        };
        let request_id = self.command_dispatcher.next_request_id();
        self.dispatch(cancel_backend_command(
            request_id,
            connection_id.to_owned(),
            backend_id,
            backend_start,
        ))
    }

    fn request_monitoring_workload(&mut self) {
        let Some(connection_id) = self.connection_id else {
            return;
        };
        let request_id = self.command_dispatcher.next_request_id();
        self.dispatch(workload_command(
            self.state,
            request_id,
            connection_id.to_owned(),
        ));
    }

    fn request_monitoring_snapshot(&mut self) {
        let Some(connection_id) = self.connection_id else {
            return;
        };
        let request_id = self.command_dispatcher.next_request_id();
        if self.dispatch(snapshot_command(
            request_id,
            connection_id.to_owned(),
            self.state.monitoring_stat_sort,
        )) {
            self.state.monitoring_last_poll = Some(std::time::Instant::now());
        }
    }

    fn dispatch(&mut self, command: UiCommand) -> bool {
        self.command_dispatcher.dispatch(command, self.feedback)
    }
}

pub(super) fn snapshot_command(
    request_id: RequestId,
    connection_id: String,
    sort: db_pro_core::domain::monitoring::StatStatementSort,
) -> UiCommand {
    UiCommand::MonitoringSnapshot {
        request_id,
        connection_id,
        sort,
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

fn cancel_backend_command(
    request_id: RequestId,
    connection_id: String,
    backend_id: i64,
    expected_backend_start: String,
) -> UiCommand {
    UiCommand::MonitoringCancelBackend {
        request_id,
        connection_id,
        backend_id,
        expected_backend_start,
    }
}

fn terminate_backend_command(
    request_id: RequestId,
    connection_id: String,
    backend_id: i64,
    expected_backend_start: String,
) -> UiCommand {
    UiCommand::MonitoringTerminateBackend {
        request_id,
        connection_id,
        backend_id,
        expected_backend_start,
    }
}

fn maintenance_target(value: &str) -> Option<String> {
    let value = value.trim();
    if value.is_empty() {
        None
    } else {
        Some(value.to_owned())
    }
}

/// Label for the maintenance confirm dialog — the user must see whether the
/// run targets `schema.table` or the entire database (#403).
fn maintenance_target_label(state: &MonitoringState) -> String {
    match (
        maintenance_target(&state.monitoring_maintenance_schema),
        maintenance_target(&state.monitoring_maintenance_table),
    ) {
        (Some(schema), Some(table)) => format!("`{schema}.{table}`"),
        _ => "the entire database".to_owned(),
    }
}

fn maintenance_command(
    state: &MonitoringState,
    request_id: RequestId,
    connection_id: String,
    action: db_pro_core::domain::monitoring::MaintenanceAction,
) -> UiCommand {
    UiCommand::MonitoringMaintenance {
        request_id,
        connection_id,
        schema: maintenance_target(&state.monitoring_maintenance_schema),
        table: maintenance_target(&state.monitoring_maintenance_table),
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

/// Monitoring auto-refresh cadence. The poll only fires while a frame is
/// painted, so `prepare_frame` schedules `request_repaint_after` with exactly
/// this remaining time — without it egui idles forever and the view goes stale
/// (#489).
pub(super) const MONITORING_POLL_INTERVAL: std::time::Duration = std::time::Duration::from_secs(5);

/// Time until the next monitoring poll is due; `ZERO` means poll now.
/// `saturating_duration_since` tolerates a `last_poll` recorded after `now`.
pub(super) fn monitoring_poll_remaining(
    last_poll: Option<std::time::Instant>,
    now: std::time::Instant,
) -> std::time::Duration {
    let Some(last_poll) = last_poll else {
        return std::time::Duration::ZERO;
    };
    MONITORING_POLL_INTERVAL.saturating_sub(now.saturating_duration_since(last_poll))
}

#[cfg(test)]
#[path = "tests/monitoring_activity_view_tests.rs"]
mod tests;

