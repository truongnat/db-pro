//! State and command planning for the database monitoring surface.

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

#[cfg(test)]
mod tests {
    use super::MonitoringState;

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

}
