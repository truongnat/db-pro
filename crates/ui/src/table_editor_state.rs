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
mod tests {
    use super::super::{RowIdentity, UiCell};
    use crate::RequestId;

    use super::*;

    #[test]
    fn reset_workspace_clears_table_requests_selection_and_staged_mutations() {
        let mut state = TableEditorState::default();
        state.state.table_info_request = Some(RequestId(7));
        state.state.table_ddl_request = Some(RequestId(8));
        state.data.select_single_row(2);
        state.data.grid_row_identity_cache_ready = true;
        state.data.grid_row_identity_cache.insert(
            0,
            RowIdentity {
                original_pk_columns: vec!["id".to_owned()],
                original_pk_values: vec![UiCell::Number("1".to_owned())],
            },
        );
        state.mutation.pending_changes_open = true;
        state.editing.data_edit_value = "draft".to_owned();
        state.editing.data_delete_confirmation = true;
        state.editing.discard_changes_confirmation = true;

        state.reset_workspace();

        assert!(state.state.table_info_request.is_none());
        assert!(state.state.table_ddl_request.is_none());
        assert!(state.data.selected_rows.is_empty());
        assert!(!state.data.grid_row_identity_cache_ready);
        assert!(state.data.grid_row_identity_cache.is_empty());
        assert!(state.mutation.staged_changes.is_empty());
        assert!(!state.mutation.pending_changes_open);
        assert!(state.editing.data_edit_value.is_empty());
        assert!(!state.editing.data_delete_confirmation);
        assert!(!state.editing.discard_changes_confirmation);
        assert_eq!(state.state.table_view, TableView::Data);
    }
}
