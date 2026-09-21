//! Monitoring destructive-action confirmations and user intents.

use super::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum MonitoringConfirmationAction {
    ConfirmTerminate(i64),
    CancelTerminate,
    ConfirmMaintenance(db_pro_core::domain::monitoring::MaintenanceAction),
    CancelMaintenance,
    ConfirmResetStatistics,
    CancelResetStatistics,
}

pub(super) struct MonitoringConfirmationContext {
    pub(super) theme: DbProTheme,
    pub(super) terminate_backend_id: Option<i64>,
    pub(super) maintenance_action: Option<db_pro_core::domain::monitoring::MaintenanceAction>,
    pub(super) reset_statistics: bool,
}

impl MonitoringConfirmationContext {
    pub(super) fn draw(&self, ctx: &egui::Context) -> Vec<MonitoringConfirmationAction> {
        let mut actions = Vec::new();
        if let Some(backend_id) = self.terminate_backend_id {
            actions.extend(self.draw_terminate(ctx, backend_id));
        }
        if let Some(action) = self.maintenance_action {
            actions.extend(self.draw_maintenance(ctx, action));
        }
        if self.reset_statistics {
            actions.extend(self.draw_reset_statistics(ctx));
        }
        actions
    }

    fn draw_terminate(&self, ctx: &egui::Context, backend_id: i64) -> Vec<MonitoringConfirmationAction> {
        let mut actions = Vec::new();
        egui::Window::new("Terminate session?")
            .collapsible(false)
            .resizable(false)
            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
            .show(ctx, |ui| {
                ui.label(format!(
                    "Terminate PostgreSQL backend pid {backend_id}? This disconnects the client."
                ));
                ui.horizontal(|ui| {
                    if danger_button(ui, "Terminate", self.theme).clicked() {
                        actions.push(MonitoringConfirmationAction::ConfirmTerminate(backend_id));
                    }
                    if secondary_button_with_icon(ui, Icon::X, "Cancel", self.theme).clicked() {
                        actions.push(MonitoringConfirmationAction::CancelTerminate);
                    }
                });
            });
        actions
    }

    fn draw_maintenance(
        &self,
        ctx: &egui::Context,
        action: db_pro_core::domain::monitoring::MaintenanceAction,
    ) -> Vec<MonitoringConfirmationAction> {
        let mut actions = Vec::new();
        egui::Window::new("Run maintenance?")
            .collapsible(false)
            .resizable(false)
            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
            .show(ctx, |ui| {
                ui.label(format!(
                    "Run {} on the active database? Long-running VACUUM can take locks.",
                    action.as_label()
                ));
                ui.horizontal(|ui| {
                    if danger_button(ui, action.as_label(), self.theme).clicked() {
                        actions.push(MonitoringConfirmationAction::ConfirmMaintenance(action));
                    }
                    if secondary_button_with_icon(ui, Icon::X, "Cancel", self.theme).clicked() {
                        actions.push(MonitoringConfirmationAction::CancelMaintenance);
                    }
                });
            });
        actions
    }

    fn draw_reset_statistics(&self, ctx: &egui::Context) -> Vec<MonitoringConfirmationAction> {
        let mut actions = Vec::new();
        egui::Window::new("Reset pg_stat_statements?")
            .collapsible(false)
            .resizable(false)
            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
            .show(ctx, |ui| {
                ui.label(
                    "This clears all accumulated statement statistics on the server. \
                     It is an administrative action and cannot be undone.",
                );
                ui.horizontal(|ui| {
                    if danger_button(ui, "Reset statistics", self.theme).clicked() {
                        actions.push(MonitoringConfirmationAction::ConfirmResetStatistics);
                    }
                    if secondary_button_with_icon(ui, Icon::X, "Cancel", self.theme).clicked() {
                        actions.push(MonitoringConfirmationAction::CancelResetStatistics);
                    }
                });
            });
        actions
    }
}
