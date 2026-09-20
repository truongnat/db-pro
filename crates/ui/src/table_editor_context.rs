//! Explicit table-editor state transitions.

use super::{
    ConnectionCatalogState, ConnectionLifecycleState, FeedbackState, TableDataState, TableMutationState, TableState,
    UiCell, UiQueryResult,
};

pub(crate) fn can_mutate_active_connection(
    catalog: &ConnectionCatalogState,
    lifecycle: &ConnectionLifecycleState,
) -> bool {
    lifecycle.is_connected()
        && lifecycle
            .active_connection_id()
            .and_then(|id| catalog.find(id))
            .is_some_and(|connection| !connection.readonly)
}

pub(crate) fn can_edit_table_rows(
    table_state: &TableState,
    catalog: &ConnectionCatalogState,
    lifecycle: &ConnectionLifecycleState,
) -> bool {
    can_mutate_active_connection(catalog, lifecycle) && table_state.has_primary_key()
}

pub(crate) struct TableMutationContext<'a> {
    table_state: &'a mut TableState,
    table_data: &'a mut TableDataState,
    table_mutation: &'a mut TableMutationState,
    feedback: &'a mut FeedbackState,
}

pub(crate) fn staged_cell_value(
    table_data: &TableDataState,
    table_state: &TableState,
    table_mutation: &TableMutationState,
    result: &UiQueryResult,
    row_index: usize,
    column_index: usize,
) -> Option<UiCell> {
    let identity = table_data.row_identity_for_result(result, table_state.table_info.as_ref(), row_index)?;
    table_mutation.staged_changes.cell_value(&identity, column_index)
}

pub(crate) fn staged_row_deleted(
    table_data: &TableDataState,
    table_state: &TableState,
    table_mutation: &TableMutationState,
    result: &UiQueryResult,
    row_index: usize,
) -> bool {
    let Some(identity) = table_data.row_identity_for_result(result, table_state.table_info.as_ref(), row_index) else {
        return false;
    };
    table_mutation.staged_changes.row_deleted(&identity)
}

impl<'a> TableMutationContext<'a> {
    pub(crate) fn new(
        table_state: &'a mut TableState,
        table_data: &'a mut TableDataState,
        table_mutation: &'a mut TableMutationState,
        feedback: &'a mut FeedbackState,
    ) -> Self {
        Self {
            table_state,
            table_data,
            table_mutation,
            feedback,
        }
    }

    pub(crate) fn revert_staged_cell(&mut self, result: &UiQueryResult, row_index: usize, column_index: usize) {
        let Some(identity) =
            self.table_data
                .row_identity_for_result(result, self.table_state.table_info.as_ref(), row_index)
        else {
            return;
        };
        if self.table_mutation.staged_changes.revert_cell(&identity, column_index) {
            self.table_mutation
                .clear_error_for_identity(&identity, Some(column_index));
            self.feedback.set_runtime_message("Cell change reverted");
        }
    }

    pub(crate) fn revert_staged_row(&mut self, result: &UiQueryResult, row_index: usize) {
        let Some(identity) =
            self.table_data
                .row_identity_for_result(result, self.table_state.table_info.as_ref(), row_index)
        else {
            return;
        };
        if self.table_mutation.staged_changes.revert_row(&identity) {
            self.table_mutation.clear_error_for_identity(&identity, None);
            self.feedback.set_runtime_message("Row changes reverted");
        }
    }

    /// Clears staged UI state and reports whether a data reload is required.
    pub(crate) fn discard_staged_changes(&mut self) -> bool {
        if self.table_mutation.staged_apply_request.is_some() {
            self.feedback
                .set_runtime_message("Wait for the current database write before discarding");
            return false;
        }
        self.table_mutation.staged_changes.clear();
        self.table_mutation.staged_apply_targets.clear();
        self.table_mutation.table_mutation_error = None;
        self.table_data.data_editing_cell = None;
        self.table_data.expanded_data_editor = None;
        self.table_data.data_edit_error = None;
        self.table_data.data_delete_confirmation = false;
        self.table_data.discard_changes_confirmation = false;
        self.table_mutation.pending_changes_open = false;
        self.table_data.data_edit_value.clear();
        self.table_state.table_data_result = None;
        self.table_state.table_data_error = None;
        self.feedback.set_runtime_message("Staged changes discarded");
        true
    }
}
