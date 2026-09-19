//! Runtime events owned by the feature slice.

use super::*;
use crate::RequestId;

impl DbProApp {
    pub(super) fn handle_table_request_failure(&mut self, request_id: RequestId, message: &str) -> bool {
        if self.table_mutation.staged_apply_request == Some(request_id) {
            // Older runtimes can still report the generic failure event. Keep
            // the staged changes and surface it as an unmapped mutation.
            self.staged_apply_failed(usize::MAX, "UNKNOWN", message, false);
            return true;
        }
        if self.table_mutation.table_mutation_request == Some(request_id) {
            self.table_mutation.table_mutation_request = None;
            self.table_data.data_editing_cell = None;
            self.table_data.data_edit_value.clear();
            self.table_data.data_edit_error = None;
            self.table_data.data_delete_confirmation = false;
            let formatted = format!("Row mutation failed · {message}");
            self.feedback.runtime_message = formatted.clone();
            self.show_toast_error(formatted);
            return true;
        }
        if self.table_state.table_info_request == Some(request_id) {
            self.table_state.table_info_request = None;
            self.table_state.table_info_error = Some(message.to_owned());
            self.feedback.runtime_message = format!("Table structure failed · {message}");
            return true;
        }
        if self.table_state.table_ddl_request == Some(request_id) {
            self.table_state.table_ddl_request = None;
            self.table_state.table_ddl_error = Some(message.to_owned());
            self.feedback.runtime_message = format!("Table DDL failed · {message}");
            return true;
        }
        if self.table_state.table_row_reload_request == Some(request_id) {
            self.table_state.table_row_reload_request = None;
            self.table_state.table_row_reload_identity = None;
            self.table_mutation.table_mutation_retry_after_reload = false;
            self.table_mutation.table_mutation_retry_target = None;
            self.feedback.runtime_message = format!("Could not reload row: {message}");
            return true;
        }
        if self.table_state.table_data_request == Some(request_id) {
            self.table_state.table_data_request = None;
            self.table_mutation.table_mutation_retry_after_reload = false;
            self.table_mutation.table_mutation_retry_target = None;
            self.table_state.table_data_error = Some(message.to_owned());
            let formatted = format!("Table data failed · {message}");
            self.feedback.runtime_message = formatted.clone();
            self.show_toast_error(formatted);
            return true;
        }
        if self.table_state.ddl_execution_request == Some(request_id) {
            self.table_state.ddl_execution_request = None;
            self.table_state.ddl_execute_confirmation = false;
            self.table_state.table_ddl_error = Some(message.to_owned());
            let formatted = format!("DDL execution failed · {message}");
            self.feedback.runtime_message = formatted.clone();
            self.show_toast_error(formatted);
            return true;
        }
        false
    }

    /// Table structure arrived: seed filter/sort defaults on first load.
    pub(super) fn on_table_info_loaded(&mut self, request_id: RequestId, table_info: UiTableInfo) {
        if self.table_state.table_info_request != Some(request_id) {
            return;
        }
        if self.table_state.table_data_filter_column.is_empty() {
            self.table_state.table_data_filter_column = table_info
                .columns
                .first()
                .map(|column| column.name.clone())
                .unwrap_or_default();
        }
        if self.table_state.table_data_sorts.is_empty() {
            if let Some(column) = table_info
                .primary_key
                .as_ref()
                .and_then(|columns| columns.first().cloned())
                .or_else(|| table_info.columns.first().map(|column| column.name.clone()))
            {
                self.table_state.table_data_sorts.push(UiTableDataSort {
                    column,
                    descending: false,
                });
            }
        }
        self.table_state.table_info = Some(table_info);
        self.invalidate_grid_row_caches();
        self.table_state.table_info_error = None;
        self.table_state.table_info_request = None;
        self.feedback.runtime_message = "Table structure loaded".to_owned();
    }

    pub(super) fn on_table_ddl_loaded(&mut self, request_id: RequestId, sql: String) {
        if self.table_state.table_ddl_request == Some(request_id) {
            self.table_state.table_ddl = Some(sql);
            self.table_state.ddl_execute_confirmation = false;
            self.table_state.table_ddl_error = None;
            self.table_state.table_ddl_request = None;
            self.feedback.runtime_message = "Table DDL loaded".to_owned();
        }
    }

    /// Table data arrived: seed filter/sort defaults on first load.
    pub(super) fn on_table_data_loaded(&mut self, request_id: RequestId, result: UiQueryResult, total_rows: u64) {
        if self.table_state.table_row_reload_request == Some(request_id) {
            self.on_table_row_reloaded(result);
            return;
        }
        if self.table_state.table_data_request != Some(request_id) {
            return;
        }
        if self.table_state.table_data_filter_column.is_empty() {
            self.table_state.table_data_filter_column = result
                .columns
                .first()
                .map(|column| column.name.clone())
                .unwrap_or_default();
        }
        if self.table_state.table_data_sorts.is_empty() {
            if let Some(column) = result.columns.first().map(|column| column.name.clone()) {
                self.table_state.table_data_sorts.push(UiTableDataSort {
                    column,
                    descending: false,
                });
            }
        }
        self.table_state.table_data_result = Some(result);
        self.invalidate_grid_row_caches();
        self.table_state.table_data_total_rows = Some(total_rows);
        if self.table_mutation.staged_changes.is_empty() {
            self.table_data.selected_cell = None;
            self.table_data.selected_row = None;
            self.table_data.selected_rows.clear();
            self.table_data.selection_anchor_row = None;
            self.table_data.selection_anchor_cell = None;
        }
        self.table_state.table_data_error = None;
        self.table_state.table_data_request = None;
        self.feedback.runtime_message = format!("Table data loaded · {total_rows} rows");
        if self.table_mutation.table_mutation_retry_after_reload {
            self.table_mutation.table_mutation_retry_after_reload = false;
            self.apply_staged_changes();
        }
    }

    pub(crate) fn on_table_row_reloaded(&mut self, result: UiQueryResult) {
        self.table_state.table_row_reload_request = None;
        let Some(identity) = self.table_state.table_row_reload_identity.take() else {
            return;
        };
        let Some(server_row) = result.rows.into_iter().next() else {
            self.feedback.runtime_message = "Row was deleted".to_owned();
            if self.table_mutation.table_mutation_retry_after_reload {
                self.table_mutation.table_mutation_retry_after_reload = false;
                self.table_mutation.table_mutation_retry_target = None;
            }
            return;
        };

        let mut replaced_row = false;
        if let Some(table_result) = self.table_state.table_data_result.as_mut() {
            let column_indexes: std::collections::HashMap<&str, usize> = table_result
                .columns
                .iter()
                .enumerate()
                .map(|(index, column)| (column.name.as_str(), index))
                .collect();
            if let Some(row_index) = table_result
                .rows
                .iter()
                .position(|row| Self::row_matches_identity(row, &column_indexes, &identity))
            {
                table_result.rows[row_index] = server_row;
                replaced_row = true;
            }
        }
        // Outside the borrow above: a replaced row invalidates both grid caches at once.
        if replaced_row {
            self.invalidate_grid_row_caches();
        }
        self.table_state.table_data_error = None;
        self.feedback.runtime_message = "Row reloaded from database".to_owned();
        if self.table_mutation.table_mutation_retry_after_reload {
            self.table_mutation.table_mutation_retry_after_reload = false;
            self.apply_staged_changes();
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
}
