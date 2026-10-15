//! Monitoring destructive-action confirmations and user intents.

use super::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum MonitoringConfirmationAction {
    ConfirmTerminate(i64),
    CancelTerminate,
    ConfirmCancelBackend(i64),
    DismissCancelBackend,
    ConfirmMaintenance(db_pro_core::domain::monitoring::MaintenanceAction),
    CancelMaintenance,
    ConfirmResetStatistics,
    CancelResetStatistics,
}

pub(super) struct MonitoringConfirmationContext {
    pub(super) theme: DbProTheme,
    pub(super) terminate_backend_id: Option<i64>,
    pub(super) cancel_backend_id: Option<i64>,
    pub(super) maintenance_action: Option<db_pro_core::domain::monitoring::MaintenanceAction>,
    /// `schema.table`, or `entire database` when no target is set.
    pub(super) maintenance_target: String,
    pub(super) reset_statistics: bool,
}

impl MonitoringConfirmationContext {
    pub(super) fn draw(&self, ctx: &egui::Context) -> Vec<MonitoringConfirmationAction> {
        let mut actions = Vec::new();
        if let Some(backend_id) = self.terminate_backend_id {
            actions.extend(self.draw_terminate(ctx, backend_id));
        }
        if let Some(backend_id) = self.cancel_backend_id {
            actions.extend(self.draw_cancel_backend(ctx, backend_id));
        }
        if let Some(action) = self.maintenance_action {
            actions.extend(self.draw_maintenance(ctx, action, &self.maintenance_target));
        }
        if self.reset_statistics {
            actions.extend(self.draw_reset_statistics(ctx));
        }
        actions
    }

    fn draw_terminate(&self, ctx: &egui::Context, backend_id: i64) -> Vec<MonitoringConfirmationAction> {
        let mut actions = Vec::new();
        let mut open = true;
        Dialog::new(&mut open, "Terminate session?", self.theme)
            .width(460.0)
            .id_salt("monitoring_terminate_dialog")
            .show_framed_ctx(ctx, |frame| {
                frame.body(|ui| {
                    ui.label(format!(
                        "Terminate PostgreSQL backend pid {backend_id}? This disconnects the client."
                    ));
                });
                frame.footer(|ui| {
                    if danger_button(ui, "Terminate", self.theme).clicked() {
                        actions.push(MonitoringConfirmationAction::ConfirmTerminate(backend_id));
                    }
                    if secondary_button_with_icon(ui, Icon::X, "Cancel", self.theme).clicked() {
                        actions.push(MonitoringConfirmationAction::CancelTerminate);
                    }
                });
            });
        if !open {
            actions.push(MonitoringConfirmationAction::CancelTerminate);
        }
        actions
    }

    fn draw_cancel_backend(&self, ctx: &egui::Context, backend_id: i64) -> Vec<MonitoringConfirmationAction> {
        let mut actions = Vec::new();
        let mut open = true;
        Dialog::new(&mut open, "Cancel backend?", self.theme)
            .width(460.0)
            .id_salt("monitoring_cancel_backend_dialog")
            .show_framed_ctx(ctx, |frame| {
                frame.body(|ui| {
                    ui.label(format!("Cancel backend pid {backend_id}?"));
                });
                frame.footer(|ui| {
                    if danger_button(ui, "Cancel backend", self.theme).clicked() {
                        actions.push(MonitoringConfirmationAction::ConfirmCancelBackend(backend_id));
                    }
                    if secondary_button_with_icon(ui, Icon::X, "Keep running", self.theme).clicked() {
                        actions.push(MonitoringConfirmationAction::DismissCancelBackend);
                    }
                });
            });
        if !open {
            actions.push(MonitoringConfirmationAction::DismissCancelBackend);
        }
        actions
    }

    fn draw_maintenance(
        &self,
        ctx: &egui::Context,
        action: db_pro_core::domain::monitoring::MaintenanceAction,
        target: &str,
    ) -> Vec<MonitoringConfirmationAction> {
        let mut actions = Vec::new();
        let mut open = true;
        Dialog::new(&mut open, "Run maintenance?", self.theme)
            .width(460.0)
            .id_salt("monitoring_maintenance_dialog")
            .show_framed_ctx(ctx, |frame| {
                frame.body(|ui| {
                    ui.label(format!(
                        "Run {} on {target}? Long-running VACUUM can take locks.",
                        action.as_label()
                    ));
                });
                frame.footer(|ui| {
                    if danger_button(ui, action.as_label(), self.theme).clicked() {
                        actions.push(MonitoringConfirmationAction::ConfirmMaintenance(action));
                    }
                    if secondary_button_with_icon(ui, Icon::X, "Cancel", self.theme).clicked() {
                        actions.push(MonitoringConfirmationAction::CancelMaintenance);
                    }
                });
            });
        if !open {
            actions.push(MonitoringConfirmationAction::CancelMaintenance);
        }
        actions
    }

    fn draw_reset_statistics(&self, ctx: &egui::Context) -> Vec<MonitoringConfirmationAction> {
        let mut actions = Vec::new();
        let mut open = true;
        Dialog::new(&mut open, "Reset pg_stat_statements?", self.theme)
            .width(460.0)
            .id_salt("monitoring_reset_statistics_dialog")
            .show_framed_ctx(ctx, |frame| {
                frame.body(|ui| {
                    ui.label(
                        "This clears all accumulated statement statistics on the server. \
                         It is an administrative action and cannot be undone.",
                    );
                });
                frame.footer(|ui| {
                    if danger_button(ui, "Reset statistics", self.theme).clicked() {
                        actions.push(MonitoringConfirmationAction::ConfirmResetStatistics);
                    }
                    if secondary_button_with_icon(ui, Icon::X, "Cancel", self.theme).clicked() {
                        actions.push(MonitoringConfirmationAction::CancelResetStatistics);
                    }
                });
            });
        if !open {
            actions.push(MonitoringConfirmationAction::CancelResetStatistics);
        }
        actions
    }
}
