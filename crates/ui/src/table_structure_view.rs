//! Table structure root adapter.
use super::*;

#[path = "table_structure_surface_view.rs"]
pub(super) mod table_structure_surface_view;

impl DbProApp {
    /// Draws the structure surface from an immutable table-info snapshot.
    pub(super) fn draw_table_structure_view(&mut self, ui: &mut egui::Ui) {
        let Some(info) = self.table.state.table_info.clone() else {
            table_structure_surface_view::draw_placeholder(
                self.theme,
                self.table.state.table_info_error.as_deref(),
                ui,
            );
            return;
        };
        let mut context = table_structure_surface_view::TableStructureContext {
            theme: self.theme,
            info: &info,
            search: &mut self.table.state.table_structure_search,
        };
        context.draw(ui);
    }
}
