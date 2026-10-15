//! Composition state for the table editor feature.
//!
//! Keeping table metadata, grid interaction, and staged mutations under one
//! feature owner prevents the application shell from presenting them as
//! unrelated global concerns.

use super::{TableDataQueryState, TableDataState, TableEditingState, TableMutationState, TableState, TableView};

#[derive(Default)]
pub(crate) struct TableEditorState {
    pub(super) data: TableDataState,
    pub(super) data_query: TableDataQueryState,
    pub(super) editing: TableEditingState,
    pub(super) mutation: TableMutationState,
    pub(super) state: TableState,
}

impl TableEditorState {
    pub(crate) fn reset_workspace(&mut self) {
        self.state.table_info = None;
        self.state.table_ddl = None;
        self.state.table_ddl_baseline = None;
        self.state.table_ddl_discard_confirm = false;
        self.state.table_info_error = None;
        self.state.table_ddl_error = None;
        self.state.ddl_execute_confirmation = false;
        self.state.ddl_execution_request = None;
        self.data_query.reset_for_table();
        self.state.table_info_request = None;
        self.state.table_ddl_request = None;
        self.mutation.table_mutation_request = None;
        self.mutation.staged_changes.clear();
        self.mutation.staged_apply_request = None;
        self.mutation.staged_apply_confirm = false;
        self.mutation.staged_apply_targets.clear();
        self.mutation.table_mutation_retry_after_reload = false;
        self.mutation.table_mutation_retry_target = None;
        self.mutation.table_mutation_error = None;
        self.mutation.pending_changes_open = false;
        self.mutation.conflict_dialog_open = false;
        self.data.clear_selection();
        self.data.invalidate_grid_row_caches();
        self.editing.data_editing_cell = None;
        self.editing.data_edit_value.clear();
        self.editing.data_edit_error = None;
        self.editing.data_delete_confirmation = false;
        self.editing.discard_changes_confirmation = false;
        self.state.table_view = TableView::Data;
    }
}

#[cfg(test)]
#[path = "tests/table_editor_state_tests.rs"]
mod tests;

