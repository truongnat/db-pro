//! Result-grid selection and keyboard navigation helpers.
use super::result_grid_view::GridSelectionLookup;
use super::*;

impl DbProApp {
    pub(super) fn select_visible_row(
        &mut self,
        indexes: &[usize],
        row_positions: &HashMap<usize, usize>,
        position: usize,
        extend: bool,
        toggle: bool,
    ) {
        let Some(&row_index) = indexes.get(position) else {
            return;
        };

        if extend {
            let anchor = self.selection_anchor_row.or(self.selected_row).unwrap_or(row_index);
            let anchor_position = row_positions.get(&anchor).copied().unwrap_or(position);
            let (start, end) = if anchor_position <= position {
                (anchor_position, position)
            } else {
                (position, anchor_position)
            };
            self.selected_rows.clear();
            self.selected_rows.extend(indexes[start..=end].iter().copied());
        } else if toggle {
            if !self.selected_rows.remove(&row_index) {
                self.selected_rows.insert(row_index);
            }
            if self.selected_rows.is_empty() {
                self.selected_rows.insert(row_index);
            }
        } else {
            self.selected_rows.clear();
            self.selected_rows.insert(row_index);
        }

        self.selected_row = if self.selected_rows.contains(&row_index) {
            Some(row_index)
        } else {
            self.selected_rows.iter().next().copied()
        };
        if !extend {
            self.selection_anchor_row = Some(row_index);
        }
    }

    pub(super) fn select_single_row(&mut self, row_index: usize) {
        self.selected_rows.clear();
        self.selected_rows.insert(row_index);
        self.selected_row = Some(row_index);
        self.selection_anchor_row = Some(row_index);
        self.selection_anchor_cell = None;
    }

    pub(super) fn select_single_cell(&mut self, selection: (usize, usize)) {
        self.selected_cell = Some(selection);
        self.selected_rows.clear();
        self.selected_rows.insert(selection.0);
        self.selected_row = Some(selection.0);
        self.selection_anchor_row = Some(selection.0);
        self.selection_anchor_cell = Some(selection);
    }

    pub(super) fn select_cell_range(
        &mut self,
        indexes: &[usize],
        row_positions: &HashMap<usize, usize>,
        focus: (usize, usize),
        extend: bool,
    ) {
        if !extend {
            self.select_single_cell(focus);
            return;
        }

        let anchor = self.selection_anchor_cell.or(self.selected_cell).unwrap_or(focus);
        let anchor_row = row_positions.get(&anchor.0).copied().unwrap_or(0);
        let focus_row = row_positions.get(&focus.0).copied().unwrap_or(anchor_row);
        let (row_start, row_end) = if anchor_row <= focus_row {
            (anchor_row, focus_row)
        } else {
            (focus_row, anchor_row)
        };
        self.selected_rows.clear();
        self.selected_rows.extend(indexes[row_start..=row_end].iter().copied());
        self.selected_row = Some(focus.0);
        self.selected_cell = Some(focus);
        self.selection_anchor_row = Some(anchor.0);
        self.selection_anchor_cell = Some(anchor);
    }

    pub(super) fn select_all_visible_cells(&mut self, indexes: &[usize], order: &[usize]) {
        let (Some(&first_row), Some(&last_row), Some(&first_column), Some(&last_column)) =
            (indexes.first(), indexes.last(), order.first(), order.last())
        else {
            return;
        };
        self.selected_rows.clear();
        self.selected_rows.extend(indexes.iter().copied());
        self.selection_anchor_row = Some(first_row);
        self.selection_anchor_cell = Some((first_row, first_column));
        self.selected_row = Some(last_row);
        self.selected_cell = Some((last_row, last_column));
        self.copy_status.clear();
    }

    pub(super) fn is_cell_selected(&self, lookup: &GridSelectionLookup, selection: (usize, usize)) -> bool {
        let Some(anchor) = self.selection_anchor_cell else {
            return self.selected_cell == Some(selection);
        };
        let Some(focus) = self.selected_cell else {
            return false;
        };
        let Some(&anchor_row) = lookup.row_positions.get(&anchor.0) else {
            return self.selected_cell == Some(selection);
        };
        let Some(&focus_row) = lookup.row_positions.get(&focus.0) else {
            return false;
        };
        let Some(&selection_row) = lookup.row_positions.get(&selection.0) else {
            return false;
        };
        let Some(&anchor_column) = lookup.column_positions.get(&anchor.1) else {
            return self.selected_cell == Some(selection);
        };
        let Some(&focus_column) = lookup.column_positions.get(&focus.1) else {
            return false;
        };
        let Some(&selection_column) = lookup.column_positions.get(&selection.1) else {
            return false;
        };
        let row_in_range = selection_row >= anchor_row.min(focus_row) && selection_row <= anchor_row.max(focus_row);
        let column_in_range =
            selection_column >= anchor_column.min(focus_column) && selection_column <= anchor_column.max(focus_column);
        row_in_range && column_in_range
    }

    pub(super) fn mutation_error_for_row(&self, result: &UiQueryResult, row_index: usize) -> bool {
        let Some(failure) = self.table_mutation_error.as_ref() else {
            return false;
        };
        let Some(identity) = self.row_identity_for_result(result, row_index) else {
            return false;
        };
        match failure.target.as_ref() {
            Some(MutationTarget::Update { identity: target, .. })
            | Some(MutationTarget::Delete { identity: target, .. }) => target == &identity,
            Some(MutationTarget::Insert) | None => false,
        }
    }

    pub(super) fn mutation_error_for_cell(
        &self,
        result: &UiQueryResult,
        row_index: usize,
        column_index: usize,
    ) -> bool {
        let Some(failure) = self.table_mutation_error.as_ref() else {
            return false;
        };
        let Some(identity) = self.row_identity_for_result(result, row_index) else {
            return false;
        };
        match failure.target.as_ref() {
            Some(MutationTarget::Update {
                identity: target,
                columns,
                ..
            }) => target == &identity && columns.contains(&column_index),
            Some(MutationTarget::Delete { identity: target, .. }) => target == &identity,
            Some(MutationTarget::Insert) | None => false,
        }
    }

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
            if let Some((curr_row, curr_col)) = self.selected_cell {
                if editable && self.data_editing_cell.is_some() && !self.commit_active_data_edit(result) {
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
                    self.select_cell_range(indexes, &selection_lookup.row_positions, selection, false);
                    self.data_editing_cell = None;
                    self.data_edit_value.clear();
                    self.copy_status.clear();
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

        if let Some((curr_row, curr_col)) = self.selected_cell {
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
                self.select_cell_range(
                    indexes,
                    &selection_lookup.row_positions,
                    selection,
                    ui.input(|input| input.modifiers.shift),
                );
                self.data_editing_cell = None;
                self.data_edit_value.clear();
                self.copy_status.clear();
            }
        } else if let Some(&first_row) = indexes.first() {
            self.select_single_cell((first_row, order[0]));
        }
    }
}
