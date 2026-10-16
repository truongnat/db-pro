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
        crate::components::input::Input::new(schema, "public", theme)
            .width(120.0)
            .show(ui);
        ui.label(RichText::new("Table").small().color(theme.text_muted));
        crate::components::input::Input::new(table, "table", theme)
            .width(160.0)
            .show(ui);
    });
    // A schema without a table (or a bare table name) would silently expand to
    // a whole-database run — the service only supports full-DB or
    // schema.table targets (#403).
    let partial_target = schema.trim().is_empty() != table.trim().is_empty();
    let mut rejected: Option<String> = None;
    ui.horizontal_wrapped(|ui| {
        use db_pro_core::domain::monitoring::MaintenanceAction;
        for action in [
            MaintenanceAction::Analyze,
            MaintenanceAction::Vacuum,
            MaintenanceAction::VacuumAnalyze,
        ] {
            if secondary_button(ui, action.as_label(), theme).clicked() {
                if partial_target {
                    rejected = Some(
                        "Maintenance needs both schema and table (or neither for the whole database)".to_owned(),
                    );
                } else {
                    selected = Some(action);
                }
            }
        }
    });
    if let Some(message) = rejected {
        ui.colored_label(theme.danger, message);
    }
    selected
}
