use super::*;

pub(super) fn draw_maintenance_activity(ui: &mut egui::Ui, theme: DbProTheme, monitoring: &mut MonitoringState) {
    ui.add_space(SPACE_MD);
    section_label(ui, "MAINTENANCE", theme);
    ui.add_space(SPACE_SM);
    ui.label(
        RichText::new("Actions run through MonitoringService — SQL is never built in the UI.")
            .small()
            .color(theme.text_muted),
    );
    ui.horizontal_wrapped(|ui| {
        use db_pro_core::domain::monitoring::MaintenanceAction;
        for action in [
            MaintenanceAction::Analyze,
            MaintenanceAction::Vacuum,
            MaintenanceAction::VacuumAnalyze,
        ] {
            if secondary_button(ui, action.as_label(), theme).clicked() {
                monitoring.monitoring_maintenance_confirm = Some(action);
            }
        }
    });
}
