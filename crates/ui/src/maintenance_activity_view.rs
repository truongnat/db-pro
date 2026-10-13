use super::*;

pub(super) fn draw_maintenance_activity(
    ui: &mut egui::Ui,
    theme: DbProTheme,
    schema: &mut String,
    table: &mut String,
) -> Option<db_pro_core::domain::monitoring::MaintenanceAction> {
    let mut selected = None;
    ui.add_space(SPACE_MD);
    section_label(ui, "MAINTENANCE", theme);
    ui.add_space(SPACE_SM);
    ui.label(
        RichText::new("Actions run through MonitoringService — SQL is never built in the UI.")
            .small()
            .color(theme.text_muted),
    );
    ui.horizontal(|ui| {
        ui.label(RichText::new("Schema").small().color(theme.text_muted));
        ui.add(egui::TextEdit::singleline(schema).desired_width(120.0).hint_text("public"));
        ui.label(RichText::new("Table").small().color(theme.text_muted));
        ui.add(egui::TextEdit::singleline(table).desired_width(160.0).hint_text("table"));
    });
    ui.horizontal_wrapped(|ui| {
        use db_pro_core::domain::monitoring::MaintenanceAction;
        for action in [
            MaintenanceAction::Analyze,
            MaintenanceAction::Vacuum,
            MaintenanceAction::VacuumAnalyze,
        ] {
            if secondary_button(ui, action.as_label(), theme).clicked() {
                selected = Some(action);
            }
        }
    });
    selected
}
