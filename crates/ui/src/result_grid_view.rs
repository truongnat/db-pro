use super::*;
use egui::{Align2, Pos2, Rect, Rounding, Stroke, Vec2};
use std::collections::{HashMap, HashSet};

/// Coordinate lookup for the current visible grid slice.
///
/// Selection is rendered once per visible cell, so resolving coordinates must
/// not scan the filtered row list and reordered column list for every cell.
struct GridSelectionLookup {
    row_positions: HashMap<usize, usize>,
    column_positions: HashMap<usize, usize>,
}

impl GridSelectionLookup {
    fn new(indexes: &[usize], order: &[usize]) -> Self {
        Self {
            row_positions: indexes
                .iter()
                .enumerate()
                .map(|(position, &row)| (row, position))
                .collect(),
            column_positions: order
                .iter()
                .enumerate()
                .map(|(position, &column)| (column, position))
                .collect(),
        }
    }
}

/// Per-cell render context for the result grid.
struct GridCell<'a> {
    visible_indexes: &'a [usize],
    selection_lookup: &'a GridSelectionLookup,
    row_index: usize,
    column_index: usize,
    display_position: usize,
    row_selected: bool,
    row_dirty: bool,
    row_mutation_error: bool,
    cell_mutation_error: bool,
    editable: bool,
    width: f32,
    cell: &'a UiCell,
}

/// Context shared by every visible grid row.
struct GridRows<'a> {
    indexes: &'a [usize],
    widths: &'a [f32],
    order: &'a [usize],
    editable: bool,
    row_offset: u64,
    selection_lookup: &'a GridSelectionLookup,
}

impl DbProApp {
    pub(super) fn draw_result_grid(&mut self, ui: &mut egui::Ui, result: &UiQueryResult) {
        if result.columns.is_empty() {
            ui.centered_and_justified(|ui| {
                empty_state(
                    ui,
                    Icon::CircleCheck,
                    "Statement completed",
                    "This statement did not return any rows to display.",
                    self.theme,
                );
            });
            return;
        }

        let order = self.column_order_for_columns(&result.columns);
        let editable =
            self.active_tab == WorkspaceTab::Table && self.table_view == TableView::Data && self.can_edit_table_rows();
        let indexes =
            crate::filtered_sorted_indexes(result, &self.grid_filter, self.grid_sort_column, self.grid_sort_desc);

        if self.active_tab == WorkspaceTab::Table && self.table_view == TableView::Data {
            self.rebuild_row_identity_cache(result, &indexes);
        } else {
            self.grid_row_identity_cache.clear();
            self.grid_row_identity_cache_ready = false;
        }

        self.handle_grid_keyboard(ui, result, &indexes, &order, editable);

        let is_table_data = self.active_tab == WorkspaceTab::Table && self.table_view == TableView::Data;
        if !is_table_data {
            self.draw_grid_toolbar(ui, result, editable, indexes.len(), &indexes);
        }

        let row_offset = if is_table_data { self.table_data_offset } else { 0 };
        self.draw_grid_body(ui, result, &indexes, &order, editable, row_offset);
    }

    /// Retrieve or initialize column visual ordering.
    pub(crate) fn column_order_for_columns(&mut self, columns: &[crate::UiColumn]) -> Vec<usize> {
        let count = columns.len();
        self.grid_layout_column_names = columns.iter().map(|column| column.name.clone()).collect();

        if let Some(mut persisted) = self.grid_pending_named_layout.take() {
            let indexes_by_name: HashMap<&str, usize> = columns
                .iter()
                .enumerate()
                .map(|(index, column)| (column.name.as_str(), index))
                .collect();
            persisted.sort_by_key(|column| column.order);
            let mut seen_names = HashSet::with_capacity(persisted.len());
            let mut seen_indices = HashSet::with_capacity(count);
            let mut order = Vec::with_capacity(count);
            let mut widths = vec![180.0; count];
            let mut hidden = BTreeSet::new();

            for entry in persisted {
                let Some(&index) = indexes_by_name.get(entry.column_name.as_str()) else {
                    continue;
                };
                if !seen_names.insert(entry.column_name) {
                    continue;
                }
                seen_indices.insert(index);
                order.push(index);
                widths[index] = entry.width.clamp(60.0, 1000.0);
                if entry.hidden {
                    hidden.insert(index);
                }
            }
            for index in 0..count {
                if seen_indices.insert(index) {
                    order.push(index);
                }
            }
            self.grid_column_order = order;
            self.grid_column_widths = widths;
            self.grid_hidden_columns = hidden;
            self.grid_columns_user_resized = true;
        } else if self.grid_legacy_layout_pending {
            // Index-based layouts cannot be safely migrated across a schema
            // shape change. Keep the old layout only when it exactly matches
            // the current schema; otherwise start from a safe default.
            let widths_match = self.grid_column_widths.is_empty() || self.grid_column_widths.len() == count;
            if self.grid_column_order.len() != count || !widths_match {
                self.grid_column_order.clear();
                self.grid_column_widths.clear();
                self.grid_hidden_columns.clear();
                self.grid_columns_user_resized = false;
            }
            self.grid_legacy_layout_pending = false;
        }

        self.column_order(count)
    }

    pub(crate) fn column_order(&mut self, count: usize) -> Vec<usize> {
        self.grid_hidden_columns.retain(|&column| column < count);
        let mut normalized_order = Vec::with_capacity(count);
        let mut seen = HashSet::with_capacity(count);
        for column in self.grid_column_order.iter().copied() {
            if column < count && seen.insert(column) {
                normalized_order.push(column);
            }
        }
        for column in 0..count {
            if seen.insert(column) {
                normalized_order.push(column);
            }
        }
        if normalized_order != self.grid_column_order {
            self.grid_column_order = normalized_order;
        }
        if !self.grid_column_widths.is_empty() {
            self.grid_column_widths.resize(count, 180.0);
            self.grid_column_widths
                .iter_mut()
                .for_each(|width| *width = width.clamp(60.0, 1000.0));
        }
        self.grid_column_order
            .iter()
            .copied()
            .filter(|column| !self.grid_hidden_columns.contains(column))
            .collect()
    }

    /// Reorder a column visually from one position to another.
    pub(crate) fn move_column(&mut self, from_visual_idx: usize, to_visual_idx: usize, count: usize) {
        let visible_order = self.column_order(count);
        if from_visual_idx < visible_order.len()
            && to_visual_idx < visible_order.len()
            && from_visual_idx != to_visual_idx
        {
            let from_column = visible_order[from_visual_idx];
            let to_column = visible_order[to_visual_idx];
            let from = self
                .grid_column_order
                .iter()
                .position(|column| *column == from_column)
                .unwrap_or(from_visual_idx);
            let to = self
                .grid_column_order
                .iter()
                .position(|column| *column == to_column)
                .unwrap_or(to_visual_idx);
            let column = self.grid_column_order.remove(from);
            self.grid_column_order.insert(to, column);
        }
    }

    fn hide_column(&mut self, column_index: usize, visible_count: usize) {
        if visible_count <= 1 {
            return;
        }
        self.grid_hidden_columns.insert(column_index);
    }

    fn show_all_columns(&mut self) {
        self.grid_hidden_columns.clear();
    }

    fn reset_grid_layout(&mut self, count: usize) {
        self.grid_column_order = (0..count).collect();
        self.grid_hidden_columns.clear();
        self.grid_column_widths.clear();
        self.grid_columns_user_resized = false;
    }

    fn auto_size_column(&mut self, result: &UiQueryResult, indexes: &[usize], column_index: usize) {
        if column_index >= result.columns.len() {
            return;
        }
        if self.grid_column_widths.len() < result.columns.len() {
            self.grid_column_widths.resize(result.columns.len(), 180.0);
        }
        let column = &result.columns[column_index];
        let content_width = indexes
            .iter()
            .take(100)
            .filter_map(|row_index| result.rows.get(*row_index).and_then(|row| row.get(column_index)))
            .map(crate::cell_text)
            .map(|value| value.chars().count() as f32 * 7.0 + 24.0)
            .fold(column.name.chars().count() as f32 * 7.0 + 42.0, f32::max);
        self.grid_column_widths[column_index] = content_width.clamp(60.0, 520.0);
        self.grid_columns_user_resized = true;
    }

    fn set_table_or_grid_sort(&mut self, result: &UiQueryResult, column_index: usize, descending: Option<bool>) {
        if self.active_tab == WorkspaceTab::Table && self.table_view == TableView::Data {
            if !self.staged_changes.is_empty() {
                self.runtime_message = "Apply or discard staged changes before changing sort".to_owned();
                return;
            }
            self.table_data_sorts = descending
                .and_then(|_| {
                    result.columns.get(column_index).map(|column| UiTableDataSort {
                        column: column.name.clone(),
                        descending: descending.unwrap_or(false),
                    })
                })
                .into_iter()
                .collect();
            self.grid_sort_column = None;
            self.grid_sort_desc = false;
            self.reload_table_data_from_start();
        } else {
            self.grid_sort_column = descending.map(|_| column_index);
            self.grid_sort_desc = descending.unwrap_or(false);
        }
    }

    /// Cycle a table-data sort clause. Shift-click keeps other clauses and
    /// makes the clicked column the next priority; plain click selects one
    /// clause and cycles ASC -> DESC -> none.
    fn cycle_table_data_sort(&mut self, result: &UiQueryResult, column_index: usize, additive: bool) {
        if !self.staged_changes.is_empty() {
            self.runtime_message = "Apply or discard staged changes before changing sort".to_owned();
            return;
        }
        let Some(column) = result.columns.get(column_index).map(|column| column.name.clone()) else {
            return;
        };
        if additive {
            if let Some(index) = self.table_data_sorts.iter().position(|sort| sort.column == column) {
                if self.table_data_sorts[index].descending {
                    self.table_data_sorts.remove(index);
                } else {
                    self.table_data_sorts[index].descending = true;
                }
            } else {
                self.table_data_sorts.push(UiTableDataSort {
                    column,
                    descending: false,
                });
            }
        } else if self.table_data_sorts.len() == 1
            && self.table_data_sorts.first().map(|sort| sort.column.as_str()) == Some(column.as_str())
        {
            if self.table_data_sorts[0].descending {
                self.table_data_sorts.clear();
            } else {
                self.table_data_sorts[0].descending = true;
            }
        } else {
            self.table_data_sorts = vec![UiTableDataSort {
                column,
                descending: false,
            }];
        }
        self.grid_sort_column = None;
        self.grid_sort_desc = false;
        self.reload_table_data_from_start();
    }

    /// Copy, paste-to-edit, staged-changes, and keyboard navigation for the result grid.
    fn handle_grid_keyboard(
        &mut self,
        ui: &mut egui::Ui,
        result: &UiQueryResult,
        indexes: &[usize],
        order: &[usize],
        editable: bool,
    ) {
        let modifier = Self::primary_modifier_pressed_ui(ui);
        let shift = ui.input(|i| i.modifiers.shift);

        if !ui.ctx().wants_keyboard_input()
            && ui.input(|input| input.key_pressed(egui::Key::A) && Self::primary_modifier_pressed(input))
        {
            self.select_all_visible_cells(indexes, order);
            return;
        }

        if !ui.ctx().wants_keyboard_input() && ui.input(|input| input.key_pressed(egui::Key::Escape)) {
            self.selected_cell = None;
            self.selected_row = None;
            self.selected_rows.clear();
            self.selection_anchor_row = None;
            self.selection_anchor_cell = None;
            self.copy_status.clear();
            return;
        }

        if ui.input(|i| i.key_pressed(egui::Key::C)) && modifier && shift {
            self.copy_selected_rows(ui, result);
        } else if ui.input(|i| i.key_pressed(egui::Key::C)) && modifier {
            self.copy_selected_cell(ui, result);
        }

        if ui.input(|input| input.key_pressed(egui::Key::S) && Self::primary_modifier_pressed(input)) {
            self.apply_staged_changes();
        }
        if ui.input(|input| input.key_pressed(egui::Key::Z) && Self::primary_modifier_pressed(input)) {
            if self.staged_changes.counts().total() > 1 {
                self.discard_changes_confirmation = true;
            } else {
                self.discard_staged_changes();
            }
        }
        if editable
            && self.data_editing_cell.is_none()
            && ui.input(|input| input.key_pressed(egui::Key::Delete) || input.key_pressed(egui::Key::Backspace))
        {
            self.request_delete_selected_data_rows(result);
        }
        let pasted = ui.input(|input| {
            input.events.iter().find_map(|event| match event {
                egui::Event::Paste(text) => Some(text.clone()),
                _ => None,
            })
        });
        if editable {
            self.handle_grid_edit_input(ui, result, pasted);
        }
        if self.data_editing_cell.is_some() && ui.input(|input| input.key_pressed(egui::Key::Tab)) {
            self.handle_grid_navigation(ui, indexes, order, editable, result);
            return;
        }
        if !ui.ctx().wants_keyboard_input() {
            self.handle_grid_navigation(ui, indexes, order, editable, result);
        }
    }

    fn primary_modifier_pressed_ui(ui: &egui::Ui) -> bool {
        ui.input(Self::primary_modifier_pressed)
    }

    /// Paste-into-cell and Enter/F2-to-edit while the grid is editable.
    fn handle_grid_edit_input(&mut self, ui: &mut egui::Ui, result: &UiQueryResult, pasted: Option<String>) {
        if let (Some((row_index, column_index)), Some(text)) = (self.selected_cell, pasted) {
            self.data_editing_cell = Some((row_index, column_index));
            self.data_edit_value = text;
            self.submit_data_cell_edit(result, row_index, column_index);
        }
        if self.data_editing_cell.is_none()
            && self.selected_cell.is_some()
            && ui.input(|input| input.key_pressed(egui::Key::Enter) || input.key_pressed(egui::Key::F2))
        {
            if let Some((row_index, column_index)) = self.selected_cell {
                if let Some(cell) = result.rows.get(row_index).and_then(|row| row.get(column_index)) {
                    self.begin_data_cell_edit(result, row_index, column_index, cell);
                }
            }
        }
    }

    fn select_visible_row(
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

    fn select_single_row(&mut self, row_index: usize) {
        self.selected_rows.clear();
        self.selected_rows.insert(row_index);
        self.selected_row = Some(row_index);
        self.selection_anchor_row = Some(row_index);
        self.selection_anchor_cell = None;
    }

    fn select_single_cell(&mut self, selection: (usize, usize)) {
        self.selected_cell = Some(selection);
        self.selected_rows.clear();
        self.selected_rows.insert(selection.0);
        self.selected_row = Some(selection.0);
        self.selection_anchor_row = Some(selection.0);
        self.selection_anchor_cell = Some(selection);
    }

    fn select_cell_range(
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

    fn select_all_visible_cells(&mut self, indexes: &[usize], order: &[usize]) {
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

    fn is_cell_selected(&self, lookup: &GridSelectionLookup, selection: (usize, usize)) -> bool {
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

    fn mutation_error_for_row(&self, result: &UiQueryResult, row_index: usize) -> bool {
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

    fn mutation_error_for_cell(&self, result: &UiQueryResult, row_index: usize, column_index: usize) -> bool {
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
    fn handle_grid_navigation(
        &mut self,
        ui: &mut egui::Ui,
        indexes: &[usize],
        order: &[usize],
        editable: bool,
        result: &UiQueryResult,
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
                let selection_lookup = GridSelectionLookup::new(indexes, order);
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
            let selection_lookup = GridSelectionLookup::new(indexes, order);
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

    /// Filter box, copy buttons and the row-count hint above the grid.
    fn draw_grid_toolbar(
        &mut self,
        ui: &mut egui::Ui,
        result: &UiQueryResult,
        editable: bool,
        matching_rows: usize,
        indexes: &[usize],
    ) {
        toolbar_frame(self.theme).show(ui, |ui| {
            ui.horizontal(|ui| {
                input(ui, &mut self.grid_filter, "Filter visible rows…", 200.0, self.theme);
                if !self.grid_filter.is_empty()
                    && compact_icon_button(ui, Icon::X, self.theme)
                        .on_hover_text("Clear filter")
                        .clicked()
                {
                    self.grid_filter.clear();
                }

                crate::components::badge::Badge::new(&format!("{matching_rows} rows"), self.theme)
                    .variant(crate::components::badge::BadgeVariant::Secondary)
                    .compact(true)
                    .show(ui);

                ui.separator();

                if compact_button_with_icon(ui, Icon::Copy, "Copy Cell", self.theme)
                    .on_hover_text("Copy selected cell value (Cmd/Ctrl+C)")
                    .clicked()
                {
                    self.copy_selected_cell(ui, result);
                }
                if compact_button_with_icon(ui, Icon::Table2, "Copy Row", self.theme)
                    .on_hover_text("Copy entire selected row as tab-separated text (Cmd/Ctrl+Shift+C)")
                    .clicked()
                {
                    self.copy_selected_row(ui, result);
                }
                if compact_button_with_icon(ui, Icon::FileSpreadsheet, "CSV", self.theme)
                    .on_hover_text("Copy visible rows as CSV")
                    .clicked()
                {
                    self.copy_all_as_csv(ui, result, indexes);
                }
                if compact_button_with_icon(ui, Icon::Braces, "JSON", self.theme)
                    .on_hover_text("Copy visible rows as JSON array")
                    .clicked()
                {
                    self.copy_all_as_json(ui, result, indexes);
                }

                if !self.copy_status.is_empty() {
                    crate::components::badge::Badge::new(&self.copy_status, self.theme)
                        .variant(crate::components::badge::BadgeVariant::Success)
                        .compact(true)
                        .show(ui);
                }

                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    ui.label(
                        RichText::new(if editable {
                            "Double-click / Enter to edit · Right-click for actions · Drag divider to resize"
                        } else {
                            "Click cell to select · Right-click for actions · Drag divider to resize"
                        })
                        .font(font_caption())
                        .color(self.theme.text_muted),
                    );
                });
            });
        });
        ui.add_space(4.0);
    }

    /// Scrollable grid: continuous spreadsheet header plus visible slice of rows.
    fn draw_grid_body(
        &mut self,
        ui: &mut egui::Ui,
        result: &UiQueryResult,
        indexes: &[usize],
        order: &[usize],
        editable: bool,
        row_offset: u64,
    ) {
        let grid_height = ui.available_height().max(180.0);
        let grid_width = ui.available_width().max(0.0);
        let widths = self.column_widths(result.columns.len(), grid_width);
        let selection_lookup = GridSelectionLookup::new(indexes, order);

        ui.allocate_ui_with_layout(
            egui::vec2(grid_width, grid_height),
            Layout::top_down(Align::Min),
            |ui| {
                ui.spacing_mut().item_spacing = Vec2::ZERO;
                egui::ScrollArea::horizontal().show(ui, |ui| {
                    ui.spacing_mut().item_spacing = Vec2::ZERO;
                    let content_width = GRID_ROW_NUMBER_WIDTH + widths.iter().sum::<f32>();
                    ui.set_min_width(content_width);
                    self.draw_grid_header(ui, result, &widths, order);
                    let rows = GridRows {
                        indexes,
                        widths: &widths,
                        order,
                        editable,
                        row_offset,
                        selection_lookup: &selection_lookup,
                    };
                    let row_height = 28.0;
                    egui::ScrollArea::vertical()
                        .max_height((grid_height - 34.0).max(140.0))
                        .show_rows(ui, row_height, indexes.len(), |ui, range| {
                            ui.spacing_mut().item_spacing = Vec2::ZERO;
                            for position in range {
                                self.draw_grid_row(ui, result, &rows, position);
                            }
                        });
                });
            },
        );
    }

    /// One grid row: the row-number gutter plus every visible cell with continuous borders.
    fn draw_grid_row(&mut self, ui: &mut egui::Ui, result: &UiQueryResult, rows: &GridRows<'_>, position: usize) {
        let row_index = rows.indexes[position];
        let row = &result.rows[row_index];
        let row_selected = self.selected_rows.contains(&row_index);
        let row_dirty = self.staged_row_deleted(result, row_index)
            || (0..result.columns.len())
                .any(|column_index| self.staged_cell_value(result, row_index, column_index).is_some());
        let row_mutation_error = self.mutation_error_for_row(result, row_index);

        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing = Vec2::ZERO;
            let row_number = crate::displayed_row_number(rows.row_offset, row_index);
            let (gutter_rect, gutter_resp) =
                ui.allocate_exact_size(egui::vec2(GRID_ROW_NUMBER_WIDTH, 28.0), Sense::click());

            let gutter_fill = if row_selected {
                self.theme.accent.linear_multiply(0.18)
            } else if gutter_resp.hovered() {
                self.theme.surface_hover.linear_multiply(0.5)
            } else {
                self.theme.surface_panel.linear_multiply(0.5)
            };
            ui.painter().rect_filled(gutter_rect, Rounding::ZERO, gutter_fill);
            ui.painter().hline(
                gutter_rect.x_range(),
                gutter_rect.bottom(),
                Stroke::new(1.0, self.theme.border_subtle.linear_multiply(0.4)),
            );
            ui.painter().vline(
                gutter_rect.right(),
                gutter_rect.y_range(),
                Stroke::new(1.0, self.theme.border_subtle.linear_multiply(0.4)),
            );
            ui.painter().text(
                Pos2::new(gutter_rect.right() - 8.0, gutter_rect.center().y),
                Align2::RIGHT_CENTER,
                row_number.to_string(),
                FontId::monospace(11.0),
                if row_selected {
                    self.theme.accent
                } else {
                    self.theme.text_muted
                },
            );

            if gutter_resp.clicked() {
                if self.data_editing_cell.is_some() && !self.commit_active_data_edit(result) {
                    return;
                }
                self.selected_cell = None;
                let modifiers = ui.input(|input| input.modifiers);
                self.select_visible_row(
                    rows.indexes,
                    &rows.selection_lookup.row_positions,
                    position,
                    modifiers.shift,
                    modifiers.command || modifiers.ctrl,
                );
                self.selection_anchor_cell = None;
                self.data_editing_cell = None;
                self.data_edit_value.clear();
                self.copy_status.clear();
            }

            for &column_index in rows.order {
                let cell = row.get(column_index).unwrap_or(&UiCell::Null);
                let width = rows.widths.get(column_index).copied().unwrap_or(180.0);
                self.draw_grid_cell(
                    ui,
                    result,
                    GridCell {
                        visible_indexes: rows.indexes,
                        selection_lookup: rows.selection_lookup,
                        row_index,
                        column_index,
                        display_position: position,
                        row_selected,
                        row_dirty,
                        row_mutation_error,
                        cell_mutation_error: self.mutation_error_for_cell(result, row_index, column_index),
                        editable: rows.editable,
                        width,
                        cell,
                    },
                );
            }
        });
    }

    /// One grid cell: crisp background, grid borders, active cell highlight, and formatted value.
    fn draw_grid_cell(&mut self, ui: &mut egui::Ui, result: &UiQueryResult, cell_ctx: GridCell<'_>) {
        let GridCell {
            visible_indexes,
            selection_lookup,
            row_index,
            column_index,
            display_position,
            row_selected,
            row_dirty,
            row_mutation_error,
            cell_mutation_error,
            editable,
            width,
            cell,
        } = cell_ctx;

        let staged_cell = self.staged_cell_value(result, row_index, column_index);
        let display_cell = staged_cell.as_ref().unwrap_or(cell);
        let cell_selected = self.is_cell_selected(selection_lookup, (row_index, column_index));
        let editing = editable && self.data_editing_cell == Some((row_index, column_index));
        let validation_error = editing && self.data_edit_error.is_some();
        let conflict_error = (cell_mutation_error || row_mutation_error)
            && self
                .table_mutation_error
                .as_ref()
                .is_some_and(|failure| failure.code == "CONFLICT");

        let (cell_rect, cell_resp) = ui.allocate_exact_size(egui::vec2(width, 28.0), Sense::click());

        let fill = if validation_error || (cell_mutation_error && !conflict_error) {
            self.theme.danger.linear_multiply(0.14)
        } else if conflict_error {
            self.theme.warning.linear_multiply(0.16)
        } else if row_mutation_error {
            self.theme.danger.linear_multiply(0.08)
        } else if row_selected {
            if cell_selected {
                self.theme.accent.linear_multiply(0.20)
            } else {
                self.theme.accent.linear_multiply(0.08)
            }
        } else if cell_selected {
            self.theme.accent.linear_multiply(0.14)
        } else if row_dirty {
            self.theme.warning.linear_multiply(0.12)
        } else if cell_resp.hovered() {
            self.theme.surface_hover.linear_multiply(0.35)
        } else if display_position % 2 == 1 {
            self.theme.surface_panel.linear_multiply(0.25)
        } else {
            self.theme.surface_elevated
        };

        ui.painter().rect_filled(cell_rect, Rounding::ZERO, fill);

        // Bottom and right border lines for continuous spreadsheet appearance
        let border_stroke = Stroke::new(1.0, self.theme.border_subtle.linear_multiply(0.35));
        ui.painter()
            .hline(cell_rect.x_range(), cell_rect.bottom(), border_stroke);
        ui.painter()
            .vline(cell_rect.right(), cell_rect.y_range(), border_stroke);

        if cell_selected {
            ui.painter()
                .rect_stroke(cell_rect, Rounding::ZERO, Stroke::new(1.5, self.theme.accent));
        }
        if validation_error {
            ui.painter().rect_stroke(
                cell_rect.shrink(1.0),
                Rounding::ZERO,
                Stroke::new(1.5, self.theme.danger),
            );
            if let Some(error) = self.data_edit_error.as_deref() {
                cell_resp.clone().on_hover_text(error);
            }
        }
        if cell_mutation_error || row_mutation_error {
            ui.painter().rect_stroke(
                cell_rect.shrink(1.0),
                Rounding::ZERO,
                Stroke::new(
                    1.5,
                    if conflict_error {
                        self.theme.warning
                    } else {
                        self.theme.danger
                    },
                ),
            );
            if let Some(error) = self.table_mutation_error.as_ref() {
                cell_resp.clone().on_hover_text(error.message.as_str());
            }
        }

        if editing {
            self.draw_grid_cell_editor(ui, result, row_index, column_index, cell_rect);
        } else {
            let text_rect = cell_rect.shrink2(egui::vec2(8.0, 3.0));
            let is_null = matches!(display_cell, crate::UiCell::Null);
            let text_val = Self::cell_label(display_cell);
            let font = match display_cell {
                crate::UiCell::Null => FontId::proportional(11.5),
                crate::UiCell::Number(_) | crate::UiCell::Json(_) | crate::UiCell::Bytes(_) => FontId::monospace(11.5),
                _ => FontId::proportional(12.0),
            };
            let text_color = match display_cell {
                crate::UiCell::Null => self.theme.text_muted.linear_multiply(0.55),
                crate::UiCell::Boolean(val) => {
                    if *val {
                        self.theme.success
                    } else {
                        self.theme.danger
                    }
                }
                crate::UiCell::Number(_) => self.theme.text_primary,
                crate::UiCell::Json(_) | crate::UiCell::Bytes(_) => self.theme.text_secondary,
                crate::UiCell::Text(_) => self.theme.text_primary,
            };

            let painter = ui.painter().with_clip_rect(text_rect);
            if is_null {
                // Subtle italic badge for NULL
                let galley = painter.layout_no_wrap(
                    "NULL".to_owned(),
                    FontId::new(11.0, egui::FontFamily::Proportional),
                    text_color,
                );
                painter.galley(
                    Pos2::new(text_rect.left(), text_rect.center().y - galley.size().y * 0.5),
                    galley,
                    text_color,
                );
            } else {
                painter.text(
                    Pos2::new(text_rect.left(), text_rect.center().y),
                    Align2::LEFT_CENTER,
                    text_val,
                    font,
                    text_color,
                );
            }
            let raw_value = crate::cell_text(display_cell);
            if raw_value.chars().count() > 40 {
                cell_resp.clone().on_hover_text(raw_value);
            }

            let is_ctx = is_context_menu_triggered(&cell_resp, ui);
            let mut copy_cell_req = false;
            let mut copy_row_req = false;
            let mut copy_selected_rows_req = false;
            let mut copy_selected_rows_headers_req = false;
            let mut copy_selected_rows_json_req = false;
            let mut copy_selected_rows_insert_req = false;
            let mut copy_json_req = false;
            let mut copy_csv_req = false;
            let mut edit_cell_req = false;
            let mut set_null_req = false;
            let mut revert_cell_req = false;
            let mut revert_row_req = false;
            let mut duplicate_row_req = false;
            let mut delete_row_req = false;
            let mut filter_this_val_req = false;
            let mut sort_asc_req = false;
            let mut sort_desc_req = false;
            let theme = self.theme;
            let modifier = Self::primary_modifier_label();

            context_action_menu(ui, &cell_resp, theme, |ui, close_menu| {
                let copy_sc = format!("{modifier}C");
                if ctx_menu_item(
                    ui,
                    Some(Icon::Copy),
                    "Copy Cell Value",
                    Some(&copy_sc),
                    theme.text_primary,
                    theme,
                )
                .clicked()
                {
                    copy_cell_req = true;
                    *close_menu = true;
                }
                if ctx_menu_item(
                    ui,
                    Some(Icon::Table2),
                    "Copy Row (TSV)",
                    Some(&format!("{modifier}Shift+C")),
                    theme.text_primary,
                    theme,
                )
                .clicked()
                {
                    copy_row_req = true;
                    *close_menu = true;
                }
                if ctx_menu_item(
                    ui,
                    Some(Icon::Rows3),
                    "Copy Selected Rows",
                    None,
                    theme.text_primary,
                    theme,
                )
                .clicked()
                {
                    copy_selected_rows_req = true;
                    *close_menu = true;
                }
                if ctx_menu_item(
                    ui,
                    Some(Icon::Table2),
                    "Copy Selected Rows with Headers",
                    None,
                    theme.text_primary,
                    theme,
                )
                .clicked()
                {
                    copy_selected_rows_headers_req = true;
                    *close_menu = true;
                }
                if ctx_menu_item(
                    ui,
                    Some(Icon::Braces),
                    "Copy Selected Rows as JSON",
                    None,
                    theme.text_primary,
                    theme,
                )
                .clicked()
                {
                    copy_selected_rows_json_req = true;
                    *close_menu = true;
                }
                if ctx_menu_item(
                    ui,
                    Some(Icon::Code),
                    "Copy Selected Rows as INSERT SQL",
                    None,
                    theme.text_primary,
                    theme,
                )
                .clicked()
                {
                    copy_selected_rows_insert_req = true;
                    *close_menu = true;
                }
                if ctx_menu_item(
                    ui,
                    Some(Icon::Braces),
                    "Copy Row as JSON",
                    None,
                    theme.text_primary,
                    theme,
                )
                .clicked()
                {
                    copy_json_req = true;
                    *close_menu = true;
                }
                if ctx_menu_item(
                    ui,
                    Some(Icon::FileSpreadsheet),
                    "Copy Row as CSV",
                    None,
                    theme.text_primary,
                    theme,
                )
                .clicked()
                {
                    copy_csv_req = true;
                    *close_menu = true;
                }

                if editable {
                    ui.separator();
                    if ctx_menu_item(
                        ui,
                        Some(Icon::Pencil),
                        "Edit Cell",
                        Some("Enter / F2"),
                        theme.text_primary,
                        theme,
                    )
                    .clicked()
                    {
                        edit_cell_req = true;
                        *close_menu = true;
                    }
                    if ctx_menu_item(ui, Some(Icon::Eraser), "Set to NULL", None, theme.text_secondary, theme).clicked()
                    {
                        set_null_req = true;
                        *close_menu = true;
                    }
                    if self.staged_cell_value(result, row_index, column_index).is_some()
                        && ctx_menu_item(ui, Some(Icon::Undo2), "Revert Cell", None, theme.text_primary, theme)
                            .clicked()
                    {
                        revert_cell_req = true;
                        *close_menu = true;
                    }
                    let has_row_change = self
                        .row_identity_for_result(result, row_index)
                        .as_ref()
                        .map(|identity| self.staged_changes.row_has_changes(identity))
                        .unwrap_or(false);
                    if has_row_change
                        && ctx_menu_item(
                            ui,
                            Some(Icon::Undo2),
                            if self.staged_row_deleted(result, row_index) {
                                "Undo Delete"
                            } else {
                                "Revert Row"
                            },
                            None,
                            theme.text_primary,
                            theme,
                        )
                        .clicked()
                    {
                        revert_row_req = true;
                        *close_menu = true;
                    }
                    if ctx_menu_item(
                        ui,
                        Some(Icon::CopyPlus),
                        "Duplicate Row",
                        None,
                        theme.text_primary,
                        theme,
                    )
                    .clicked()
                    {
                        duplicate_row_req = true;
                        *close_menu = true;
                    }
                    if ctx_menu_item(
                        ui,
                        Some(Icon::Trash2),
                        "Delete Row",
                        Some("Delete / Backspace"),
                        theme.danger,
                        theme,
                    )
                    .clicked()
                    {
                        delete_row_req = true;
                        *close_menu = true;
                    }
                }

                ui.separator();
                if ctx_menu_item(
                    ui,
                    Some(Icon::Filter),
                    "Filter by this value",
                    None,
                    theme.text_primary,
                    theme,
                )
                .clicked()
                {
                    filter_this_val_req = true;
                    *close_menu = true;
                }
                if ctx_menu_item(
                    ui,
                    Some(Icon::ArrowUp),
                    "Sort Ascending",
                    None,
                    theme.text_primary,
                    theme,
                )
                .clicked()
                {
                    sort_asc_req = true;
                    *close_menu = true;
                }
                if ctx_menu_item(
                    ui,
                    Some(Icon::ArrowDown),
                    "Sort Descending",
                    None,
                    theme.text_primary,
                    theme,
                )
                .clicked()
                {
                    sort_desc_req = true;
                    *close_menu = true;
                }
            });

            if is_ctx
                && self
                    .data_editing_cell
                    .is_some_and(|editing_cell| editing_cell != (row_index, column_index))
                && !self.commit_active_data_edit(result)
            {
                return;
            }

            if is_ctx {
                // Keep an existing rectangular selection when the context menu
                // is opened inside it. Right-clicking outside the range starts
                // a new selection at the clicked cell.
                let is_inside_range = self.is_cell_selected(selection_lookup, (row_index, column_index));
                if !is_inside_range {
                    self.select_single_cell((row_index, column_index));
                }
                if !self.selected_rows.contains(&row_index) {
                    self.select_single_row(row_index);
                } else if !is_inside_range {
                    self.selected_row = Some(row_index);
                    self.selection_anchor_row = Some(row_index);
                }
            }
            if copy_cell_req {
                self.copy_cell_at(ui, result, row_index, column_index);
            }
            if copy_row_req {
                self.select_single_row(row_index);
                self.copy_selected_row(ui, result);
            }
            if copy_selected_rows_req {
                if !self.selected_rows.contains(&row_index) {
                    self.select_single_row(row_index);
                }
                self.copy_selected_rows(ui, result);
            }
            if copy_selected_rows_headers_req {
                if !self.selected_rows.contains(&row_index) {
                    self.select_single_row(row_index);
                }
                self.copy_selected_rows_with_headers(ui, result);
            }
            if copy_selected_rows_json_req {
                if !self.selected_rows.contains(&row_index) {
                    self.select_single_row(row_index);
                }
                self.copy_selected_rows_as_json(ui, result);
            }
            if copy_selected_rows_insert_req {
                if !self.selected_rows.contains(&row_index) {
                    self.select_single_row(row_index);
                }
                self.copy_selected_rows_as_insert(ui, result);
            }
            if copy_json_req {
                self.selected_row = Some(row_index);
                self.copy_row_as_json(ui, result, row_index);
            }
            if copy_csv_req {
                self.selected_row = Some(row_index);
                self.copy_row_as_csv(ui, result, row_index);
            }
            if edit_cell_req && editable {
                self.begin_data_cell_edit(result, row_index, column_index, display_cell);
            }
            if set_null_req && editable {
                self.data_editing_cell = Some((row_index, column_index));
                self.data_edit_value = "NULL".to_owned();
                self.data_edit_error = None;
                self.submit_data_cell_edit(result, row_index, column_index);
            }
            if revert_cell_req && editable {
                self.revert_staged_cell(result, row_index, column_index);
            }
            if revert_row_req && editable {
                self.revert_staged_row(result, row_index);
            }
            if duplicate_row_req && editable {
                self.open_duplicate_row(result, row_index);
            }
            if delete_row_req && editable {
                if !self.selected_rows.contains(&row_index) {
                    self.select_single_row(row_index);
                }
                self.request_delete_selected_data_rows(result);
            }
            if filter_this_val_req {
                if self.active_tab == WorkspaceTab::Table && self.table_view == TableView::Data {
                    self.table_data_filter_column = result
                        .columns
                        .get(column_index)
                        .map(|column| column.name.clone())
                        .unwrap_or_default();
                    if matches!(display_cell, UiCell::Null) {
                        self.table_data_filter_operator = UiTableFilterOperator::IsNull;
                        self.table_data_filter_value.clear();
                    } else {
                        self.table_data_filter_operator = UiTableFilterOperator::Equals;
                        self.table_data_filter_value = crate::cell_text(display_cell);
                    }
                    self.commit_table_filter_draft();
                } else {
                    self.grid_filter = crate::cell_text(display_cell);
                }
            }
            if sort_asc_req {
                self.set_table_or_grid_sort(result, column_index, Some(false));
            }
            if sort_desc_req {
                self.set_table_or_grid_sort(result, column_index, Some(true));
            }

            if cell_resp.double_clicked() && editable {
                if self.data_editing_cell.is_some() && !self.commit_active_data_edit(result) {
                    return;
                }
                self.begin_data_cell_edit(result, row_index, column_index, display_cell);
            } else if cell_resp.clicked() && !is_ctx {
                if self.data_editing_cell.is_some() && !self.commit_active_data_edit(result) {
                    return;
                }
                let modifiers = ui.input(|input| input.modifiers);
                self.select_cell_range(
                    visible_indexes,
                    &selection_lookup.row_positions,
                    (row_index, column_index),
                    modifiers.shift,
                );
                self.copy_status.clear();
            }
        }
    }

    /// Inline text editor for the cell currently being edited.
    fn draw_grid_cell_editor(
        &mut self,
        ui: &mut egui::Ui,
        result: &UiQueryResult,
        row_index: usize,
        column_index: usize,
        cell_rect: Rect,
    ) {
        let is_boolean = result
            .columns
            .get(column_index)
            .is_some_and(|column| column.data_type.to_ascii_lowercase().contains("bool"));
        let is_expanded = self.expanded_data_editor == Some((row_index, column_index));
        if !is_expanded {
            ui.allocate_new_ui(egui::UiBuilder::new().max_rect(cell_rect.shrink(1.0)), |ui| {
                let response = if is_boolean {
                    let mut checked = self.data_edit_value.eq_ignore_ascii_case("true");
                    let response = ui.checkbox(&mut checked, "");
                    if response.changed() {
                        self.data_edit_value = checked.to_string();
                        self.data_edit_error = None;
                    }
                    response
                } else {
                    ui.add_sized(
                        ui.available_size(),
                        TextEdit::singleline(&mut self.data_edit_value)
                            .margin(egui::Margin::symmetric(6.0, 2.0))
                            .text_color(self.theme.text_primary),
                    )
                };
                response.request_focus();
                if response.changed() {
                    self.data_edit_error = None;
                }
            });
            let commit = ui.input(|input| input.key_pressed(egui::Key::Enter));
            if commit {
                self.submit_data_cell_edit(result, row_index, column_index);
            } else if ui.input(|input| input.key_pressed(egui::Key::Escape)) {
                self.data_editing_cell = None;
                self.expanded_data_editor = None;
                self.data_edit_value.clear();
                self.data_edit_error = None;
            }
            return;
        }

        let ctx = ui.ctx().clone();
        let mut open = true;
        let mut commit = false;
        let mut cancel = false;
        egui::Window::new("Expanded cell editor")
            .id(egui::Id::new(("expanded-table-cell-editor", row_index, column_index)))
            .open(&mut open)
            .resizable(true)
            .default_width(520.0)
            .show(&ctx, |ui| {
                ui.label(
                    RichText::new("Edit value · Enter applies, Escape cancels")
                        .small()
                        .color(self.theme.text_muted),
                );
                let response = ui.add(
                    TextEdit::multiline(&mut self.data_edit_value)
                        .desired_width(ui.available_width())
                        .desired_rows(14),
                );
                if response.changed() {
                    self.data_edit_error = None;
                }
                if let Some(error) = self.data_edit_error.as_deref() {
                    ui.label(RichText::new(error).small().color(self.theme.danger));
                }
                ui.horizontal(|ui| {
                    if ui.button("Apply").clicked() {
                        commit = true;
                    }
                    if ui.button("Cancel").clicked() {
                        cancel = true;
                    }
                });
            });
        if commit {
            self.submit_data_cell_edit(result, row_index, column_index);
        } else if cancel || !open || ctx.input(|input| input.key_pressed(egui::Key::Escape)) {
            self.data_editing_cell = None;
            self.expanded_data_editor = None;
            self.data_edit_value.clear();
            self.data_edit_error = None;
        }
    }

    fn commit_active_data_edit(&mut self, result: &UiQueryResult) -> bool {
        if let Some((row_index, column_index)) = self.data_editing_cell {
            self.submit_data_cell_edit(result, row_index, column_index)
        } else {
            true
        }
    }

    fn copy_selected_cell(&mut self, ui: &mut egui::Ui, result: &UiQueryResult) {
        let Some((row_index, column_index)) = self.selected_cell else {
            self.copy_status = "Select a cell first".to_owned();
            return;
        };
        self.copy_cell_at(ui, result, row_index, column_index);
    }

    fn copy_cell_at(&mut self, ui: &mut egui::Ui, result: &UiQueryResult, row_index: usize, column_index: usize) {
        let Some(cell) = self.copy_cell_value(result, row_index, column_index) else {
            self.copy_status = "Selected cell is no longer available".to_owned();
            return;
        };
        ui.output_mut(|output| output.copied_text = crate::cell_text(&cell));
        self.copy_status = "Cell copied".to_owned();
    }

    /// One row as a delimited line, with each cell escaped so an embedded tab,
    /// quote or newline cannot be mistaken for a field or row boundary.
    fn copied_row_text(&self, result: &UiQueryResult, row_index: usize, row: &[UiCell]) -> String {
        (0..result.columns.len())
            .map(|column_index| {
                self.copy_cell_value(result, row_index, column_index)
                    .unwrap_or_else(|| row.get(column_index).cloned().unwrap_or(UiCell::Null))
            })
            .map(|cell| Self::format_cell_delimited(&cell, '\t'))
            .collect::<Vec<_>>()
            .join("\t")
    }

    fn copy_selected_row(&mut self, ui: &mut egui::Ui, result: &UiQueryResult) {
        let Some(row_index) = self.selected_row else {
            self.copy_status = "Select a row first".to_owned();
            return;
        };
        let Some(row) = result.rows.get(row_index) else {
            self.copy_status = "Selected row is no longer available".to_owned();
            return;
        };
        let row_text = self.copied_row_text(result, row_index, row);
        ui.output_mut(|output| output.copied_text = row_text);
        self.copy_status = "Row copied".to_owned();
    }

    fn copy_selected_rows(&mut self, ui: &mut egui::Ui, result: &UiQueryResult) {
        let row_indexes = self.selected_row_indexes();
        if row_indexes.is_empty() {
            self.copy_status = "Select one or more rows first".to_owned();
            return;
        }

        let rows = row_indexes
            .iter()
            .filter_map(|&row_index| result.rows.get(row_index).map(|row| (row_index, row)))
            .map(|(row_index, row)| self.copied_row_text(result, row_index, row))
            .collect::<Vec<_>>();
        ui.output_mut(|output| output.copied_text = rows.join("\n"));
        self.copy_status = format!("{} rows copied", rows.len());
    }

    fn selected_row_indexes(&self) -> Vec<usize> {
        if self.selected_rows.is_empty() {
            self.selected_row.into_iter().collect()
        } else {
            self.selected_rows.iter().copied().collect()
        }
    }

    fn copy_selected_rows_with_headers(&mut self, ui: &mut egui::Ui, result: &UiQueryResult) {
        let row_indexes = self.selected_row_indexes();
        if row_indexes.is_empty() {
            self.copy_status = "Select one or more rows first".to_owned();
            return;
        }
        let header = result
            .columns
            .iter()
            .map(|column| Self::escape_delimited_field(&column.name, '\t'))
            .collect::<Vec<_>>()
            .join("\t");
        let rows = row_indexes
            .iter()
            .filter_map(|&row_index| result.rows.get(row_index).map(|row| (row_index, row)))
            .map(|(row_index, row)| self.copied_row_text(result, row_index, row))
            .collect::<Vec<_>>();
        let mut lines = Vec::with_capacity(rows.len() + 1);
        lines.push(header);
        lines.extend(rows);
        ui.output_mut(|output| output.copied_text = lines.join("\n"));
        self.copy_status = format!("{} rows copied with headers", row_indexes.len());
    }

    fn copy_selected_rows_as_json(&mut self, ui: &mut egui::Ui, result: &UiQueryResult) {
        let indexes = self.selected_row_indexes();
        self.copy_all_as_json(ui, result, &indexes);
    }

    fn copy_selected_rows_as_insert(&mut self, ui: &mut egui::Ui, result: &UiQueryResult) {
        let indexes = self.selected_row_indexes();
        if indexes.is_empty() {
            self.copy_status = "Select one or more rows first".to_owned();
            return;
        }
        let table = self.selected_table.as_deref().unwrap_or("table_name");
        let target = if self.active_tab == WorkspaceTab::Table && self.table_view == TableView::Data {
            format!(
                "{}.{}",
                Self::quote_sql_identifier(self.active_schema()),
                Self::quote_sql_identifier(table)
            )
        } else {
            Self::quote_sql_identifier(table)
        };
        let columns = result
            .columns
            .iter()
            .map(|column| Self::quote_sql_identifier(&column.name))
            .collect::<Vec<_>>()
            .join(", ");
        let statements = indexes
            .iter()
            .filter_map(|&row_index| result.rows.get(row_index).map(|row| (row_index, row)))
            .map(|(row_index, row)| {
                let values = (0..result.columns.len())
                    .map(|column_index| {
                        let cell = self
                            .copy_cell_value(result, row_index, column_index)
                            .unwrap_or_else(|| row.get(column_index).cloned().unwrap_or(UiCell::Null));
                        Self::cell_sql_literal(&cell)
                    })
                    .collect::<Vec<_>>()
                    .join(", ");
                format!("INSERT INTO {target} ({columns}) VALUES ({values});")
            })
            .collect::<Vec<_>>();
        ui.output_mut(|output| output.copied_text = statements.join("\n"));
        self.copy_status = format!("{} INSERT statements copied", statements.len());
    }

    fn quote_sql_identifier(identifier: &str) -> String {
        format!("\"{}\"", identifier.replace('"', "\"\""))
    }

    fn cell_sql_literal(cell: &UiCell) -> String {
        match cell {
            UiCell::Null => "NULL".to_owned(),
            UiCell::Boolean(value) => value.to_string().to_uppercase(),
            UiCell::Number(value) if value.parse::<f64>().is_ok() => value.clone(),
            UiCell::Json(value) => format!("'{}'", value.replace('\'', "''")),
            UiCell::Bytes(value) => format!("'{}'", value.replace('\'', "''")),
            UiCell::Number(value) | UiCell::Text(value) => format!("'{}'", value.replace('\'', "''")),
        }
    }

    pub(crate) fn copy_row_as_json(&mut self, ui: &mut egui::Ui, result: &UiQueryResult, row_index: usize) {
        let Some(row) = result.rows.get(row_index) else {
            return;
        };
        let mut map = serde_json::Map::new();
        for (col_idx, col) in result.columns.iter().enumerate() {
            let cell = self
                .copy_cell_value(result, row_index, col_idx)
                .unwrap_or_else(|| row.get(col_idx).cloned().unwrap_or(UiCell::Null));
            map.insert(col.name.clone(), Self::cell_to_json_value(&cell));
        }
        let json_text = serde_json::to_string_pretty(&serde_json::Value::Object(map)).unwrap_or_default();
        ui.output_mut(|output| output.copied_text = json_text);
        self.copy_status = "Row copied as JSON".to_owned();
    }

    pub(crate) fn copy_row_as_csv(&mut self, ui: &mut egui::Ui, result: &UiQueryResult, row_index: usize) {
        let Some(row) = result.rows.get(row_index) else {
            return;
        };
        let header = result
            .columns
            .iter()
            .map(|c| {
                if c.name.contains(',') || c.name.contains('"') {
                    format!("\"{}\"", c.name.replace('"', "\"\""))
                } else {
                    c.name.clone()
                }
            })
            .collect::<Vec<_>>()
            .join(",");
        let row_values = (0..result.columns.len())
            .map(|col_idx| {
                let cell = self
                    .copy_cell_value(result, row_index, col_idx)
                    .unwrap_or_else(|| row.get(col_idx).cloned().unwrap_or(UiCell::Null));
                Self::format_cell_csv(&cell)
            })
            .collect::<Vec<_>>()
            .join(",");
        let csv_text = format!("{header}\n{row_values}");
        ui.output_mut(|output| output.copied_text = csv_text);
        self.copy_status = "Row copied as CSV".to_owned();
    }

    pub(crate) fn copy_all_as_csv(&mut self, ui: &mut egui::Ui, result: &UiQueryResult, indexes: &[usize]) {
        let header = result
            .columns
            .iter()
            .map(|c| {
                if c.name.contains(',') || c.name.contains('"') {
                    format!("\"{}\"", c.name.replace('"', "\"\""))
                } else {
                    c.name.clone()
                }
            })
            .collect::<Vec<_>>()
            .join(",");
        let mut lines = vec![header];
        for &row_index in indexes {
            if let Some(row) = result.rows.get(row_index) {
                let row_values = (0..result.columns.len())
                    .map(|col_idx| {
                        let cell = self
                            .copy_cell_value(result, row_index, col_idx)
                            .unwrap_or_else(|| row.get(col_idx).cloned().unwrap_or(UiCell::Null));
                        Self::format_cell_csv(&cell)
                    })
                    .collect::<Vec<_>>()
                    .join(",");
                lines.push(row_values);
            }
        }
        ui.output_mut(|output| output.copied_text = lines.join("\n"));
        self.copy_status = format!("{} rows copied as CSV", indexes.len());
    }

    pub(crate) fn copy_all_as_json(&mut self, ui: &mut egui::Ui, result: &UiQueryResult, indexes: &[usize]) {
        let mut rows_arr = Vec::new();
        for &row_index in indexes {
            if let Some(row) = result.rows.get(row_index) {
                let mut map = serde_json::Map::new();
                for (col_idx, col) in result.columns.iter().enumerate() {
                    let cell = self
                        .copy_cell_value(result, row_index, col_idx)
                        .unwrap_or_else(|| row.get(col_idx).cloned().unwrap_or(UiCell::Null));
                    map.insert(col.name.clone(), Self::cell_to_json_value(&cell));
                }
                rows_arr.push(serde_json::Value::Object(map));
            }
        }
        let json_text = serde_json::to_string_pretty(&serde_json::Value::Array(rows_arr)).unwrap_or_default();
        ui.output_mut(|output| output.copied_text = json_text);
        self.copy_status = format!("{} rows copied as JSON", indexes.len());
    }

    /// Quote a value so that the delimiter, quotes and line breaks inside it cannot
    /// change the shape of the copy/export.
    pub(crate) fn escape_delimited_field(value: &str, delimiter: char) -> String {
        if value.contains(delimiter) || value.contains('"') || value.contains('\n') || value.contains('\r') {
            format!("\"{}\"", value.replace('"', "\"\""))
        } else {
            value.to_owned()
        }
    }

    /// One `UiCell` as a single delimited field. Numbers are emitted verbatim so the
    /// exact digits survive; `Null` is an empty field, which is how both CSV and TSV
    /// represent a missing value.
    pub(crate) fn format_cell_delimited(cell: &crate::UiCell, delimiter: char) -> String {
        match cell {
            crate::UiCell::Null => String::new(),
            crate::UiCell::Boolean(b) => b.to_string(),
            crate::UiCell::Number(n) => n.clone(),
            crate::UiCell::Text(t) => Self::escape_delimited_field(t, delimiter),
            crate::UiCell::Json(j) => Self::escape_delimited_field(j, delimiter),
            crate::UiCell::Bytes(b) => Self::escape_delimited_field(b, delimiter),
        }
    }

    pub(crate) fn format_cell_csv(cell: &crate::UiCell) -> String {
        Self::format_cell_delimited(cell, ',')
    }

    /// The exact text written by the query-view export for one delimiter.
    pub(crate) fn format_result_delimited(result: &UiQueryResult, delimiter: &str) -> String {
        let separator = delimiter.chars().next().unwrap_or(',');
        let mut output = result
            .columns
            .iter()
            .map(|column| Self::escape_delimited_field(&column.name, separator))
            .collect::<Vec<_>>()
            .join(delimiter);
        output.push('\n');
        for row in &result.rows {
            output.push_str(
                &row.iter()
                    .map(|cell| Self::format_cell_delimited(cell, separator))
                    .collect::<Vec<_>>()
                    .join(delimiter),
            );
            output.push('\n');
        }
        output
    }

    /// Copy-as-JSON must not lose a value. An integer inside the range every JSON
    /// reader represents exactly is written as a JSON integer; any other number is
    /// written as a JSON number only when the decimal text is exactly the shortest form
    /// of the parsed `f64`. Everything else (an integer past 2^53, an exact decimal with
    /// trailing zeroes) keeps its exact digits as a JSON string — the same choice the
    /// provider-value contract already makes for decimals — instead of being rounded
    /// through `f64`.
    pub(crate) fn cell_to_json_value(cell: &crate::UiCell) -> serde_json::Value {
        const JSON_EXACT_INTEGER_LIMIT: i64 = 1 << 53;
        match cell {
            crate::UiCell::Null => serde_json::Value::Null,
            crate::UiCell::Boolean(b) => serde_json::Value::Bool(*b),
            crate::UiCell::Number(n) => match n.parse::<i64>() {
                Ok(i) if (-JSON_EXACT_INTEGER_LIMIT..=JSON_EXACT_INTEGER_LIMIT).contains(&i) => {
                    serde_json::Value::Number(i.into())
                }
                _ => match n.parse::<f64>() {
                    Ok(f) if f.is_finite() && f.to_string() == *n => serde_json::Number::from_f64(f)
                        .map(serde_json::Value::Number)
                        .unwrap_or_else(|| serde_json::Value::String(n.clone())),
                    _ => serde_json::Value::String(n.clone()),
                },
            },
            crate::UiCell::Text(t) => serde_json::Value::String(t.clone()),
            crate::UiCell::Json(j) => serde_json::from_str(j).unwrap_or_else(|_| serde_json::Value::String(j.clone())),
            crate::UiCell::Bytes(b) => serde_json::Value::String(b.clone()),
        }
    }

    pub(crate) fn copy_cell_value(
        &self,
        result: &UiQueryResult,
        row_index: usize,
        column_index: usize,
    ) -> Option<crate::UiCell> {
        let cell = result.rows.get(row_index).and_then(|row| row.get(column_index))?;
        if self.active_tab == WorkspaceTab::Table && self.table_view == TableView::Data {
            Some(
                self.staged_cell_value(result, row_index, column_index)
                    .unwrap_or_else(|| cell.clone()),
            )
        } else {
            Some(cell.clone())
        }
    }

    pub(super) fn column_widths(&mut self, count: usize, available_width: f32) -> Vec<f32> {
        if self.grid_column_widths.len() != count {
            self.grid_column_widths = vec![180.0; count];
            self.grid_columns_user_resized = false;
        }
        let mut widths = self.grid_column_widths.clone();
        if count > 0 && !self.grid_columns_user_resized {
            let usable_width = (available_width - GRID_ROW_NUMBER_WIDTH - 4.0 * count as f32).max(0.0);
            let default_width = (usable_width / count as f32).clamp(180.0, 520.0);
            widths.fill(default_width);
        }
        widths
    }

    fn draw_grid_header(&mut self, ui: &mut egui::Ui, result: &UiQueryResult, widths: &[f32], order: &[usize]) {
        let (mut move_left_req, mut move_right_req, mut reset_order_req, mut reset_widths_req) =
            (None, None, false, false);
        let (mut hide_column_req, mut show_columns_req, mut reset_layout_req, mut auto_size_req) =
            (None, false, false, None);
        let mut add_filter_req = None;

        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing = Vec2::ZERO;
            let (gutter_rect, _) = ui.allocate_exact_size(egui::vec2(GRID_ROW_NUMBER_WIDTH, 34.0), Sense::hover());
            ui.painter()
                .rect_filled(gutter_rect, Rounding::ZERO, self.theme.surface_panel);
            ui.painter().hline(
                gutter_rect.x_range(),
                gutter_rect.bottom(),
                Stroke::new(1.0, self.theme.border_subtle.linear_multiply(0.7)),
            );
            ui.painter().vline(
                gutter_rect.right(),
                gutter_rect.y_range(),
                Stroke::new(1.0, self.theme.border_subtle.linear_multiply(0.7)),
            );
            ui.painter().text(
                gutter_rect.center(),
                Align2::CENTER_CENTER,
                "#",
                FontId::monospace(11.5),
                self.theme.text_muted,
            );

            for (visual_idx, &col_idx) in order.iter().enumerate() {
                let Some(column) = result.columns.get(col_idx) else {
                    continue;
                };
                let width = widths.get(col_idx).copied().unwrap_or(180.0);
                let (col_rect, col_resp) = ui.allocate_exact_size(egui::vec2(width, 34.0), Sense::click());

                // Check PK / FK indicators
                let is_pk = self
                    .table_info
                    .as_ref()
                    .is_some_and(|info| info.columns.iter().any(|c| c.name == column.name && c.is_primary_key));
                let is_fk = self.table_info.as_ref().is_some_and(|info| {
                    info.foreign_keys
                        .iter()
                        .any(|fk| fk.from_columns.iter().any(|col| col == &column.name))
                });

                // Resize divider on the right edge (4px grab target)
                let divider_rect = Rect::from_min_max(
                    Pos2::new(col_rect.right() - 3.0, col_rect.top()),
                    Pos2::new(col_rect.right() + 3.0, col_rect.bottom()),
                );
                let divider_id = ui.id().with(("grid_col_resize", col_idx));
                let divider = ui.interact(divider_rect, divider_id, Sense::drag());
                let is_resizing = divider.hovered() || divider.dragged();
                if is_resizing {
                    ui.ctx().set_cursor_icon(egui::CursorIcon::ResizeHorizontal);
                }

                let col_hovered = col_resp.hovered() && !is_resizing;
                let bg_fill = if col_hovered {
                    self.theme.surface_hover.linear_multiply(0.4)
                } else {
                    self.theme.surface_panel
                };
                ui.painter().rect_filled(col_rect, Rounding::ZERO, bg_fill);

                // Bottom and right border lines
                ui.painter().hline(
                    col_rect.x_range(),
                    col_rect.bottom(),
                    Stroke::new(1.0, self.theme.border_subtle.linear_multiply(0.7)),
                );
                ui.painter().vline(
                    col_rect.right(),
                    col_rect.y_range(),
                    Stroke::new(
                        if divider.dragged() { 2.0 } else { 1.0 },
                        if is_resizing {
                            self.theme.accent
                        } else {
                            self.theme.border_subtle.linear_multiply(0.7)
                        },
                    ),
                );

                // Column title + data type + sort icon
                let table_sort = (self.active_tab == WorkspaceTab::Table && self.table_view == TableView::Data)
                    .then(|| self.table_data_sorts.iter().find(|sort| sort.column == column.name))
                    .flatten();
                let table_sort_priority = table_sort.and_then(|_| {
                    self.table_data_sorts
                        .iter()
                        .position(|sort| sort.column == column.name)
                        .map(|position| position + 1)
                });
                let table_sort_active = table_sort.is_some();
                let sort_active = table_sort_active || self.grid_sort_column == Some(col_idx);
                let sort_desc = if let Some(sort) = table_sort {
                    sort.descending
                } else {
                    self.grid_sort_desc
                };
                let sort_marker = if let Some(priority) = table_sort_priority {
                    format!(" {}{}", if sort_desc { "↓" } else { "↑" }, priority)
                } else if sort_active {
                    format!(" {}", if sort_desc { "↓" } else { "↑" })
                } else {
                    String::new()
                };

                let header_text_rect = col_rect.shrink2(egui::vec2(8.0, 4.0));
                let painter = ui.painter().with_clip_rect(header_text_rect);

                let mut text_x = header_text_rect.left();

                // Draw PK / FK badges
                if is_pk {
                    let pk_galley = painter.layout_no_wrap(
                        "PK".to_owned(),
                        FontId::new(9.5, egui::FontFamily::Proportional),
                        self.theme.warning,
                    );
                    let badge_rect = Rect::from_min_size(
                        Pos2::new(text_x, header_text_rect.center().y - 7.0),
                        Vec2::new(pk_galley.size().x + 6.0, 14.0),
                    );
                    painter.rect_filled(
                        badge_rect,
                        Rounding::same(3.0),
                        self.theme.warning.linear_multiply(0.18),
                    );
                    painter.galley(
                        Pos2::new(
                            badge_rect.left() + 3.0,
                            badge_rect.center().y - pk_galley.size().y * 0.5,
                        ),
                        pk_galley,
                        self.theme.warning,
                    );
                    text_x += badge_rect.width() + 4.0;
                } else if is_fk {
                    let fk_galley = painter.layout_no_wrap(
                        "FK".to_owned(),
                        FontId::new(9.5, egui::FontFamily::Proportional),
                        self.theme.accent,
                    );
                    let badge_rect = Rect::from_min_size(
                        Pos2::new(text_x, header_text_rect.center().y - 7.0),
                        Vec2::new(fk_galley.size().x + 6.0, 14.0),
                    );
                    painter.rect_filled(badge_rect, Rounding::same(3.0), self.theme.accent.linear_multiply(0.18));
                    painter.galley(
                        Pos2::new(
                            badge_rect.left() + 3.0,
                            badge_rect.center().y - fk_galley.size().y * 0.5,
                        ),
                        fk_galley,
                        self.theme.accent,
                    );
                    text_x += badge_rect.width() + 4.0;
                }

                // Draw column name
                let col_name_galley = painter.layout_no_wrap(
                    column.name.clone(),
                    DbProTheme::ui_medium_font(12.5),
                    self.theme.text_primary,
                );
                let name_width = col_name_galley.size().x;
                painter.galley(
                    Pos2::new(text_x, header_text_rect.center().y - col_name_galley.size().y * 0.5),
                    col_name_galley,
                    self.theme.text_primary,
                );

                // Draw column type
                let type_text = format!(" {}{}", column.data_type, sort_marker);
                let type_galley = painter.layout_no_wrap(
                    type_text,
                    FontId::monospace(10.5),
                    if sort_active {
                        self.theme.accent
                    } else {
                        self.theme.text_muted
                    },
                );
                painter.galley(
                    Pos2::new(
                        text_x + name_width + 4.0,
                        header_text_rect.center().y - type_galley.size().y * 0.5,
                    ),
                    type_galley,
                    if sort_active {
                        self.theme.accent
                    } else {
                        self.theme.text_muted
                    },
                );

                // Header right-click context menu
                let theme = self.theme;
                let is_sorted = sort_active;
                context_action_menu(ui, &col_resp, theme, |ui, close_menu| {
                    if ctx_menu_item(
                        ui,
                        Some(Icon::ArrowUp),
                        "Sort Ascending (A → Z)",
                        None,
                        theme.text_primary,
                        theme,
                    )
                    .clicked()
                    {
                        self.set_table_or_grid_sort(result, col_idx, Some(false));
                        *close_menu = true;
                    }
                    if ctx_menu_item(
                        ui,
                        Some(Icon::ArrowDown),
                        "Sort Descending (Z → A)",
                        None,
                        theme.text_primary,
                        theme,
                    )
                    .clicked()
                    {
                        self.set_table_or_grid_sort(result, col_idx, Some(true));
                        *close_menu = true;
                    }
                    if is_sorted
                        && ctx_menu_item(ui, Some(Icon::X), "Clear Sort", None, theme.text_secondary, theme).clicked()
                    {
                        self.set_table_or_grid_sort(result, col_idx, None);
                        *close_menu = true;
                    }
                    if self.active_tab == WorkspaceTab::Table
                        && self.table_view == TableView::Data
                        && ctx_menu_item(ui, Some(Icon::Filter), "Add Filter", None, theme.text_primary, theme)
                            .clicked()
                    {
                        add_filter_req = Some(col_idx);
                        *close_menu = true;
                    }
                    ui.separator();
                    if visual_idx > 0
                        && ctx_menu_item(
                            ui,
                            Some(Icon::ArrowLeft),
                            "Move Column Left",
                            None,
                            theme.text_primary,
                            theme,
                        )
                        .clicked()
                    {
                        move_left_req = Some(visual_idx);
                        *close_menu = true;
                    }
                    if visual_idx + 1 < order.len()
                        && ctx_menu_item(
                            ui,
                            Some(Icon::ArrowRight),
                            "Move Column Right",
                            None,
                            theme.text_primary,
                            theme,
                        )
                        .clicked()
                    {
                        move_right_req = Some(visual_idx);
                        *close_menu = true;
                    }
                    ui.separator();
                    if ctx_menu_item(
                        ui,
                        Some(Icon::RotateCcw),
                        "Reset Column Order",
                        None,
                        theme.text_secondary,
                        theme,
                    )
                    .clicked()
                    {
                        reset_order_req = true;
                        *close_menu = true;
                    }
                    if ctx_menu_item(
                        ui,
                        Some(Icon::Maximize2),
                        "Reset Column Widths",
                        None,
                        theme.text_secondary,
                        theme,
                    )
                    .clicked()
                    {
                        reset_widths_req = true;
                        *close_menu = true;
                    }
                    if ctx_menu_item(ui, Some(Icon::EyeOff), "Hide Column", None, theme.text_secondary, theme).clicked()
                    {
                        hide_column_req = Some(col_idx);
                        *close_menu = true;
                    }
                    if !self.grid_hidden_columns.is_empty()
                        && ctx_menu_item(ui, Some(Icon::Eye), "Show Columns", None, theme.text_secondary, theme)
                            .clicked()
                    {
                        show_columns_req = true;
                        *close_menu = true;
                    }
                    if ctx_menu_item(
                        ui,
                        Some(Icon::RotateCcw),
                        "Reset Layout",
                        None,
                        theme.text_secondary,
                        theme,
                    )
                    .clicked()
                    {
                        reset_layout_req = true;
                        *close_menu = true;
                    }
                    if ctx_menu_item(ui, Some(Icon::Ruler), "Auto Size", None, theme.text_secondary, theme).clicked() {
                        auto_size_req = Some(col_idx);
                        *close_menu = true;
                    }
                });

                if col_resp.clicked() && !divider.dragged() {
                    if self.active_tab == WorkspaceTab::Table && self.table_view == TableView::Data {
                        self.cycle_table_data_sort(result, col_idx, ui.input(|input| input.modifiers.shift));
                    } else if self.grid_sort_column == Some(col_idx) {
                        self.set_table_or_grid_sort(
                            result,
                            col_idx,
                            if self.grid_sort_desc { None } else { Some(true) },
                        );
                    } else {
                        self.set_table_or_grid_sort(result, col_idx, Some(false));
                    }
                }

                if divider.drag_started() {
                    self.grid_column_widths = widths.to_vec();
                    self.grid_columns_user_resized = true;
                }
                if divider.dragged() {
                    self.grid_column_widths[col_idx] =
                        (self.grid_column_widths[col_idx] + divider.drag_delta().x).clamp(60.0, 1000.0);
                }
                if divider.double_clicked() {
                    auto_size_req = Some(col_idx);
                }
            }
        });

        if let Some(idx) = move_left_req {
            if idx > 0 {
                self.move_column(idx, idx - 1, order.len());
            }
        }
        if let Some(idx) = move_right_req {
            if idx + 1 < order.len() {
                self.move_column(idx, idx + 1, order.len());
            }
        }
        if reset_order_req {
            self.grid_column_order = (0..result.columns.len()).collect();
        }
        if reset_widths_req {
            self.grid_columns_user_resized = false;
        }
        if let Some(column_index) = hide_column_req {
            self.hide_column(column_index, order.len());
        }
        if let Some(column_index) = add_filter_req {
            if let Some(column) = result.columns.get(column_index) {
                self.table_data_filter_column = column.name.clone();
                self.table_data_filter_operator = UiTableFilterOperator::Equals;
                self.table_data_filter_value.clear();
                self.table_data_filter_editing = None;
                self.runtime_message = format!("Filter draft ready for {}", column.name);
            }
        }
        if show_columns_req {
            self.show_all_columns();
        }
        if reset_layout_req {
            self.reset_grid_layout(result.columns.len());
        }
        if let Some(column_index) = auto_size_req {
            let indexes =
                crate::filtered_sorted_indexes(result, &self.grid_filter, self.grid_sort_column, self.grid_sort_desc);
            self.auto_size_column(result, &indexes, column_index);
        }
    }

    fn cell_label(cell: &crate::UiCell) -> String {
        match cell {
            crate::UiCell::Null => "NULL".to_owned(),
            crate::UiCell::Boolean(value) => value.to_string(),
            crate::UiCell::Number(value) => value.clone(),
            crate::UiCell::Text(value) => value.clone(),
            crate::UiCell::Json(value) => serde_json::from_str::<serde_json::Value>(value)
                .ok()
                .and_then(|json| serde_json::to_string_pretty(&json).ok())
                .unwrap_or_else(|| value.clone()),
            crate::UiCell::Bytes(value) => value.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn row_range_selection_follows_filtered_sort_order() {
        let mut app = DbProApp::default();
        let indexes = [4, 1, 7, 2];
        let lookup = GridSelectionLookup::new(&indexes, &[]);
        app.select_visible_row(&indexes, &lookup.row_positions, 1, false, false);
        app.select_visible_row(&indexes, &lookup.row_positions, 3, true, false);

        assert_eq!(app.selected_rows.into_iter().collect::<Vec<_>>(), vec![1, 2, 7]);
        assert_eq!(app.selected_row, Some(2));
        assert_eq!(app.selection_anchor_row, Some(1));
    }

    #[test]
    fn toggling_last_row_keeps_a_non_empty_selection() {
        let mut app = DbProApp::default();
        let indexes = [3];
        let lookup = GridSelectionLookup::new(&indexes, &[]);
        app.select_visible_row(&indexes, &lookup.row_positions, 0, false, false);
        app.select_visible_row(&indexes, &lookup.row_positions, 0, false, true);

        assert_eq!(app.selected_rows.into_iter().collect::<Vec<_>>(), vec![3]);
        assert_eq!(app.selected_row, Some(3));
    }

    #[test]
    fn cell_range_selection_uses_visible_row_and_column_order() {
        let mut app = DbProApp::default();
        let indexes = [4, 1, 7, 2];
        let order = [2, 0, 1];
        let lookup = GridSelectionLookup::new(&indexes, &order);

        app.select_single_cell((1, 0));
        app.select_cell_range(&indexes, &lookup.row_positions, (2, 1), true);

        assert_eq!(app.selected_cell, Some((2, 1)));
        assert_eq!(app.selected_rows.iter().copied().collect::<Vec<_>>(), vec![1, 2, 7]);
        assert!(app.is_cell_selected(&lookup, (1, 0)));
        assert!(app.is_cell_selected(&lookup, (7, 0)));
        assert!(app.is_cell_selected(&lookup, (2, 1)));
        assert!(!app.is_cell_selected(&lookup, (1, 2)));
        assert!(!app.is_cell_selected(&lookup, (4, 0)));
    }

    #[test]
    fn select_all_visible_cells_covers_current_grid() {
        let mut app = DbProApp::default();
        let indexes = [5, 2, 9];
        let order = [1, 0, 3];
        let lookup = GridSelectionLookup::new(&indexes, &order);
        app.select_all_visible_cells(&indexes, &order);

        assert_eq!(app.selected_cell, Some((9, 3)));
        assert_eq!(app.selection_anchor_cell, Some((5, 1)));
        assert!([5, 2, 9].iter().all(|row| app.selected_rows.contains(row)));
        assert!(app.is_cell_selected(&lookup, (2, 0)));
    }

    #[test]
    fn table_sort_cycles_and_shift_adds_prioritized_clauses() {
        let mut app = DbProApp {
            active_tab: WorkspaceTab::Table,
            table_view: TableView::Data,
            ..Default::default()
        };
        let result = UiQueryResult {
            columns: vec![
                crate::UiColumn {
                    name: "tenant_id".to_owned(),
                    data_type: "integer".to_owned(),
                    nullable: false,
                },
                crate::UiColumn {
                    name: "item_id".to_owned(),
                    data_type: "integer".to_owned(),
                    nullable: false,
                },
            ],
            rows: Vec::new(),
            row_count: 0,
            duration_ms: 0,
        };

        app.cycle_table_data_sort(&result, 0, false);
        assert_eq!(app.table_data_sorts[0].column, "tenant_id");
        assert!(!app.table_data_sorts[0].descending);
        app.cycle_table_data_sort(&result, 0, false);
        assert!(app.table_data_sorts[0].descending);
        app.cycle_table_data_sort(&result, 1, true);
        assert_eq!(
            app.table_data_sorts
                .iter()
                .map(|sort| sort.column.as_str())
                .collect::<Vec<_>>(),
            vec!["tenant_id", "item_id"]
        );
        app.cycle_table_data_sort(&result, 0, true);
        assert_eq!(app.table_data_sorts.len(), 1);
        assert_eq!(app.table_data_sorts[0].column, "item_id");
    }

    #[test]
    fn named_layout_drops_removed_columns_and_appends_new_columns() {
        let mut app = DbProApp {
            grid_pending_named_layout: Some(vec![
                PersistedGridColumnLayout {
                    column_name: "id".to_owned(),
                    width: 240.0,
                    order: 1,
                    hidden: true,
                },
                PersistedGridColumnLayout {
                    column_name: "removed".to_owned(),
                    width: 500.0,
                    order: 0,
                    hidden: true,
                },
            ]),
            ..Default::default()
        };
        let columns = vec![
            crate::UiColumn {
                name: "id".to_owned(),
                data_type: "integer".to_owned(),
                nullable: false,
            },
            crate::UiColumn {
                name: "name".to_owned(),
                data_type: "text".to_owned(),
                nullable: true,
            },
        ];

        assert_eq!(app.column_order_for_columns(&columns), vec![1]);
        assert_eq!(app.grid_column_order, vec![0, 1]);
        assert_eq!(app.grid_column_widths, vec![240.0, 180.0]);
        assert_eq!(app.grid_hidden_columns, [0].into_iter().collect());
    }

    #[test]
    fn named_layout_does_not_map_renamed_column_state() {
        let mut app = DbProApp {
            grid_pending_named_layout: Some(vec![PersistedGridColumnLayout {
                column_name: "old_name".to_owned(),
                width: 420.0,
                order: 0,
                hidden: true,
            }]),
            ..Default::default()
        };
        let columns = vec![crate::UiColumn {
            name: "new_name".to_owned(),
            data_type: "text".to_owned(),
            nullable: true,
        }];

        assert_eq!(app.column_order_for_columns(&columns), vec![0]);
        assert_eq!(app.grid_column_widths, vec![180.0]);
        assert!(app.grid_hidden_columns.is_empty());
    }

    #[test]
    fn legacy_layout_is_discarded_when_schema_shape_changes() {
        let mut app = DbProApp {
            grid_column_order: vec![1, 0],
            grid_column_widths: vec![300.0, 300.0],
            grid_legacy_layout_pending: true,
            ..Default::default()
        };
        let columns = vec![crate::UiColumn {
            name: "only_column".to_owned(),
            data_type: "text".to_owned(),
            nullable: true,
        }];

        assert_eq!(app.column_order_for_columns(&columns), vec![0]);
        assert!(app.grid_column_widths.is_empty());
        assert!(app.grid_hidden_columns.is_empty());
    }

    #[test]
    fn persisted_layout_is_normalized_when_schema_changes() {
        let mut app = DbProApp {
            grid_column_order: vec![4, 1, 1, 99],
            grid_hidden_columns: [4, 88].into_iter().collect(),
            grid_column_widths: vec![40.0, 120.0, 2000.0, 240.0],
            ..Default::default()
        };

        assert_eq!(app.column_order(3), vec![1, 0, 2]);
        assert!(app.grid_hidden_columns.is_empty());
        assert_eq!(app.grid_column_widths, vec![60.0, 120.0, 1000.0]);
    }

    #[test]
    fn sorting_is_blocked_while_staged_changes_are_present() {
        let mut app = DbProApp {
            active_tab: WorkspaceTab::Table,
            table_view: TableView::Data,
            staged_changes: ChangeSet::from(vec![StagedChange::Insert {
                local_id: 1,
                columns: vec!["name".to_owned()],
                values: vec![UiCell::Text("draft".to_owned())],
            }]),
            ..Default::default()
        };
        let result = UiQueryResult {
            columns: vec![crate::UiColumn {
                name: "name".to_owned(),
                data_type: "text".to_owned(),
                nullable: true,
            }],
            rows: Vec::new(),
            row_count: 0,
            duration_ms: 0,
        };

        app.cycle_table_data_sort(&result, 0, false);

        assert!(app.table_data_sorts.is_empty());
        assert!(app.runtime_message.contains("staged changes"));
    }
}
