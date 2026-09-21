//! Table structure root adapter.
use super::*;

#[path = "table_structure_surface_view.rs"]
mod table_structure_surface_view;

impl DbProApp {
    /// Draws the structure surface from an immutable table-info snapshot.
    pub(super) fn draw_table_structure_view(&mut self, ui: &mut egui::Ui) {
        let Some(info) = self.table.state.table_info.clone() else {
            self.draw_table_structure_placeholder(ui);
            return;
        };
        let actions = {
            let egui_context = ui.ctx().clone();
            let mut context = table_structure_surface_view::TableStructureContext {
                theme: self.theme,
                info: &info,
                search: &mut self.table.state.table_structure_search,
                selected_column: self.table.state.table_column_detail.as_deref(),
            };
            context.draw(ui, &egui_context)
        };
        for action in actions {
            match action {
                table_structure_surface_view::TableStructureAction::SelectColumn(name) => {
                    self.table.state.table_column_detail = Some(name);
                }
                table_structure_surface_view::TableStructureAction::CloseColumnDetail => {
                    self.table.state.table_column_detail = None;
                }
            }
        }
    }
}
