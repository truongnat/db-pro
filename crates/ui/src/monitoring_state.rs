//! State and command planning for the database monitoring surface.

use super::{RequestId, UiCommand};

pub(super) struct MonitoringState {
    pub(super) monitoring_snapshot: Option<db_pro_core::domain::monitoring::MonitoringSnapshot>,
    pub(super) monitoring_error: Option<String>,
    pub(super) monitoring_poll: bool,
    pub(super) monitoring_last_poll: Option<std::time::Instant>,
    pub(super) monitoring_terminate_confirm: Option<i64>,
    pub(super) monitoring_filter_active_only: bool,
    pub(super) monitoring_maintenance_confirm: Option<db_pro_core::domain::monitoring::MaintenanceAction>,
    pub(super) monitoring_stat_sort: db_pro_core::domain::monitoring::StatStatementSort,
    pub(super) monitoring_reset_stats_confirm: bool,
    pub(super) monitoring_workload_prev: Option<db_pro_core::domain::monitoring::StatStatementsSnapshot>,
    pub(super) monitoring_workload_filter: String,
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

impl MonitoringState {
    pub(super) fn snapshot_command(&self, request_id: RequestId, connection_id: String) -> UiCommand {
        UiCommand::MonitoringSnapshot {
            request_id,
            connection_id,
        }
    }

    pub(super) fn workload_command(&self, request_id: RequestId, connection_id: String) -> UiCommand {
        UiCommand::MonitoringStatStatements {
            request_id,
            connection_id,
            sort: self.monitoring_stat_sort,
            limit: 100,
        }
    }

    pub(super) fn cancel_backend_command(
        &self,
        request_id: RequestId,
        connection_id: String,
        backend_id: i64,
    ) -> UiCommand {
        UiCommand::MonitoringCancelBackend {
            request_id,
            connection_id,
            backend_id,
        }
    }

    pub(super) fn terminate_backend_command(
        &self,
        request_id: RequestId,
        connection_id: String,
        backend_id: i64,
    ) -> UiCommand {
        UiCommand::MonitoringTerminateBackend {
            request_id,
            connection_id,
            backend_id,
        }
    }

    pub(super) fn maintenance_command(
        &self,
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

    pub(super) fn reset_statements_command(&self, request_id: RequestId, connection_id: String) -> UiCommand {
        UiCommand::MonitoringResetStatStatements {
            request_id,
            connection_id,
            confirmed: true,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{MonitoringState, RequestId, UiCommand};

    #[test]
    fn defaults_keep_monitoring_safe_and_bounded() {
        let state = MonitoringState::default();

        assert!(state.monitoring_poll);
        assert!(state.monitoring_filter_active_only);
        assert_eq!(
            state.monitoring_stat_sort,
            db_pro_core::domain::monitoring::StatStatementSort::TotalTime
        );
        assert!(state.monitoring_snapshot.is_none());
    }

    #[test]
    fn workload_command_uses_state_sort_and_bounded_limit() {
        let state = MonitoringState {
            monitoring_stat_sort: db_pro_core::domain::monitoring::StatStatementSort::Calls,
            ..MonitoringState::default()
        };

        assert!(matches!(
            state.workload_command(RequestId(7), "source".to_owned()),
            UiCommand::MonitoringStatStatements {
                request_id: RequestId(7),
                connection_id,
                sort: db_pro_core::domain::monitoring::StatStatementSort::Calls,
                limit: 100,
            } if connection_id == "source"
        ));
    }

    #[test]
    fn destructive_commands_are_explicitly_confirmed() {
        let state = MonitoringState::default();
        let action = db_pro_core::domain::monitoring::MaintenanceAction::Vacuum;

        assert!(matches!(
            state.maintenance_command(RequestId(8), "source".to_owned(), action),
            UiCommand::MonitoringMaintenance {
                request_id: RequestId(8),
                connection_id,
                action: db_pro_core::domain::monitoring::MaintenanceAction::Vacuum,
                confirmed: true,
                ..
            } if connection_id == "source"
        ));
        assert!(matches!(
            state.reset_statements_command(RequestId(9), "source".to_owned()),
            UiCommand::MonitoringResetStatStatements {
                request_id: RequestId(9),
                connection_id,
                confirmed: true,
            } if connection_id == "source"
        ));
    }
}
