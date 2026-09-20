//! Local editing and inspector state for the table data surface.
//!
//! Grid projection/layout/selection belongs to [`TableDataState`]. This state
//! owns only the transient interaction lifecycle around editing a cell,
//! inspecting a value and staging a new row.

use super::cell_inspector;

#[derive(Debug)]
pub(crate) struct TableEditingState {
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

impl Default for TableEditingState {
    fn default() -> Self {
        Self {
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
