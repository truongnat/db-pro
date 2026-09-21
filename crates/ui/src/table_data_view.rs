use super::*;

impl DbProApp {
    pub(crate) fn draw_table_data(&mut self, ui: &mut egui::Ui, table_name: &str) {
        // Temporarily move the result out while rendering. The grid mutates
        // selection/edit state, so borrowing it directly from `self` would
        // conflict with those updates; moving avoids cloning every frame.
        let Some(result) = self.table.data_query.result.take() else {
            self.draw_table_data_placeholder(ui, table_name);
            return;
        };

        let can_mutate = self.can_mutate_active_connection();
        let total_known = self.table.data_query.total_rows.is_some();
        let total_rows = self.table.data_query.total_rows.unwrap_or(result.row_count);
        let paging = table_data_toolbar_surface_view::TableDataPaging {
            page_range: crate::components::common_utils::format_page_range(
                self.table.data_query.offset,
                result.row_count,
                self.table.data_query.total_rows,
            ),
            total_rows,
            total_known,
            has_next: if total_known {
                self.table.data_query.offset.saturating_add(self.table.data_query.limit) < total_rows
            } else {
                result.row_count >= self.table.data_query.limit
            },
            has_previous: self.table.data_query.offset > 0,
        };

        self.draw_table_data_unified_toolbar(ui, table_name, &result, can_mutate, &paging);
        ui.add_space(4.0);
        let data_width = ui.max_rect().width();
        grid_frame(self.theme).show(ui, |ui| {
            ui.set_min_width(data_width.max(0.0));
            self.draw_result_grid(ui, &result);
        });
        self.draw_pending_changes_dialog(ui);
        self.draw_conflict_dialog(ui, &result);
        if self.table.data_query.result.is_none() {
            self.table.data_query.result = Some(result);
        }
    }

    fn draw_table_data_placeholder(&mut self, ui: &mut egui::Ui, table_name: &str) {
        let context = table_data_placeholder_view::TableDataPlaceholderContext {
            theme: self.theme,
            error: self.table.data_query.error.as_deref(),
        };
        if matches!(
            table_data_placeholder_view::draw_placeholder(&context, ui, table_name),
            Some(table_data_placeholder_view::TableDataPlaceholderAction::Retry)
        ) {
            self.table.data_query.error = None;
            self.request_table_data();
        }
    }

    fn draw_table_data_unified_toolbar(
        &mut self,
        ui: &mut egui::Ui,
        table_name: &str,
        result: &UiQueryResult,
        can_mutate: bool,
        paging: &table_data_toolbar_surface_view::TableDataPaging,
    ) {
        let column_names: Vec<String> = result.columns.iter().map(|column| column.name.clone()).collect();
        let action = {
            let mut context = table_data_toolbar_surface_view::TableDataToolbarContext {
                theme: self.theme,
                table_name,
                result,
                column_names: &column_names,
                can_mutate,
                connected: self.connection.lifecycle.is_connected(),
                has_primary_key: self.table.state.has_primary_key(),
                staged_changes: &self.table.mutation.staged_changes,
                staged_apply_pending: self.table.mutation.staged_apply_request.is_some(),
                has_data_edit_error: self.table.editing.data_edit_error.is_some(),
                failure: self.table.mutation.table_mutation_error.as_ref(),
                selected_rows: self.table.data.selected_rows.len(),
                data_query: &mut self.table.data_query,
                data: &mut self.table.data,
                table_info: self.table.state.table_info.as_ref(),
                paging,
            };
            table_data_toolbar_surface_view::draw_toolbar(&mut context, ui)
        };
        if let Some(action) = action {
            self.apply_table_data_toolbar_action(action);
        }
    }

    fn apply_table_data_toolbar_action(&mut self, action: table_data_toolbar_surface_view::TableDataToolbarAction) {
        use table_data_toolbar_surface_view::TableDataToolbarAction as Action;
        match action {
            Action::RequestData => self.request_table_data(),
            Action::RefreshBlocked => {
                self.feedback.runtime_message = "Apply or discard staged changes before refreshing".to_owned();
            }
            Action::AddRow => self.open_insert_row(),
            Action::OpenPendingChanges => self.table.mutation.pending_changes_open = true,
            Action::ApplyStagedChanges => self.apply_staged_changes(),
            Action::ConfirmDiscardChanges => self.table.editing.discard_changes_confirmation = true,
            Action::DiscardStagedChanges => self.discard_staged_changes(),
            Action::ReloadFailedMutation => self.reload_failed_mutation(),
            Action::DiscardFailedMutation => self.discard_failed_mutation(false),
            Action::ResolveConflict => self.table.mutation.conflict_dialog_open = true,
            Action::RetryFailedMutation => self.retry_failed_mutation_after_reload(),
            Action::CommitFilter => self.commit_table_filter_draft(),
            Action::RemoveFilter(index) => self.remove_table_filter(index),
            Action::ClearFilters => self.clear_table_filters(),
            Action::ReloadFromStart => self.reload_table_data_from_start(),
            Action::SortBlocked => {
                self.feedback.runtime_message = "Apply or discard staged changes before changing sort".to_owned();
            }
            Action::ResetPage => self.reset_table_data_page(),
        }
    }
}
