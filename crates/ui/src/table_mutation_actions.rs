use super::*;

pub(crate) struct StagedApplyFailure<'a> {
    pub(crate) statement_index: usize,
    pub(crate) code: &'a str,
    pub(crate) message: &'a str,
    pub(crate) rolled_back: bool,
}

impl DbProApp {
    pub(crate) fn can_mutate_active_connection(&self) -> bool {
        table_editor_context::can_mutate_active_connection(&self.connection.catalog, &self.connection.lifecycle)
    }

    pub(crate) fn can_edit_table_rows(&self) -> bool {
        table_editor_context::can_edit_table_rows(
            &self.table.state,
            &self.connection.catalog,
            &self.connection.lifecycle,
        )
    }

    pub(crate) fn begin_data_cell_edit(
        &mut self,
        result: &UiQueryResult,
        row_index: usize,
        column_index: usize,
        cell: &UiCell,
    ) {
        if !self.can_edit_table_rows() {
            if self.can_mutate_active_connection() && !self.table.state.has_primary_key() {
                self.feedback.runtime_message = "Table has no primary key; safe row editing is unavailable.".to_owned();
            } else {
                self.feedback.runtime_message = "Connect with write access to edit rows".to_owned();
            }
            return;
        }
        if !self.can_mutate_active_connection() {
            self.feedback.runtime_message = "Connect with write access to edit rows".to_owned();
            return;
        }
        if self.staged_row_deleted(result, row_index) {
            self.feedback.runtime_message = "Discard the staged delete before editing this row".to_owned();
            return;
        }
        if let Some(block) = result
            .columns
            .get(column_index)
            .and_then(|column| self.table.state.column_write_policy(&column.name))
            .and_then(|policy| policy.write_block())
        {
            // Still allow the advanced inspector for binary / blocked columns (#228).
            self.open_cell_inspector(result, row_index, column_index);
            self.feedback.runtime_message = block.reason().to_owned();
            return;
        }
        self.table.data.selected_cell = Some((row_index, column_index));
        self.table.data.selected_row = Some(row_index);
        self.table.data.selected_rows.clear();
        self.table.data.selected_rows.insert(row_index);
        self.table.data.selection_anchor_row = Some(row_index);
        self.table.data.selection_anchor_cell = Some((row_index, column_index));
        if let Some(identity) =
            self.table
                .data
                .row_identity_for_result(result, self.table.state.table_info.as_ref(), row_index)
        {
            self.table
                .mutation
                .clear_error_for_identity(&identity, Some(column_index));
        }
        self.table.editing.data_editing_cell = Some((row_index, column_index));
        let should_expand = result.columns.get(column_index).is_some_and(|column| {
            let data_type = column.data_type.to_ascii_lowercase();
            data_type.contains("json")
                || matches!(cell, UiCell::Json(_))
                || matches!(cell, UiCell::Bytes(_))
                || matches!(cell, UiCell::Text(value) if value.chars().count() > 120)
        });
        if should_expand {
            self.open_cell_inspector(result, row_index, column_index);
        } else {
            self.table.editing.expanded_data_editor = None;
            self.table.editing.data_edit_error = None;
            self.table.editing.data_edit_value = match cell {
                UiCell::Null => "NULL".to_owned(),
                _ => crate::cell_text(cell),
            };
        }
        self.feedback.copy_status.clear();
    }

    pub(crate) fn submit_data_cell_edit(
        &mut self,
        result: &UiQueryResult,
        row_index: usize,
        column_index: usize,
    ) -> bool {
        self.table_mutation_context()
            .stage_data_cell_edit(result, row_index, column_index)
    }

    pub(crate) fn request_delete_selected_data_rows(&mut self, result: &UiQueryResult) {
        if !self.can_edit_table_rows() {
            if self.can_mutate_active_connection() && !self.table.state.has_primary_key() {
                self.feedback.runtime_message = "Table has no primary key; safe row editing is unavailable.".to_owned();
            } else {
                self.feedback.runtime_message = "Connect with write access to delete rows".to_owned();
            }
            return;
        }
        if !self.can_mutate_active_connection() {
            self.feedback.runtime_message = "Connect with write access to delete rows".to_owned();
            return;
        }
        self.table_mutation_context().stage_delete_selected_rows(result);
    }

    pub(crate) fn staged_cell_value(
        &self,
        result: &UiQueryResult,
        row_index: usize,
        column_index: usize,
    ) -> Option<UiCell> {
        table_editor_context::staged_cell_value(
            &self.table.data,
            &self.table.state,
            &self.table.mutation,
            result,
            row_index,
            column_index,
        )
    }

    pub(crate) fn staged_row_deleted(&self, result: &UiQueryResult, row_index: usize) -> bool {
        table_editor_context::staged_row_deleted(
            &self.table.data,
            &self.table.state,
            &self.table.mutation,
            result,
            row_index,
        )
    }

    pub(crate) fn revert_staged_cell(&mut self, result: &UiQueryResult, row_index: usize, column_index: usize) {
        self.table_mutation_context()
            .revert_staged_cell(result, row_index, column_index);
    }

    pub(crate) fn revert_staged_row(&mut self, result: &UiQueryResult, row_index: usize) {
        self.table_mutation_context().revert_staged_row(result, row_index);
    }

    pub(crate) fn discard_staged_changes(&mut self) {
        if self.table_mutation_context().discard_staged_changes() {
            self.request_table_data();
        }
    }

    pub(crate) fn table_mutation_context(&mut self) -> table_editor_context::TableMutationContext<'_> {
        table_editor_context::TableMutationContext::new(
            &mut self.table.state,
            &mut self.table.data_query,
            &mut self.table.data,
            &mut self.table.editing,
            &mut self.table.mutation,
            &mut self.feedback,
            self.schema.explorer.selected_table.as_deref(),
        )
    }

    pub(crate) fn discard_failed_mutation(&mut self, reload: bool) {
        let Some(target) = self
            .table
            .mutation
            .table_mutation_error
            .as_ref()
            .and_then(|failure| failure.target.clone())
        else {
            return;
        };
        match target {
            MutationTarget::Update { identity, columns, .. } => {
                for column_index in columns {
                    self.table.mutation.staged_changes.revert_cell(&identity, column_index);
                }
            }
            MutationTarget::Delete { identity, .. } => {
                self.table.mutation.staged_changes.revert_row(&identity);
            }
            MutationTarget::Insert => {}
        }
        self.table.mutation.table_mutation_error = None;
        self.table.mutation.table_mutation_retry_after_reload = false;
        self.table.mutation.table_mutation_retry_target = None;
        if reload {
            self.table.data_query.result = None;
            self.table.data_query.total_rows = None;
            self.table.data_query.error = None;
            self.request_table_data();
        }
    }

    pub(super) fn reload_failed_mutation(&mut self) {
        let target = self
            .table
            .mutation
            .table_mutation_error
            .as_ref()
            .and_then(|failure| failure.target.clone());
        if let Some(MutationTarget::Update { identity, .. } | MutationTarget::Delete { identity, .. }) = target {
            self.request_table_row_reload(identity);
            return;
        }
        self.table.mutation.table_mutation_error = None;
        self.table.data_query.result = None;
        self.table.data_query.total_rows = None;
        self.table.data_query.error = None;
        self.request_table_data();
    }

    pub(crate) fn retry_failed_mutation_after_reload(&mut self) {
        let Some(target) = self
            .table
            .mutation
            .table_mutation_error
            .as_ref()
            .and_then(|failure| failure.target.clone())
        else {
            self.feedback.runtime_message = "This failure has no retryable mutation target".to_owned();
            return;
        };
        self.table.mutation.table_mutation_retry_target = Some(target);
        self.table.mutation.table_mutation_retry_after_reload = true;
        self.reload_failed_mutation();
    }

    pub(crate) fn request_table_row_reload(&mut self, identity: RowIdentity) {
        let (Some(connection_id), Some(table)) = (
            self.connection.lifecycle.active_connection_id().map(str::to_owned),
            self.schema.explorer.selected_table.clone(),
        ) else {
            self.feedback.runtime_message = "Connect to a database before reloading the row".to_owned();
            return;
        };
        let Some(info) = self.table.state.table_info.as_ref() else {
            self.feedback.runtime_message = "Table structure is still loading".to_owned();
            return;
        };
        let filters = match TableMutationState::row_reload_filters(info, &identity) {
            Ok(filters) => filters,
            Err(error) => {
                self.feedback.runtime_message = error;
                return;
            }
        };
        let request_id = self.task_bridge.next_request_id();
        let command = self.table.data_query.load_row_command(
            request_id,
            connection_id,
            self.active_schema().to_owned(),
            table,
            filters,
        );
        if self.dispatch_command(command) {
            self.table.data_query.row_reload_request = Some(request_id);
            self.table.data_query.row_reload_identity = Some(identity);
            self.feedback.runtime_message = "Reloading row from database…".to_owned();
        }
    }

    pub(crate) fn apply_staged_changes(&mut self) {
        if self.table.mutation.staged_apply_request.is_some() {
            return;
        }
        if self.table.mutation.staged_changes.is_empty() {
            return;
        }
        if self.table.editing.data_edit_error.is_some() {
            self.feedback.runtime_message = "Fix the validation error before applying changes".to_owned();
            return;
        }
        let Some(connection) = self.active_connection().cloned() else {
            self.feedback.runtime_message = "Connect to a database before applying changes".to_owned();
            return;
        };
        let Some(table) = self.schema.explorer.selected_table.clone() else {
            self.feedback.runtime_message = "Select a table before applying changes".to_owned();
            return;
        };
        let Some(plan) = self.table_mutation_context().prepare_apply(&table) else {
            return;
        };
        let request_id = self.task_bridge.next_request_id();
        let command = UiCommand::ApplyTableChanges {
            request_id,
            connection_id: connection.id,
            schema: self.active_schema().to_owned(),
            table,
            changes: plan.changes,
        };
        if self.dispatch_command(command) {
            self.table.mutation.staged_apply_request = Some(request_id);
            self.table.mutation.table_mutation_request = Some(request_id);
            self.table.mutation.staged_apply_targets = plan.targets;
            self.table.mutation.table_mutation_error = None;
            let counts = self.table.mutation.staged_changes.counts();
            self.feedback.runtime_message = format!(
                "Applying {} changes in one transaction (+{} ~{} -{})…",
                counts.total(),
                counts.inserts,
                counts.updates,
                counts.deletes
            );
        } else {
            self.feedback.runtime_message = "Could not send staged change to runtime".to_owned();
        }
    }

    pub(crate) fn staged_apply_completed(&mut self) {
        self.table.mutation.staged_apply_request = None;
        self.table.mutation.table_mutation_request = None;
        self.table.mutation.table_mutation_retry_after_reload = false;
        self.table.mutation.table_mutation_retry_target = None;
        self.table.mutation.staged_changes.clear();
        self.table.mutation.staged_apply_targets.clear();
        self.table.mutation.table_mutation_error = None;
        self.feedback.runtime_message = "All staged changes applied".to_owned();
        self.feedback
            .show_success_toast("All staged changes applied successfully");
        if let Some(action) = self.workspace.pending_navigation_action.take() {
            self.execute_pending_navigation(action);
            return;
        }
        self.table.data_query.result = None;
        self.table.data_query.total_rows = None;
        self.table.data_query.error = None;
        self.request_table_data();
    }

    pub(crate) fn staged_apply_failed(&mut self, failure: StagedApplyFailure<'_>) {
        self.workspace.pending_navigation_action = None;
        self.table.mutation.reset_apply_lifecycle();
        let transition = self.table.mutation.record_apply_failure(failure);
        if let Some(target) = transition.target() {
            self.table_mutation_context().select_failed_target(target);
        }
        if transition.is_conflict() {
            self.table.mutation.conflict_dialog_open = true;
            if let Some(identity) = transition.reload_identity() {
                self.request_table_row_reload(identity.clone());
            }
        }
        let outcome = if transition.is_rolled_back() {
            "transaction rolled back"
        } else {
            "transaction outcome is unknown"
        };
        let formatted = if transition.has_target() {
            format!(
                "Staged change #{} failed · {outcome} · {}",
                transition.statement_index().saturating_add(1),
                transition.display_message()
            )
        } else {
            format!("Staged changes failed · {outcome} · {}", transition.display_message())
        };
        self.feedback.runtime_message = formatted.clone();
        self.feedback.show_error_toast(formatted);
    }
}
