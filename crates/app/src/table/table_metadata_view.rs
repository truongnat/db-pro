use super::*;
#[path = "table_indexes_surface_view.rs"]
mod table_indexes_surface_view;

impl DbProApp {
    /// Adapts table metadata state into the indexes presentation surface.
    pub(super) fn draw_table_indexes_view(&mut self, ui: &mut egui::Ui) {
        let Some(info) = self.table.state.table_info.clone() else {
            table_indexes_surface_view::draw_loading(self.theme, ui);
            return;
        };
        let mut context = table_indexes_surface_view::TableIndexesContext {
            theme: self.theme,
            info: &info,
            search: &mut self.table.state.table_metadata_search,
        };
        context.draw(ui);
    }
}
