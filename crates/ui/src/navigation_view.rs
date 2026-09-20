use super::*;

impl DbProApp {
    pub(super) fn draw_diagram_sidebar(&mut self, ui: &mut egui::Ui) {
        ui.vertical(|ui| {
            section_label(ui, "SCHEMA MAP", self.theme);
            ui.add_space(8.0);
            ui.label(RichText::new("Tables and foreign-key relationships").color(self.theme.text_primary));
            ui.add_space(4.0);
            ui.label(
                RichText::new(format!(
                    "{} · {}",
                    plural_count(self.schema.explorer.schema.table_details.len(), "table", "tables"),
                    plural_count(
                        self.schema
                            .explorer
                            .schema
                            .table_details
                            .iter()
                            .map(|table| table.foreign_keys.len())
                            .sum::<usize>(),
                        "relationship",
                        "relationships",
                    )
                ))
                .small()
                .color(self.theme.text_muted),
            );
        });
        ui.add_space(12.0);
        ui.separator();
        ui.add_space(12.0);
        ui.vertical(|ui| {
            section_label(ui, "NAVIGATION", self.theme);
            ui.add_space(8.0);
            ui.label(
                RichText::new(
                    "Drag the canvas to pan. Use the floating controls in the map to zoom or fit the schema.",
                )
                .small()
                .color(self.theme.text_secondary),
            );
            ui.add_space(10.0);
            if secondary_button_with_icon(ui, Icon::Database, "Back to Explorer", self.theme).clicked() {
                self.workspace.activity = Activity::Explorer;
                self.workspace.sidebar_open = true;
            }
        });
    }
}
