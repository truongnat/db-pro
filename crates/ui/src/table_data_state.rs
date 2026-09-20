//! Feature-owned state for the table/data grid interaction surface.

use super::*;
use std::collections::HashSet;

pub(crate) struct TableDataState {
    pub(super) grid_filter: String,
    pub(super) grid_sort_column: Option<usize>,
    pub(super) grid_sort_desc: bool,
    pub(super) grid_column_widths: Vec<f32>,
    pub(super) grid_column_order: Vec<usize>,
    pub(super) grid_hidden_columns: BTreeSet<usize>,
    pub(super) grid_layout_preferences: HashMap<String, PersistedGridLayout>,
    pub(super) grid_pending_named_layout: Option<Vec<PersistedGridColumnLayout>>,
    pub(super) grid_legacy_layout_pending: bool,
    pub(super) grid_layout_column_names: Vec<String>,
    pub(super) grid_row_identity_cache: HashMap<usize, RowIdentity>,
    pub(super) grid_row_identity_cache_ready: bool,
    pub(super) grid_projection_epoch: u64,
    pub(super) grid_projection_cache: GridProjectionCache,
    pub(super) grid_selection_cache: GridSelectionCache,
    pub(super) grid_columns_user_resized: bool,
    pub(super) selected_cell: Option<(usize, usize)>,
    pub(super) selected_row: Option<usize>,
    pub(super) selected_rows: BTreeSet<usize>,
    pub(super) selection_anchor_row: Option<usize>,
    pub(super) selection_anchor_cell: Option<(usize, usize)>,
    pub(super) data_editing_cell: Option<(usize, usize)>,
    pub(super) expanded_data_editor: Option<(usize, usize)>,
    pub(super) cell_inspector_mode: cell_inspector::CellInspectorMode,
    pub(super) record_inspector_open: bool,
    pub(super) data_edit_value: String,
    pub(super) data_edit_error: Option<String>,
    pub(super) data_delete_confirmation: bool,
    pub(super) discard_changes_confirmation: bool,
    pub(super) insert_row_open: bool,
    pub(super) insert_row_values: Vec<String>,
    pub(super) insert_row_error: String,
}

impl Default for TableDataState {
    fn default() -> Self {
        Self {
            grid_filter: String::new(),
            grid_sort_column: None,
            grid_sort_desc: false,
            grid_column_widths: Vec::new(),
            grid_column_order: Vec::new(),
            grid_hidden_columns: BTreeSet::new(),
            grid_layout_preferences: HashMap::new(),
            grid_pending_named_layout: None,
            grid_legacy_layout_pending: false,
            grid_layout_column_names: Vec::new(),
            grid_row_identity_cache: HashMap::new(),
            grid_row_identity_cache_ready: false,
            grid_projection_epoch: 0,
            grid_projection_cache: GridProjectionCache::default(),
            grid_selection_cache: GridSelectionCache::default(),
            grid_columns_user_resized: false,
            selected_cell: None,
            selected_row: None,
            selected_rows: BTreeSet::new(),
            selection_anchor_row: None,
            selection_anchor_cell: None,
            data_editing_cell: None,
            expanded_data_editor: None,
            cell_inspector_mode: cell_inspector::CellInspectorMode::Raw,
            record_inspector_open: false,
            data_edit_value: String::new(),
            data_edit_error: None,
            data_delete_confirmation: false,
            discard_changes_confirmation: false,
            insert_row_open: false,
            insert_row_values: Vec::new(),
            insert_row_error: String::new(),
        }
    }
}

impl TableDataState {
    pub(crate) fn column_order_for_columns(&mut self, columns: &[crate::UiColumn]) -> Vec<usize> {
        let count = columns.len();
        self.grid_layout_column_names = columns.iter().map(|column| column.name.clone()).collect();
        self.restore_named_layout(columns, count);
        self.reconcile_legacy_layout(count);
        self.column_order(count)
    }

    fn restore_named_layout(&mut self, columns: &[crate::UiColumn], count: usize) {
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
        }
    }

    fn reconcile_legacy_layout(&mut self, count: usize) {
        if !self.grid_legacy_layout_pending {
            return;
        }
        let widths_match = self.grid_column_widths.is_empty() || self.grid_column_widths.len() == count;
        if self.grid_column_order.len() != count || !widths_match {
            self.grid_column_order.clear();
            self.grid_column_widths.clear();
            self.grid_hidden_columns.clear();
            self.grid_columns_user_resized = false;
        }
        self.grid_legacy_layout_pending = false;
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

    pub(crate) fn hide_column(&mut self, column_index: usize, visible_count: usize) {
        if visible_count > 1 {
            self.grid_hidden_columns.insert(column_index);
        }
    }

    pub(crate) fn show_all_columns(&mut self) {
        self.grid_hidden_columns.clear();
    }

    pub(crate) fn reset_grid_layout(&mut self, count: usize) {
        self.grid_column_order = (0..count).collect();
        self.grid_hidden_columns.clear();
        self.grid_column_widths.clear();
        self.grid_columns_user_resized = false;
    }

    pub(crate) fn auto_size_column(&mut self, result: &UiQueryResult, indexes: &[usize], column_index: usize) {
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

    pub(crate) fn column_widths(&mut self, count: usize, available_width: f32) -> Vec<f32> {
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

    pub(crate) fn row_identity(
        result: &UiQueryResult,
        info: &UiTableInfo,
        row_index: usize,
    ) -> Result<RowIdentity, String> {
        let Some(primary_key) = info.primary_key.as_ref() else {
            return Err("This table has no primary key for safe row editing".to_owned());
        };
        let column_indexes: HashMap<&str, usize> = result
            .columns
            .iter()
            .enumerate()
            .map(|(index, column)| (column.name.as_str(), index))
            .collect();
        let row = result
            .rows
            .get(row_index)
            .ok_or_else(|| "The selected row is no longer available".to_owned())?;
        Self::row_identity_from_row(row, primary_key, &column_indexes)
    }

    fn row_identity_from_row(
        row: &[UiCell],
        primary_key: &[String],
        column_indexes: &HashMap<&str, usize>,
    ) -> Result<RowIdentity, String> {
        let mut pk_values = Vec::with_capacity(primary_key.len());
        for pk_column in primary_key {
            let Some(&pk_index) = column_indexes.get(pk_column.as_str()) else {
                return Err(format!(
                    "The primary-key column {pk_column} is not present in this result"
                ));
            };
            let Some(pk_cell) = row.get(pk_index) else {
                return Err("The selected row is no longer available".to_owned());
            };
            if matches!(pk_cell, UiCell::Null) {
                return Err(format!("A NULL primary key ({pk_column}) cannot identify a row"));
            }
            pk_values.push(pk_cell.clone());
        }
        Ok(RowIdentity {
            original_pk_columns: primary_key.to_vec(),
            original_pk_values: pk_values,
        })
    }

    pub(crate) fn rebuild_row_identity_cache(&mut self, result: &UiQueryResult, info: Option<&UiTableInfo>) {
        if self.grid_row_identity_cache_ready {
            return;
        }
        self.grid_row_identity_cache.clear();
        let Some(primary_key) = info.and_then(|info| info.primary_key.as_ref()) else {
            self.grid_row_identity_cache_ready = true;
            return;
        };
        let column_indexes: HashMap<&str, usize> = result
            .columns
            .iter()
            .enumerate()
            .map(|(index, column)| (column.name.as_str(), index))
            .collect();
        for (row_index, row) in result.rows.iter().enumerate() {
            if let Ok(identity) = Self::row_identity_from_row(row, primary_key, &column_indexes) {
                self.grid_row_identity_cache.insert(row_index, identity);
            }
        }
        self.grid_row_identity_cache_ready = true;
    }

    pub(crate) fn row_identity_for_result(
        &self,
        result: &UiQueryResult,
        info: Option<&UiTableInfo>,
        row_index: usize,
    ) -> Option<RowIdentity> {
        self.grid_row_identity_cache
            .get(&row_index)
            .cloned()
            .or_else(|| info.and_then(|info| Self::row_identity(result, info, row_index).ok()))
    }

    pub(crate) fn projection_key(&self, result: &UiQueryResult) -> GridProjectionKey {
        GridProjectionKey {
            epoch: self.grid_projection_epoch,
            filter: self.grid_filter.clone(),
            sort_column: self.grid_sort_column,
            sort_desc: self.grid_sort_desc,
            row_count: result.row_count,
            column_count: result.columns.len(),
        }
    }

    pub(crate) fn layout_scope(connection_id: Option<&str>, schema: &str, table: Option<&str>) -> Option<String> {
        Some(format!("{}|{}|{}", connection_id?, schema, table?))
    }

    pub(crate) fn persist_layout(&mut self, scope: Option<String>) {
        let Some(scope) = scope else {
            return;
        };
        let column_names = self.grid_layout_column_names.clone();
        if column_names.len() != self.grid_column_order.len() {
            return;
        }
        let columns = self
            .grid_column_order
            .iter()
            .enumerate()
            .filter_map(|(order, &column_index)| {
                let column_name = column_names.get(column_index)?.clone();
                Some(PersistedGridColumnLayout {
                    column_name,
                    width: self.grid_column_widths.get(column_index).copied().unwrap_or(180.0),
                    order,
                    hidden: self.grid_hidden_columns.contains(&column_index),
                })
            })
            .collect();
        self.grid_layout_preferences.insert(
            scope,
            PersistedGridLayout {
                columns,
                ..PersistedGridLayout::default()
            },
        );
    }

    pub(crate) fn restore_layout(&mut self, scope: Option<String>) {
        let Some(scope) = scope else {
            return;
        };
        let Some(layout) = self.grid_layout_preferences.get(&scope).cloned() else {
            self.grid_column_widths.clear();
            self.grid_column_order.clear();
            self.grid_hidden_columns.clear();
            self.grid_pending_named_layout = None;
            self.grid_legacy_layout_pending = false;
            self.grid_columns_user_resized = false;
            return;
        };
        self.grid_layout_column_names.clear();
        self.grid_pending_named_layout = (!layout.columns.is_empty()).then_some(layout.columns);
        self.grid_legacy_layout_pending = self.grid_pending_named_layout.is_none()
            && (!layout.widths.is_empty() || !layout.order.is_empty() || !layout.hidden_columns.is_empty());
        self.grid_column_widths = layout.widths;
        self.grid_column_order = layout.order;
        self.grid_hidden_columns = layout.hidden_columns.into_iter().collect();
        self.grid_columns_user_resized = !self.grid_column_widths.is_empty();
    }

    pub(crate) fn select_visible_row(
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

    pub(crate) fn select_single_row(&mut self, row_index: usize) {
        self.selected_rows.clear();
        self.selected_rows.insert(row_index);
        self.selected_row = Some(row_index);
        self.selection_anchor_row = Some(row_index);
        self.selection_anchor_cell = None;
    }

    pub(crate) fn select_single_cell(&mut self, selection: (usize, usize)) {
        self.selected_cell = Some(selection);
        self.selected_rows.clear();
        self.selected_rows.insert(selection.0);
        self.selected_row = Some(selection.0);
        self.selection_anchor_row = Some(selection.0);
        self.selection_anchor_cell = Some(selection);
    }

    pub(crate) fn select_cell_range(
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

    pub(crate) fn select_all_visible_cells(&mut self, indexes: &[usize], order: &[usize]) {
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
    }

    pub(crate) fn selected_row_indexes(&self) -> Vec<usize> {
        if self.selected_rows.is_empty() {
            self.selected_row.into_iter().collect()
        } else {
            self.selected_rows.iter().copied().collect()
        }
    }

    pub(crate) fn is_cell_selected(
        &self,
        lookup: &super::result_grid_view::GridSelectionLookup,
        selection: (usize, usize),
    ) -> bool {
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

    pub(crate) fn navigation_target(
        &self,
        indexes: &[usize],
        order: &[usize],
        lookup: &super::result_grid_view::GridSelectionLookup,
        key: egui::Key,
        backwards: bool,
    ) -> Option<(usize, usize)> {
        let (&first_row, &first_column) = (indexes.first()?, order.first()?);
        let Some((current_row, current_column)) = self.selected_cell else {
            return (key != egui::Key::Tab).then_some((first_row, first_column));
        };

        let row_position = lookup.row_positions.get(&current_row).copied().unwrap_or(0);
        let column_position = lookup.column_positions.get(&current_column).copied().unwrap_or(0);
        match key {
            egui::Key::Tab if backwards => {
                if column_position > 0 {
                    Some((current_row, order[column_position - 1]))
                } else if row_position > 0 {
                    Some((indexes[row_position - 1], *order.last()?))
                } else {
                    Some((current_row, current_column))
                }
            }
            egui::Key::Tab => {
                if column_position + 1 < order.len() {
                    Some((current_row, order[column_position + 1]))
                } else if row_position + 1 < indexes.len() {
                    Some((indexes[row_position + 1], first_column))
                } else {
                    Some((current_row, current_column))
                }
            }
            egui::Key::ArrowUp => Some((indexes[row_position.saturating_sub(1)], current_column)),
            egui::Key::ArrowDown => Some((indexes[(row_position + 1).min(indexes.len() - 1)], current_column)),
            egui::Key::ArrowLeft => Some((
                current_row,
                if column_position > 0 {
                    order[column_position - 1]
                } else {
                    current_column
                },
            )),
            egui::Key::ArrowRight => Some((
                current_row,
                if column_position + 1 < order.len() {
                    order[column_position + 1]
                } else {
                    current_column
                },
            )),
            egui::Key::Home => Some((current_row, first_column)),
            egui::Key::End => Some((current_row, *order.last()?)),
            _ => None,
        }
    }

    pub(super) fn invalidate_grid_projection(&mut self) {
        self.grid_projection_epoch = self.grid_projection_epoch.wrapping_add(1);
    }

    pub(super) fn invalidate_grid_row_caches(&mut self) {
        self.grid_row_identity_cache.clear();
        self.grid_row_identity_cache_ready = false;
        self.invalidate_grid_projection();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_table_data_state_has_no_selection_or_pending_edit() {
        let state = TableDataState::default();

        assert!(state.grid_filter.is_empty());
        assert!(state.selected_cell.is_none());
        assert!(state.selected_rows.is_empty());
        assert!(state.data_editing_cell.is_none());
        assert!(!state.insert_row_open);
    }

    #[test]
    fn layout_scope_requires_connection_and_table_identity() {
        assert_eq!(
            TableDataState::layout_scope(Some("connection"), "public", Some("users")),
            Some("connection|public|users".to_owned())
        );
        assert_eq!(TableDataState::layout_scope(None, "public", Some("users")), None);
        assert_eq!(TableDataState::layout_scope(Some("connection"), "public", None), None);
    }

    #[test]
    fn layout_persists_and_restores_named_columns() {
        let mut state = TableDataState {
            grid_layout_column_names: vec!["id".to_owned(), "email".to_owned()],
            grid_column_order: vec![1, 0],
            grid_column_widths: vec![100.0, 240.0],
            ..Default::default()
        };
        state.grid_hidden_columns.insert(0);

        state.persist_layout(Some("connection|public|users".to_owned()));

        state.grid_layout_column_names.clear();
        state.grid_column_order.clear();
        state.grid_column_widths.clear();
        state.grid_hidden_columns.clear();
        state.restore_layout(Some("connection|public|users".to_owned()));

        let pending = state.grid_pending_named_layout.as_ref().expect("named layout");
        assert_eq!(pending[0].column_name, "email");
        assert_eq!(pending[0].width, 240.0);
        assert!(!pending[0].hidden);
        assert_eq!(pending[1].column_name, "id");
        assert_eq!(pending[1].width, 100.0);
        assert!(pending[1].hidden);
        assert!(state.grid_column_order.is_empty());
        assert!(state.grid_column_widths.is_empty());
        assert!(state.grid_hidden_columns.is_empty());
        assert!(!state.grid_legacy_layout_pending);
    }

    #[test]
    fn restoring_unknown_layout_scope_clears_grid_projection() {
        let mut state = TableDataState {
            grid_column_order: vec![1, 0],
            grid_column_widths: vec![120.0, 180.0],
            ..Default::default()
        };
        state.grid_hidden_columns.insert(1);
        state.grid_pending_named_layout = Some(Vec::new());
        state.grid_legacy_layout_pending = true;
        state.grid_columns_user_resized = true;

        state.restore_layout(Some("missing".to_owned()));

        assert!(state.grid_column_order.is_empty());
        assert!(state.grid_column_widths.is_empty());
        assert!(state.grid_hidden_columns.is_empty());
        assert!(state.grid_pending_named_layout.is_none());
        assert!(!state.grid_legacy_layout_pending);
        assert!(!state.grid_columns_user_resized);
    }

    #[test]
    fn grid_cache_invalidation_is_owned_by_table_data_state() {
        let mut state = TableDataState::default();
        state.grid_row_identity_cache.insert(
            0,
            RowIdentity {
                original_pk_columns: Vec::new(),
                original_pk_values: Vec::new(),
            },
        );
        state.grid_row_identity_cache_ready = true;

        state.invalidate_grid_row_caches();

        assert!(state.grid_row_identity_cache.is_empty());
        assert!(!state.grid_row_identity_cache_ready);
        assert_eq!(state.grid_projection_epoch, 1);
    }

    #[test]
    fn navigation_target_follows_visual_row_and_column_order() {
        let mut state = TableDataState::default();
        let indexes = [4, 1, 7];
        let order = [2, 0, 1];
        let lookup = super::result_grid_view::GridSelectionLookup::new(&indexes, &order);
        assert_eq!(
            state.navigation_target(&indexes, &order, &lookup, egui::Key::Tab, false),
            None
        );
        state.select_single_cell((1, 0));

        assert_eq!(
            state.navigation_target(&indexes, &order, &lookup, egui::Key::Tab, false),
            Some((1, 1))
        );
        assert_eq!(
            state.navigation_target(&indexes, &order, &lookup, egui::Key::Tab, true),
            Some((1, 2))
        );
        assert_eq!(
            state.navigation_target(&indexes, &order, &lookup, egui::Key::ArrowDown, false),
            Some((7, 0))
        );
    }

    #[test]
    fn projection_key_tracks_grid_state_and_result_shape() {
        let state = TableDataState {
            grid_projection_epoch: 7,
            grid_filter: "active".to_owned(),
            grid_sort_column: Some(2),
            grid_sort_desc: true,
            ..Default::default()
        };
        let result = UiQueryResult {
            columns: vec![
                crate::UiColumn {
                    name: "id".to_owned(),
                    data_type: "integer".to_owned(),
                    nullable: false,
                },
                crate::UiColumn {
                    name: "status".to_owned(),
                    data_type: "text".to_owned(),
                    nullable: false,
                },
            ],
            rows: Vec::new(),
            row_count: 12,
            duration_ms: 4,
        };

        assert_eq!(
            state.projection_key(&result),
            GridProjectionKey {
                epoch: 7,
                filter: "active".to_owned(),
                sort_column: Some(2),
                sort_desc: true,
                row_count: 12,
                column_count: 2,
            }
        );
    }
}
