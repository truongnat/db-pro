use super::*;

impl DbProApp {
    pub(super) fn draw_maintenance_activity(&mut self, ui: &mut egui::Ui) {
        ui.add_space(SPACE_MD);
        section_label(ui, "MAINTENANCE", self.theme);
        ui.add_space(SPACE_SM);
        ui.label(
            RichText::new("Actions run through MonitoringService — SQL is never built in the UI.")
                .small()
                .color(self.theme.text_muted),
        );
        ui.horizontal_wrapped(|ui| {
            use db_pro_core::domain::monitoring::MaintenanceAction;
            for action in [
                MaintenanceAction::Analyze,
                MaintenanceAction::Vacuum,
                MaintenanceAction::VacuumAnalyze,
            ] {
                if secondary_button(ui, action.as_label(), self.theme).clicked() {
                    self.management.monitoring.monitoring_maintenance_confirm = Some(action);
                }
            }
        });
    }
}
