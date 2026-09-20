//! Result-grid selection and keyboard navigation helpers.
use super::result_grid_view::GridSelectionLookup;
use super::*;

impl DbProApp {
    /// Arrow / Tab / Home / End navigation over the visible (filtered, sorted) indexes and column order.
    pub(super) fn handle_grid_navigation(
        &mut self,
        ui: &mut egui::Ui,
        indexes: &[usize],
        order: &[usize],
        editable: bool,
        result: &UiQueryResult,
        selection_lookup: &GridSelectionLookup,
    ) {
        if order.is_empty() || indexes.is_empty() {
            return;
        }

        let is_tab = ui.input(|input| input.key_pressed(egui::Key::Tab));
        let is_shift_tab = is_tab && ui.input(|input| input.modifiers.shift);

        if is_tab && editable && self.table.editing.data_editing_cell.is_some() && !self.commit_active_data_edit(result)
        {
            return;
        }

        let navigation_key = if is_tab {
            Some(egui::Key::Tab)
        } else {
            ui.input(|input| {
                [
                    egui::Key::ArrowUp,
                    egui::Key::ArrowDown,
                    egui::Key::ArrowLeft,
                    egui::Key::ArrowRight,
                    egui::Key::Home,
                    egui::Key::End,
                ]
                .into_iter()
                .find(|key| input.key_pressed(*key))
            })
        };
        let Some(key) = navigation_key else {
            return;
        };

        if let Some(selection) = self
            .table
            .data
            .navigation_target(indexes, order, selection_lookup, key, is_shift_tab)
        {
            self.table.data.select_cell_range(
                indexes,
                &selection_lookup.row_positions,
                selection,
                ui.input(|input| input.modifiers.shift),
            );
            self.table.editing.data_editing_cell = None;
            self.table.editing.data_edit_value.clear();
            self.feedback.copy_status.clear();
        }
    }
}
