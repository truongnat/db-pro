//! Feature-owned state for the table/data grid interaction surface.

use super::*;

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
        let mut state = TableDataState::default();
        state.grid_layout_column_names = vec!["id".to_owned(), "email".to_owned()];
        state.grid_column_order = vec![1, 0];
        state.grid_column_widths = vec![100.0, 240.0];
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
        let mut state = TableDataState::default();
        state.grid_column_order = vec![1, 0];
        state.grid_column_widths = vec![120.0, 180.0];
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
}
