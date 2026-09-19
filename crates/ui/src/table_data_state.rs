//! Feature-owned state for the table/data grid interaction surface.

use super::*;

pub(crate) struct TableDataState {
    pub(crate) grid_filter: String,
    pub(crate) grid_sort_column: Option<usize>,
    pub(crate) grid_sort_desc: bool,
    pub(crate) grid_column_widths: Vec<f32>,
    pub(crate) grid_column_order: Vec<usize>,
    pub(crate) grid_hidden_columns: BTreeSet<usize>,
    pub(crate) grid_layout_preferences: HashMap<String, PersistedGridLayout>,
    pub(crate) grid_pending_named_layout: Option<Vec<PersistedGridColumnLayout>>,
    pub(crate) grid_legacy_layout_pending: bool,
    pub(crate) grid_layout_column_names: Vec<String>,
    pub(crate) grid_row_identity_cache: HashMap<usize, RowIdentity>,
    pub(crate) grid_row_identity_cache_ready: bool,
    pub(crate) grid_projection_epoch: u64,
    pub(crate) grid_projection_cache: GridProjectionCache,
    pub(crate) grid_selection_cache: GridSelectionCache,
    pub(crate) grid_columns_user_resized: bool,
    pub(crate) selected_cell: Option<(usize, usize)>,
    pub(crate) selected_row: Option<usize>,
    pub(crate) selected_rows: BTreeSet<usize>,
    pub(crate) selection_anchor_row: Option<usize>,
    pub(crate) selection_anchor_cell: Option<(usize, usize)>,
    pub(crate) data_editing_cell: Option<(usize, usize)>,
    pub(crate) expanded_data_editor: Option<(usize, usize)>,
    pub(crate) cell_inspector_mode: cell_inspector::CellInspectorMode,
    pub(crate) record_inspector_open: bool,
    pub(crate) data_edit_value: String,
    pub(crate) data_edit_error: Option<String>,
    pub(crate) data_delete_confirmation: bool,
    pub(crate) discard_changes_confirmation: bool,
    pub(crate) insert_row_open: bool,
    pub(crate) insert_row_values: Vec<String>,
    pub(crate) insert_row_error: String,
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
}
