//! Table runtime event reducers.

use super::*;
use crate::RequestId;

#[derive(Debug, Default, PartialEq, Eq)]
pub(crate) struct TableEventTransition {
    pub(crate) invalidate_grid_caches: bool,
    pub(crate) apply_staged_changes: bool,
}

#[derive(Debug, PartialEq, Eq)]
pub(crate) enum TableFailureTransition {
    Handled,
    StagedApplyFailed,
}

/// Routes a table-related failure to the matching request slot.
pub(crate) fn handle_table_request_failure(
    table_state: &mut TableState,
    table_mutation: &mut TableMutationState,
    table_data: &mut TableDataState,
    feedback: &mut FeedbackState,
    request_id: RequestId,
    message: &str,
) -> Option<TableFailureTransition> {
    if table_mutation.staged_apply_request == Some(request_id) {
        // Older runtimes can still report the generic failure event. Keep
        // the staged changes and let the root apply the unmapped mutation result.
        return Some(TableFailureTransition::StagedApplyFailed);
    }
    if table_mutation.table_mutation_request == Some(request_id) {
        table_mutation.table_mutation_request = None;
        table_data.data_editing_cell = None;
        table_data.data_edit_value.clear();
        table_data.data_edit_error = None;
        table_data.data_delete_confirmation = false;
        let formatted = format!("Row mutation failed · {message}");
        feedback.set_runtime_message(formatted.clone());
        feedback.show_error_toast(formatted);
        return Some(TableFailureTransition::Handled);
    }
    if table_state.table_info_request == Some(request_id) {
        table_state.table_info_request = None;
        table_state.table_info_error = Some(message.to_owned());
        feedback.set_runtime_message(format!("Table structure failed · {message}"));
        return Some(TableFailureTransition::Handled);
    }
    if table_state.table_ddl_request == Some(request_id) {
        table_state.table_ddl_request = None;
        table_state.table_ddl_error = Some(message.to_owned());
        feedback.set_runtime_message(format!("Table DDL failed · {message}"));
        return Some(TableFailureTransition::Handled);
    }
    if table_state.table_row_reload_request == Some(request_id) {
        table_state.table_row_reload_request = None;
        table_state.table_row_reload_identity = None;
        table_mutation.table_mutation_retry_after_reload = false;
        table_mutation.table_mutation_retry_target = None;
        feedback.set_runtime_message(format!("Could not reload row: {message}"));
        return Some(TableFailureTransition::Handled);
    }
    if table_state.table_data_request == Some(request_id) {
        table_state.table_data_request = None;
        table_mutation.table_mutation_retry_after_reload = false;
        table_mutation.table_mutation_retry_target = None;
        table_state.table_data_error = Some(message.to_owned());
        let formatted = format!("Table data failed · {message}");
        feedback.set_runtime_message(formatted.clone());
        feedback.show_error_toast(formatted);
        return Some(TableFailureTransition::Handled);
    }
    if table_state.ddl_execution_request == Some(request_id) {
        table_state.ddl_execution_request = None;
        table_state.ddl_execute_confirmation = false;
        table_state.table_ddl_error = Some(message.to_owned());
        let formatted = format!("DDL execution failed · {message}");
        feedback.set_runtime_message(formatted.clone());
        feedback.show_error_toast(formatted);
        return Some(TableFailureTransition::Handled);
    }
    None
}

/// Applies table metadata and seeds the first filter/sort choices.
pub(crate) fn on_table_info_loaded(
    table_state: &mut TableState,
    feedback: &mut FeedbackState,
    request_id: RequestId,
    table_info: UiTableInfo,
) -> Option<TableEventTransition> {
    if table_state.table_info_request != Some(request_id) {
        return None;
    }
    if table_state.table_data_filter_column.is_empty() {
        table_state.table_data_filter_column = table_info
            .columns
            .first()
            .map(|column| column.name.clone())
            .unwrap_or_default();
    }
    if table_state.table_data_sorts.is_empty() {
        if let Some(column) = table_info
            .primary_key
            .as_ref()
            .and_then(|columns| columns.first().cloned())
            .or_else(|| table_info.columns.first().map(|column| column.name.clone()))
        {
            table_state.table_data_sorts.push(UiTableDataSort {
                column,
                descending: false,
            });
        }
    }
    table_state.table_info = Some(table_info);
    table_state.table_info_error = None;
    table_state.table_info_request = None;
    feedback.set_runtime_message("Table structure loaded");
    Some(TableEventTransition {
        invalidate_grid_caches: true,
        ..Default::default()
    })
}

pub(crate) fn on_table_ddl_loaded(
    table_state: &mut TableState,
    feedback: &mut FeedbackState,
    request_id: RequestId,
    sql: String,
) {
    if table_state.table_ddl_request != Some(request_id) {
        return;
    }
    table_state.table_ddl = Some(sql);
    table_state.ddl_execute_confirmation = false;
    table_state.table_ddl_error = None;
    table_state.table_ddl_request = None;
    feedback.set_runtime_message("Table DDL loaded");
}

/// Applies table data and returns the side effects that need root orchestration.
pub(crate) fn on_table_data_loaded(
    table_state: &mut TableState,
    table_mutation: &mut TableMutationState,
    table_data: &mut TableDataState,
    feedback: &mut FeedbackState,
    request_id: RequestId,
    result: UiQueryResult,
    total_rows: u64,
) -> Option<TableEventTransition> {
    if table_state.table_row_reload_request == Some(request_id) {
        return Some(on_table_row_reloaded(table_state, table_mutation, feedback, result));
    }
    if table_state.table_data_request != Some(request_id) {
        return None;
    }
    if table_state.table_data_filter_column.is_empty() {
        table_state.table_data_filter_column = result
            .columns
            .first()
            .map(|column| column.name.clone())
            .unwrap_or_default();
    }
    if table_state.table_data_sorts.is_empty() {
        if let Some(column) = result.columns.first().map(|column| column.name.clone()) {
            table_state.table_data_sorts.push(UiTableDataSort {
                column,
                descending: false,
            });
        }
    }
    table_state.table_data_result = Some(result);
    table_state.table_data_total_rows = Some(total_rows);
    let clear_selection = table_mutation.staged_changes.is_empty();
    if clear_selection {
        table_data.selected_cell = None;
        table_data.selected_row = None;
        table_data.selected_rows.clear();
        table_data.selection_anchor_row = None;
        table_data.selection_anchor_cell = None;
    }
    table_state.table_data_error = None;
    table_state.table_data_request = None;
    feedback.set_runtime_message(format!("Table data loaded · {total_rows} rows"));
    Some(TableEventTransition {
        invalidate_grid_caches: true,
        apply_staged_changes: table_mutation.table_mutation_retry_after_reload,
    })
}

pub(crate) fn on_table_row_reloaded(
    table_state: &mut TableState,
    table_mutation: &mut TableMutationState,
    feedback: &mut FeedbackState,
    result: UiQueryResult,
) -> TableEventTransition {
    table_state.table_row_reload_request = None;
    let Some(identity) = table_state.table_row_reload_identity.take() else {
        return TableEventTransition::default();
    };
    let Some(server_row) = result.rows.into_iter().next() else {
        feedback.set_runtime_message("Row was deleted");
        if table_mutation.table_mutation_retry_after_reload {
            table_mutation.table_mutation_retry_after_reload = false;
            table_mutation.table_mutation_retry_target = None;
        }
        return TableEventTransition::default();
    };

    let mut replaced_row = false;
    if let Some(table_result) = table_state.table_data_result.as_mut() {
        let column_indexes: std::collections::HashMap<&str, usize> = table_result
            .columns
            .iter()
            .enumerate()
            .map(|(index, column)| (column.name.as_str(), index))
            .collect();
        if let Some(row_index) = table_result
            .rows
            .iter()
            .position(|row| row_matches_identity(row, &column_indexes, &identity))
        {
            table_result.rows[row_index] = server_row;
            replaced_row = true;
        }
    }
    table_state.table_data_error = None;
    feedback.set_runtime_message("Row reloaded from database");
    TableEventTransition {
        invalidate_grid_caches: replaced_row,
        apply_staged_changes: table_mutation.table_mutation_retry_after_reload,
    }
}

pub(crate) fn row_matches_identity(
    row: &[UiCell],
    column_indexes: &std::collections::HashMap<&str, usize>,
    identity: &RowIdentity,
) -> bool {
    identity
        .original_pk_columns
        .iter()
        .zip(&identity.original_pk_values)
        .all(|(column, value)| {
            column_indexes
                .get(column.as_str())
                .and_then(|index| row.get(*index))
                .is_some_and(|candidate| candidate == value)
        })
}
