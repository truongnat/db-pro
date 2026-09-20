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

        if is_tab {
            if let Some((curr_row, curr_col)) = self.table_data.selected_cell {
                if editable && self.table_data.data_editing_cell.is_some() && !self.commit_active_data_edit(result) {
                    return;
                }
                let visual_col = selection_lookup.column_positions.get(&curr_col).copied().unwrap_or(0);
                let next_cell = if is_shift_tab {
                    if visual_col > 0 {
                        Some((curr_row, order[visual_col - 1]))
                    } else if let Some(pos) = selection_lookup.row_positions.get(&curr_row).copied() {
                        if pos > 0 {
                            Some((indexes[pos - 1], order[order.len() - 1]))
                        } else {
                            Some((curr_row, curr_col))
                        }
                    } else {
                        Some((curr_row, curr_col))
                    }
                } else if visual_col + 1 < order.len() {
                    Some((curr_row, order[visual_col + 1]))
                } else if let Some(pos) = selection_lookup.row_positions.get(&curr_row).copied() {
                    if pos + 1 < indexes.len() {
                        Some((indexes[pos + 1], order[0]))
                    } else {
                        Some((curr_row, curr_col))
                    }
                } else {
                    Some((curr_row, curr_col))
                };
                if let Some(selection) = next_cell {
                    self.table_data
                        .select_cell_range(indexes, &selection_lookup.row_positions, selection, false);
                    self.table_data.data_editing_cell = None;
                    self.table_data.data_edit_value.clear();
                    self.feedback.copy_status.clear();
                }
                return;
            }
        }

        let navigation_key = ui.input(|input| {
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
        });
        let Some(key) = navigation_key else {
            return;
        };

        if let Some((curr_row, curr_col)) = self.table_data.selected_cell {
            let row_pos = selection_lookup.row_positions.get(&curr_row).copied().unwrap_or(0);
            let visual_col = selection_lookup.column_positions.get(&curr_col).copied().unwrap_or(0);

            let next_selection = match key {
                egui::Key::ArrowUp => {
                    let next_pos = row_pos.saturating_sub(1);
                    Some((indexes[next_pos], curr_col))
                }
                egui::Key::ArrowDown => {
                    let next_pos = (row_pos + 1).min(indexes.len() - 1);
                    Some((indexes[next_pos], curr_col))
                }
                egui::Key::ArrowLeft => {
                    if visual_col > 0 {
                        Some((curr_row, order[visual_col - 1]))
                    } else {
                        Some((curr_row, curr_col))
                    }
                }
                egui::Key::ArrowRight => {
                    if visual_col + 1 < order.len() {
                        Some((curr_row, order[visual_col + 1]))
                    } else {
                        Some((curr_row, curr_col))
                    }
                }
                egui::Key::Home => Some((curr_row, order[0])),
                egui::Key::End => Some((curr_row, order[order.len() - 1])),
                _ => None,
            };

            if let Some(selection) = next_selection {
                self.table_data.select_cell_range(
                    indexes,
                    &selection_lookup.row_positions,
                    selection,
                    ui.input(|input| input.modifiers.shift),
                );
                self.table_data.data_editing_cell = None;
                self.table_data.data_edit_value.clear();
                self.feedback.copy_status.clear();
            }
        } else if let Some(&first_row) = indexes.first() {
            self.table_data.select_single_cell((first_row, order[0]));
        }
    }
}
