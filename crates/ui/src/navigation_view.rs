use super::*;

pub(super) fn draw_diagram_sidebar(ui: &mut egui::Ui, theme: DbProTheme, schema: &UiSchemaSummary) -> bool {
    ui.vertical(|ui| {
        section_label(ui, "SCHEMA MAP", theme);
        ui.add_space(8.0);
        ui.label(RichText::new("Tables and foreign-key relationships").color(theme.text_primary));
        ui.add_space(4.0);
        ui.label(
            RichText::new(format!(
                "{} · {}",
                plural_count(schema.table_details.len(), "table", "tables"),
                plural_count(
                    schema
                        .table_details
                        .iter()
                        .map(|table| table.foreign_keys.len())
                        .sum::<usize>(),
                    "relationship",
                    "relationships",
                )
            ))
            .small()
            .color(theme.text_muted),
        );
    });
    ui.add_space(12.0);
    ui.separator();
    ui.add_space(12.0);
    ui.vertical(|ui| {
        section_label(ui, "NAVIGATION", theme);
        ui.add_space(8.0);
        ui.label(
            RichText::new("Drag to pan, scroll to zoom. Click a table to pin its relationships; double-click to open it.")
                .small()
                .color(theme.text_secondary),
        );
        ui.add_space(10.0);
        secondary_button_with_icon(ui, Icon::Database, "Back to Explorer", theme).clicked()
    })
    .inner
}
