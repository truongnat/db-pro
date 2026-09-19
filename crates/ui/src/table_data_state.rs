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
}
