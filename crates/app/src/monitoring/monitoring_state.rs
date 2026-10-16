//! State and command planning for the database monitoring surface.

pub(super) const MONITORING_POLL_INTERVAL: std::time::Duration = std::time::Duration::from_secs(5);

pub(super) fn monitoring_poll_repaint_delay(
    last_poll: Option<std::time::Instant>,
    now: std::time::Instant,
) -> std::time::Duration {
    let elapsed = last_poll
        .map(|last_poll| now.saturating_duration_since(last_poll))
        .unwrap_or(MONITORING_POLL_INTERVAL);
    MONITORING_POLL_INTERVAL
        .saturating_sub(elapsed)
        .max(std::time::Duration::from_millis(1))
}

pub(super) struct MonitoringState {
    pub(super) monitoring_snapshot: Option<db_pro_core::domain::monitoring::MonitoringSnapshot>,
    pub(super) monitoring_error: Option<String>,
    pub(super) monitoring_poll: bool,
    pub(super) monitoring_last_poll: Option<std::time::Instant>,
    pub(super) monitoring_snapshot_request: Option<(u64, String)>,
    pub(super) monitoring_workload_request: Option<(u64, String)>,
    /// Full session rows (not just pid) so the terminate/cancel dialogs can
    /// show user/database/query context instead of a bare pid (#386).
    pub(super) monitoring_terminate_confirm: Option<db_pro_core::domain::monitoring::MonitorSession>,
    pub(super) monitoring_cancel_confirm: Option<db_pro_core::domain::monitoring::MonitorSession>,
    pub(super) monitoring_filter_active_only: bool,
    pub(super) monitoring_maintenance_confirm: Option<db_pro_core::domain::monitoring::MaintenanceAction>,
    pub(super) monitoring_maintenance_schema: String,
    pub(super) monitoring_maintenance_table: String,
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
            monitoring_snapshot_request: None,
            monitoring_workload_request: None,
            monitoring_terminate_confirm: None,
            monitoring_cancel_confirm: None,
            monitoring_filter_active_only: true,
            monitoring_maintenance_confirm: None,
            monitoring_maintenance_schema: String::new(),
            monitoring_maintenance_table: String::new(),
            monitoring_stat_sort: db_pro_core::domain::monitoring::StatStatementSort::TotalTime,
            monitoring_reset_stats_confirm: false,
            monitoring_workload_prev: None,
            monitoring_workload_filter: String::new(),
        }
    }
}

#[cfg(test)]
#[path = "tests/monitoring_state_tests.rs"]
mod tests;

