//! Result-grid cell painting and context menus.
use super::result_grid_view::{GridCell, GridSelectionLookup};
use super::*;

struct GridCellInteraction<'a> {
    result: &'a UiQueryResult,
    visible_indexes: &'a [usize],
    selection_lookup: &'a GridSelectionLookup,
    row_index: usize,
    column_index: usize,
    editable: bool,
    display_cell: &'a UiCell,
}

impl DbProApp {
    pub(super) fn draw_grid_cell(&mut self, ui: &mut egui::Ui, result: &UiQueryResult, cell_ctx: GridCell<'_>) {
        let GridCell {
            visible_indexes,
            selection_lookup,
            row_index,
            column_index,
            display_position,
            row_selected,
            row_dirty,
            row_mutation_error,
            cell_mutation_error,
            editable,
            width,
            cell,
        } = cell_ctx;

        let staged_cell = self.staged_cell_value(result, row_index, column_index);
        let display_cell = staged_cell.as_ref().unwrap_or(cell);
        let cell_selected = self
            .table
            .data
            .is_cell_selected(selection_lookup, (row_index, column_index));
        let editing = editable && self.table.editing.data_editing_cell == Some((row_index, column_index));
        let validation_error = editing && self.table.editing.data_edit_error.is_some();
        let conflict_error = (cell_mutation_error || row_mutation_error)
            && self
                .table
                .mutation
                .table_mutation_error
                .as_ref()
                .is_some_and(|failure| failure.code == "CONFLICT");

        let (cell_rect, cell_resp) = ui.allocate_exact_size(egui::vec2(width, 28.0), Sense::click());

        let surface_context = result_grid_cell_surface_view::GridCellSurfaceContext {
            theme: self.theme,
            row_selected,
            cell_selected,
            row_dirty,
            display_position,
            validation_error,
            conflict_error,
            cell_mutation_error,
            row_mutation_error,
            edit_error: self.table.editing.data_edit_error.as_deref(),
            mutation_error: self
                .table
                .mutation
                .table_mutation_error
                .as_ref()
                .map(|error| error.message.as_str()),
        };
        result_grid_cell_surface_view::draw_surface(&surface_context, ui, cell_rect, &cell_resp);

        if editing {
            self.draw_grid_cell_editor(ui, result, row_index, column_index, cell_rect);
        } else {
            result_grid_cell_surface_view::draw_value(self.theme, ui, cell_rect, &cell_resp, display_cell);

            self.handle_grid_cell_interaction(
                ui,
                &cell_resp,
                GridCellInteraction {
                    result,
                    visible_indexes,
                    selection_lookup,
                    row_index,
                    column_index,
                    editable,
                    display_cell,
                },
            );
        }
    }

    fn handle_grid_cell_interaction(
        &mut self,
        ui: &mut egui::Ui,
        cell_response: &egui::Response,
        interaction: GridCellInteraction<'_>,
    ) {
        let is_context_menu = self.run_grid_cell_context_menu(ui, cell_response, &interaction);
        if cell_response.double_clicked() && interaction.editable {
            if self.table.editing.data_editing_cell.is_some() && !self.commit_active_data_edit(interaction.result) {
                return;
            }
            self.begin_data_cell_edit(
                interaction.result,
                interaction.row_index,
                interaction.column_index,
                interaction.display_cell,
            );
        } else if cell_response.clicked() && !is_context_menu {
            if self.table.editing.data_editing_cell.is_some() && !self.commit_active_data_edit(interaction.result) {
                return;
            }
            let modifiers = ui.input(|input| input.modifiers);
            self.table.data.select_cell_range(
                interaction.visible_indexes,
                &interaction.selection_lookup.row_positions,
                (interaction.row_index, interaction.column_index),
                modifiers.shift,
            );
            self.feedback.copy_status.clear();
        }
    }

    // allow: grid cell context menu requires full context (result, selection, row/col, editable)
    // to paint and dispatch; kept flat for direct readability at render call site.
    fn run_grid_cell_context_menu(
        &mut self,
        ui: &mut egui::Ui,
        cell_resp: &egui::Response,
        interaction: &GridCellInteraction<'_>,
    ) -> bool {
        let GridCellInteraction {
            result,
            selection_lookup,
            row_index,
            column_index,
            editable,
            ..
        } = *interaction;
        let is_ctx = is_context_menu_triggered(cell_resp, ui);
        if is_ctx
            && self
                .table
                .editing
                .data_editing_cell
                .is_some_and(|editing_cell| editing_cell != (row_index, column_index))
            && !self.commit_active_data_edit(result)
        {
            return is_ctx;
        }

        if is_ctx {
            // Keep an existing rectangular selection when the context menu
            // is opened inside it. Right-clicking outside the range starts
            // a new selection at the clicked cell.
            let is_inside_range = self
                .table
                .data
                .is_cell_selected(selection_lookup, (row_index, column_index));
            if !is_inside_range {
                self.table.data.select_single_cell((row_index, column_index));
            }
            if !self.table.data.selected_rows.contains(&row_index) {
                self.table.data.select_single_row(row_index);
            } else if !is_inside_range {
                self.table.data.selected_row = Some(row_index);
                self.table.data.selection_anchor_row = Some(row_index);
            }
        }
        let write_block_reason = self
            .blocked_write_for_cell(result, column_index)
            .map(|block| block.reason());
        let has_staged_cell = self.staged_cell_value(result, row_index, column_index).is_some();
        let has_staged_row = self
            .table
            .data
            .row_identity_for_result(result, self.table.state.table_info.as_ref(), row_index)
            .as_ref()
            .map(|identity| self.table.mutation.staged_changes.row_has_changes(identity))
            .unwrap_or(false);
        let menu_context = result_grid_cell_menu_view::GridCellMenuContext {
            theme: self.theme,
            editable,
            write_block_reason,
            has_staged_cell,
            has_staged_row,
            row_deleted: self.staged_row_deleted(result, row_index),
        };
        let action = result_grid_cell_menu_view::draw_menu(&menu_context, ui, cell_resp);
        self.apply_grid_cell_menu_action(ui, interaction, action);
        is_ctx
    }

    fn apply_grid_cell_menu_action(
        &mut self,
        ui: &mut egui::Ui,
        interaction: &GridCellInteraction<'_>,
        action: Option<result_grid_cell_menu_view::GridCellMenuAction>,
    ) {
        let GridCellInteraction {
            result,
            row_index,
            column_index,
            editable,
            display_cell,
            ..
        } = *interaction;
        let Some(action) = action else {
            return;
        };
        use result_grid_cell_menu_view::GridCellMenuAction as Action;
        match action {
            Action::CopyCell => self.copy_cell_at(ui, result, row_index, column_index),
            Action::CopyRow => {
                self.table.data.select_single_row(row_index);
                self.copy_selected_row(ui, result);
            }
            Action::CopySelectedRows => {
                self.ensure_selected_row(row_index);
                self.copy_selected_rows(ui, result);
            }
            Action::CopySelectedRowsHeaders => {
                self.ensure_selected_row(row_index);
                self.copy_selected_rows_with_headers(ui, result);
            }
            Action::CopySelectedRowsJson => {
                self.ensure_selected_row(row_index);
                self.copy_selected_rows_as_json(ui, result);
            }
            Action::CopySelectedRowsInsert => {
                self.ensure_selected_row(row_index);
                self.copy_selected_rows_as_insert(ui, result);
            }
            Action::CopyJson => {
                self.table.data.selected_row = Some(row_index);
                self.copy_row_as_json(ui, result, row_index);
            }
            Action::CopyCsv => {
                self.table.data.selected_row = Some(row_index);
                self.copy_row_as_csv(ui, result, row_index);
            }
            Action::EditCell if editable => self.begin_data_cell_edit(result, row_index, column_index, display_cell),
            Action::SetNull if editable => {
                self.table.editing.data_editing_cell = Some((row_index, column_index));
                self.table.editing.data_edit_value = "NULL".to_owned();
                self.table.editing.data_edit_error = None;
                self.submit_data_cell_edit(result, row_index, column_index);
            }
            Action::RevertCell if editable => self.revert_staged_cell(result, row_index, column_index),
            Action::RevertRow if editable => self.revert_staged_row(result, row_index),
            Action::DuplicateRow if editable => self.open_duplicate_row(result, row_index),
            Action::DeleteRow if editable => {
                self.ensure_selected_row(row_index);
                self.request_delete_selected_data_rows(result);
            }
            Action::FilterThisValue => self.apply_cell_filter(result, column_index, display_cell),
            Action::SortAscending => self.set_table_or_grid_sort(result, column_index, Some(false)),
            Action::SortDescending => self.set_table_or_grid_sort(result, column_index, Some(true)),
            Action::EditCell
            | Action::SetNull
            | Action::RevertCell
            | Action::RevertRow
            | Action::DuplicateRow
            | Action::DeleteRow => {}
        }
    }

    fn ensure_selected_row(&mut self, row_index: usize) {
        if !self.table.data.selected_rows.contains(&row_index) {
            self.table.data.select_single_row(row_index);
        }
    }

    fn apply_cell_filter(&mut self, result: &UiQueryResult, column_index: usize, display_cell: &UiCell) {
        if self.workspace.active_tab == WorkspaceTab::Table && self.table.state.table_view == TableView::Data {
            self.table.data_query.filter_column = result
                .columns
                .get(column_index)
                .map(|column| column.name.clone())
                .unwrap_or_default();
            if matches!(display_cell, UiCell::Null) {
                self.table.data_query.filter_operator = UiTableFilterOperator::IsNull;
                self.table.data_query.filter_value.clear();
            } else {
                self.table.data_query.filter_operator = UiTableFilterOperator::Equals;
                self.table.data_query.filter_value = crate::cell_text(display_cell);
            }
            self.commit_table_filter_draft();
        } else {
            self.table.data.grid_filter = crate::cell_text(display_cell);
        }
    }
}
