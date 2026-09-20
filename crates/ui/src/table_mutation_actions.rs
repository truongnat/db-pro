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
        let Some(info) = self.table.state.table_info.clone() else {
            self.feedback.runtime_message = "Table structure is still loading".to_owned();
            return false;
        };
        let Some(column) = result.columns.get(column_index).map(|column| column.name.clone()) else {
            self.table.editing.data_editing_cell = None;
            return false;
        };
        let Some(column_info) = info.columns.iter().find(|item| item.name == column) else {
            self.feedback.runtime_message = "The selected column is not present in the table metadata".to_owned();
            self.table.editing.data_editing_cell = None;
            return false;
        };
        if let Some(block) = ColumnWritePolicy::read(column_info).write_block() {
            let error = block.reason().to_owned();
            self.table.editing.data_edit_error = Some(error.clone());
            self.feedback.runtime_message = format!("{}: {error}", column_info.name);
            return false;
        }
        let value = match table_editor_values::parse_update_value(
            &self.table.editing.data_edit_value,
            &column_info.data_type,
        ) {
            Ok(value) => value,
            Err(error) => {
                self.feedback.runtime_message = format!("{}: {error}", column_info.name);
                self.table.editing.data_edit_error = Some(error);
                return false;
            }
        };
        if matches!(value, UiCell::Null) && !column_info.nullable {
            let error = format!("{} is NOT NULL; enter a value instead", column_info.name);
            self.feedback.runtime_message = error.clone();
            self.table.editing.data_edit_error = Some(error);
            return false;
        }
        let identity = match TableDataState::row_identity(result, &info, row_index) {
            Ok(identity) => identity,
            Err(error) => {
                self.feedback.runtime_message = error;
                self.table.editing.data_edit_error = Some(self.feedback.runtime_message.clone());
                return false;
            }
        };
        let original = result
            .rows
            .get(row_index)
            .and_then(|row| row.get(column_index))
            .cloned()
            .ok_or_else(|| "The selected cell is no longer available".to_owned());
        let Ok(original) = original else {
            self.table.editing.data_editing_cell = None;
            self.feedback.runtime_message = "The selected cell is no longer available".to_owned();
            self.table.editing.data_edit_error = Some(self.feedback.runtime_message.clone());
            return false;
        };
        if let Some(table) = self.schema.explorer.selected_table.as_deref() {
            self.table.mutation.staged_changes.ensure_target(table);
        }
        self.table.mutation.staged_changes.stage_update(StagedChange::Update {
            identity,
            current_row_index: Some(row_index),
            column_index,
            column,
            data_type: column_info.data_type.clone(),
            original,
            value,
        });
        self.table.mutation.table_mutation_error = None;
        self.table.mutation.staged_apply_targets.clear();
        self.table.editing.data_editing_cell = None;
        self.table.editing.expanded_data_editor = None;
        self.table.editing.data_edit_error = None;
        let counts = self.table.mutation.staged_changes.counts();
        self.feedback.runtime_message = format!(
            "Staged edit · {} pending (+{} ~{} -{})",
            counts.total(),
            counts.inserts,
            counts.updates,
            counts.deletes
        );
        true
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
        let row_indexes: Vec<usize> = if self.table.data.selected_rows.is_empty() {
            self.table.data.selected_row.into_iter().collect()
        } else {
            self.table.data.selected_rows.iter().copied().collect()
        };
        if row_indexes.is_empty() {
            self.feedback.runtime_message = "Select a row before deleting".to_owned();
            return;
        }
        let Some(info) = self.table.state.table_info.clone() else {
            self.feedback.runtime_message = "Table structure is still loading".to_owned();
            return;
        };
        if let Some(table) = self.schema.explorer.selected_table.as_deref() {
            self.table.mutation.staged_changes.ensure_target(table);
        }
        self.table.editing.data_editing_cell = None;
        self.table.editing.expanded_data_editor = None;
        self.table.editing.data_edit_value.clear();
        self.table.editing.data_edit_error = None;
        self.table.editing.data_delete_confirmation = false;
        for row_index in row_indexes {
            if self.staged_row_deleted(result, row_index) {
                continue;
            }
            let identity = match TableDataState::row_identity(result, &info, row_index) {
                Ok(identity) => identity,
                Err(error) => {
                    self.feedback.runtime_message = error;
                    return;
                }
            };
            self.table.mutation.staged_changes.stage_delete(StagedChange::Delete {
                identity,
                current_row_index: Some(row_index),
            });
        }
        self.table.mutation.table_mutation_error = None;
        self.table.mutation.staged_apply_targets.clear();
        self.feedback.runtime_message = format!(
            "{} row(s) marked for deletion · {} staged change(s)",
            self.table
                .mutation
                .staged_changes
                .iter()
                .filter(|change| matches!(change, StagedChange::Delete { .. }))
                .count(),
            self.table.mutation.staged_changes.counts().total()
        );
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

    fn table_mutation_context(&mut self) -> table_editor_context::TableMutationContext<'_> {
        table_editor_context::TableMutationContext::new(
            &mut self.table.state,
            &mut self.table.data_query,
            &mut self.table.data,
            &mut self.table.editing,
            &mut self.table.mutation,
            &mut self.feedback,
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
        self.table.data_query.row_reload_request = Some(request_id);
        self.table.data_query.row_reload_identity = Some(identity);
        self.dispatch_command(self.table.data_query.load_row_command(
            request_id,
            connection_id,
            self.active_schema().to_owned(),
            table,
            filters,
        ));
        self.feedback.runtime_message = "Reloading row from database…".to_owned();
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
        if let Some(target) = self.table.mutation.staged_changes.target_table() {
            if target != table {
                self.feedback.runtime_message = format!("Staged changes belong to table `{target}`, not `{table}`");
                return;
            }
        }
        let plan = self.table.mutation.build_apply_plan();
        if plan.changes.is_empty() {
            self.table.mutation.table_mutation_retry_after_reload = false;
            self.feedback.runtime_message = "The related staged change is no longer available".to_owned();
            return;
        }
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
        let StagedApplyFailure {
            statement_index,
            code,
            message,
            rolled_back,
        } = failure;
        self.workspace.pending_navigation_action = None;
        self.table.mutation.staged_apply_request = None;
        self.table.mutation.table_mutation_request = None;
        self.table.mutation.table_mutation_retry_after_reload = false;
        self.table.mutation.table_mutation_retry_target = None;
        let target = self.table.mutation.staged_apply_targets.get(statement_index).cloned();
        let has_target = target.is_some();
        if let Some(target) = target.as_ref() {
            match target {
                MutationTarget::Update {
                    identity,
                    current_row_index,
                    columns,
                } => {
                    let row_index = self.current_row_index_for_identity(identity, *current_row_index);
                    self.table.data.selected_row = row_index;
                    self.table.data.selected_rows.clear();
                    if let Some(row_index) = row_index {
                        self.table.data.selected_rows.insert(row_index);
                        if let Some(column_index) = columns.first().copied() {
                            self.table.data.selected_cell = Some((row_index, column_index));
                            self.table.data.selection_anchor_cell = Some((row_index, column_index));
                        }
                    }
                }
                MutationTarget::Delete {
                    identity,
                    current_row_index,
                } => {
                    let row_index = self.current_row_index_for_identity(identity, *current_row_index);
                    self.table.data.selected_row = row_index;
                    self.table.data.selected_rows.clear();
                    if let Some(row_index) = row_index {
                        self.table.data.selected_rows.insert(row_index);
                    }
                    self.table.data.selected_cell = None;
                    self.table.data.selection_anchor_cell = None;
                }
                MutationTarget::Insert => {}
            }
        }
        let normalized_code = match code {
            "INTERNAL_ERROR" => "INTERNAL",
            "CONSTRAINT_VIOLATION" => "CONSTRAINT_VIOLATION",
            "VALIDATION_ERROR" => "VALIDATION_ERROR",
            "CONFLICT" => "CONFLICT",
            _ => "INTERNAL",
        };
        let display_message = if normalized_code == "CONFLICT" {
            format!("This row changed or was deleted in the database. Database: {message}")
        } else {
            message.to_owned()
        };
        let mutation_failure = MutationFailure {
            statement_index,
            target: target.clone(),
            code: normalized_code.to_owned(),
            message: display_message.clone(),
            rolled_back,
        };
        self.table.mutation.table_mutation_error = Some(mutation_failure);
        if normalized_code == "CONFLICT" {
            self.table.mutation.conflict_dialog_open = true;
            if let Some(MutationTarget::Update { identity, .. } | MutationTarget::Delete { identity, .. }) =
                target.as_ref()
            {
                self.request_table_row_reload(identity.clone());
            }
        }
        let outcome = if rolled_back {
            "transaction rolled back"
        } else {
            "transaction outcome is unknown"
        };
        let formatted = if has_target {
            format!(
                "Staged change #{} failed · {outcome} · {display_message}",
                statement_index.saturating_add(1)
            )
        } else {
            format!("Staged changes failed · {outcome} · {display_message}")
        };
        self.feedback.runtime_message = formatted.clone();
        self.feedback.show_error_toast(formatted);
    }

    pub(crate) fn current_row_index_for_identity(
        &self,
        identity: &RowIdentity,
        fallback: Option<usize>,
    ) -> Option<usize> {
        self.table
            .data_query
            .result
            .as_ref()
            .and_then(|result| {
                result.rows.iter().enumerate().find_map(|(row_index, _)| {
                    (self
                        .table
                        .data
                        .row_identity_for_result(result, self.table.state.table_info.as_ref(), row_index)
                        .as_ref()
                        == Some(identity))
                    .then_some(row_index)
                })
            })
            .or(fallback)
    }
}
