//! State and command planning for the database monitoring surface.

pub(super) struct MonitoringState {
    pub(super) monitoring_snapshot: Option<db_pro_core::domain::monitoring::MonitoringSnapshot>,
    pub(super) monitoring_error: Option<String>,
    pub(super) monitoring_poll: bool,
    pub(super) monitoring_last_poll: Option<std::time::Instant>,
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
    /// One-shot presentation focus used only by deterministic capture fixtures.
    pub(super) capture_workload_focus_once: bool,
}

impl MonitoringState {
    pub(crate) fn reset_connection_scope(&mut self) {
        self.monitoring_snapshot = None;
        self.monitoring_error = None;
        self.monitoring_last_poll = None;
        self.monitoring_terminate_confirm = None;
        self.monitoring_cancel_confirm = None;
        self.monitoring_maintenance_confirm = None;
        self.monitoring_workload_prev = None;
    }
}
impl Default for MonitoringState {
    fn default() -> Self {
        Self {
            monitoring_snapshot: None,
            monitoring_error: None,
            monitoring_poll: true,
            monitoring_last_poll: None,
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
            capture_workload_focus_once: false,
        }
    }
}

#[cfg(test)]
#[path = "tests/monitoring_state_tests.rs"]
mod tests;
