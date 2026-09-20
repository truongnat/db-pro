pub use super::result_grid_projection::GridSelectionLookup;
use super::*;
use crate::GridProjectionKey;
use egui::Vec2;

/// Per-cell render context for the result grid.
pub(crate) struct GridCell<'a> {
    pub(crate) visible_indexes: &'a [usize],
    pub(crate) selection_lookup: &'a GridSelectionLookup,
    pub(crate) row_index: usize,
    pub(crate) column_index: usize,
    pub(crate) display_position: usize,
    pub(crate) row_selected: bool,
    pub(crate) row_dirty: bool,
    pub(crate) row_mutation_error: bool,
    pub(crate) cell_mutation_error: bool,
    pub(crate) editable: bool,
    pub(crate) width: f32,
    pub(crate) cell: &'a UiCell,
}

/// Context shared by every visible grid row.
pub(crate) struct GridRows<'a> {
    pub(crate) indexes: &'a [usize],
    pub(crate) widths: &'a [f32],
    pub(crate) order: &'a [usize],
    pub(crate) editable: bool,
    pub(crate) row_offset: u64,
    pub(crate) selection_lookup: &'a GridSelectionLookup,
}

struct GridKeyboardApplicationContext<'a> {
    result: &'a UiQueryResult,
    indexes: &'a [usize],
    order: &'a [usize],
    editable: bool,
    selection_lookup: &'a GridSelectionLookup,
}

impl DbProApp {
    pub(super) fn draw_result_grid(&mut self, ui: &mut egui::Ui, result: &UiQueryResult) {
        if result.columns.is_empty() {
            ui.centered_and_justified(|ui| {
                empty_state(
                    ui,
                    Icon::CircleCheck,
                    "Statement completed",
                    "This statement did not return any rows to display.",
                    self.theme,
                );
            });
            return;
        }

        let is_table_data =
            self.workspace.active_tab == WorkspaceTab::Table && self.table.state.table_view == TableView::Data;
        let editable = is_table_data && self.can_edit_table_rows();
        let (projection_key, indexes, order, selection_lookup) = self.prepare_grid_cache(result, is_table_data);

        self.handle_grid_keyboard(ui, result, &indexes, &order, editable, &selection_lookup);

        if !is_table_data {
            self.draw_result_grid_toolbar(ui, result, &indexes, editable);
        }
        self.draw_record_inspector_panel(ui, result);

        let row_offset = if is_table_data { self.table.data_query.offset } else { 0 };
        self.draw_grid_body(ui, result, &indexes, &order, editable, row_offset, &selection_lookup);

        self.restore_grid_cache(projection_key, indexes, order, selection_lookup);
    }

    fn draw_result_grid_toolbar(
        &mut self,
        ui: &mut egui::Ui,
        result: &UiQueryResult,
        indexes: &[usize],
        editable: bool,
    ) {
        let action = {
            let mut context = result_grid_toolbar_view::ResultGridToolbarContext {
                theme: self.theme,
                data: &mut self.table.data,
                editing: &mut self.table.editing,
                feedback: &self.feedback,
                editable,
                matching_rows: indexes.len(),
            };
            result_grid_toolbar_view::draw_toolbar(&mut context, ui)
        };
        if let Some(action) = action {
            match action {
                result_grid_toolbar_view::ResultGridToolbarAction::CopySelectedCell => {
                    self.copy_selected_cell(ui, result);
                }
                result_grid_toolbar_view::ResultGridToolbarAction::CopySelectedRow => {
                    self.copy_selected_row(ui, result);
                }
                result_grid_toolbar_view::ResultGridToolbarAction::CopyVisibleCsv => {
                    self.copy_all_as_csv(ui, result, indexes);
                }
                result_grid_toolbar_view::ResultGridToolbarAction::CopyVisibleJson => {
                    self.copy_all_as_json(ui, result, indexes);
                }
                result_grid_toolbar_view::ResultGridToolbarAction::InspectSelectedCell {
                    row_index,
                    column_index,
                } => self.open_cell_inspector(result, row_index, column_index),
            }
        }
    }

    fn prepare_grid_cache(
        &mut self,
        result: &UiQueryResult,
        is_table_data: bool,
    ) -> (GridProjectionKey, Vec<usize>, Vec<usize>, GridSelectionLookup) {
        let order = self.table.data.column_order_for_columns(&result.columns);
        let projection_key = self.table.data.projection_key(result);
        let indexes = self
            .table
            .data
            .grid_projection_cache
            .take(projection_key.clone(), result);
        let selection_lookup = self
            .table
            .data
            .grid_selection_cache
            .take(&projection_key, &order)
            .unwrap_or_else(|| GridSelectionLookup::new(&indexes, &order));

        if is_table_data {
            self.table
                .data
                .rebuild_row_identity_cache(result, self.table.state.table_info.as_ref());
        } else {
            self.table.data.grid_row_identity_cache.clear();
            self.table.data.grid_row_identity_cache_ready = false;
        }

        (projection_key, indexes, order, selection_lookup)
    }

    fn restore_grid_cache(
        &mut self,
        projection_key: GridProjectionKey,
        indexes: Vec<usize>,
        order: Vec<usize>,
        selection_lookup: GridSelectionLookup,
    ) {
        self.table
            .data
            .grid_projection_cache
            .restore(projection_key.clone(), indexes);
        self.table
            .data
            .grid_selection_cache
            .restore(projection_key, order, selection_lookup);
    }

    pub(crate) fn set_table_or_grid_sort(
        &mut self,
        result: &UiQueryResult,
        column_index: usize,
        descending: Option<bool>,
    ) {
        if self.workspace.active_tab == WorkspaceTab::Table && self.table.state.table_view == TableView::Data {
            if !self.table.mutation.staged_changes.is_empty() {
                self.feedback.runtime_message = "Apply or discard staged changes before changing sort".to_owned();
                return;
            }
            self.table.data_query.sorts = descending
                .and_then(|_| {
                    result.columns.get(column_index).map(|column| UiTableDataSort {
                        column: column.name.clone(),
                        descending: descending.unwrap_or(false),
                    })
                })
                .into_iter()
                .collect();
            self.table.data.grid_sort_column = None;
            self.table.data.grid_sort_desc = false;
            self.reload_table_data_from_start();
        } else {
            self.table.data.grid_sort_column = descending.map(|_| column_index);
            self.table.data.grid_sort_desc = descending.unwrap_or(false);
        }
    }

    /// Cycle a table-data sort clause. Shift-click keeps other clauses and
    /// makes the clicked column the next priority; plain click selects one
    /// clause and cycles ASC -> DESC -> none.
    pub(crate) fn cycle_table_data_sort(&mut self, result: &UiQueryResult, column_index: usize, additive: bool) {
        if !self.table.mutation.staged_changes.is_empty() {
            self.feedback.runtime_message = "Apply or discard staged changes before changing sort".to_owned();
            return;
        }
        let Some(column) = result.columns.get(column_index).map(|column| column.name.clone()) else {
            return;
        };
        if additive {
            if let Some(index) = self
                .table
                .data_query
                .sorts
                .iter()
                .position(|sort| sort.column == column)
            {
                if self.table.data_query.sorts[index].descending {
                    self.table.data_query.sorts.remove(index);
                } else {
                    self.table.data_query.sorts[index].descending = true;
                }
            } else {
                self.table.data_query.sorts.push(UiTableDataSort {
                    column,
                    descending: false,
                });
            }
        } else if self.table.data_query.sorts.len() == 1
            && self.table.data_query.sorts.first().map(|sort| sort.column.as_str()) == Some(column.as_str())
        {
            if self.table.data_query.sorts[0].descending {
                self.table.data_query.sorts.clear();
            } else {
                self.table.data_query.sorts[0].descending = true;
            }
        } else {
            self.table.data_query.sorts = vec![UiTableDataSort {
                column,
                descending: false,
            }];
        }
        self.table.data.grid_sort_column = None;
        self.table.data.grid_sort_desc = false;
        self.reload_table_data_from_start();
    }

    /// Copy, paste-to-edit, staged-changes, and keyboard navigation for the result grid.
    pub(crate) fn handle_grid_keyboard(
        &mut self,
        ui: &mut egui::Ui,
        result: &UiQueryResult,
        indexes: &[usize],
        order: &[usize],
        editable: bool,
        selection_lookup: &GridSelectionLookup,
    ) {
        let intent = result_grid_keyboard_view::read_keyboard_intent(
            ui,
            result_grid_keyboard_view::GridKeyboardInputContext {
                editable,
                editing_cell: self.table.editing.data_editing_cell.is_some(),
                connection_dialog_open: self.connection.dialog.is_open(),
            },
        );
        match intent {
            result_grid_keyboard_view::GridKeyboardIntent::SelectAll => {
                self.table.data.select_all_visible_cells(indexes, order);
                self.feedback.copy_status.clear();
            }
            result_grid_keyboard_view::GridKeyboardIntent::ClearSelection => {
                self.clear_grid_selection();
            }
            result_grid_keyboard_view::GridKeyboardIntent::Commands(commands) => {
                self.apply_grid_keyboard_commands(
                    ui,
                    GridKeyboardApplicationContext {
                        result,
                        indexes,
                        order,
                        editable,
                        selection_lookup,
                    },
                    commands,
                );
            }
        }
    }

    fn clear_grid_selection(&mut self) {
        self.table.data.selected_cell = None;
        self.table.data.selected_row = None;
        self.table.data.selected_rows.clear();
        self.table.data.selection_anchor_row = None;
        self.table.data.selection_anchor_cell = None;
        self.feedback.copy_status.clear();
    }

    fn apply_grid_keyboard_commands(
        &mut self,
        ui: &mut egui::Ui,
        context: GridKeyboardApplicationContext<'_>,
        commands: result_grid_keyboard_view::GridKeyboardCommands,
    ) {
        if commands.copy_selected_rows {
            self.copy_selected_rows(ui, context.result);
        } else if commands.copy_selected_cell {
            self.copy_selected_cell(ui, context.result);
        }
        if commands.apply_staged_changes {
            self.apply_staged_changes();
        }
        if commands.discard_staged_changes {
            if self.table.mutation.staged_changes.counts().total() > 1 {
                self.table.editing.discard_changes_confirmation = true;
            } else {
                self.discard_staged_changes();
            }
        }
        if commands.delete_selected_rows {
            self.request_delete_selected_data_rows(context.result);
        }
        if context.editable {
            self.handle_grid_edit_input(ui, context.result, commands.pasted_text);
        }
        if commands.commit_edit_and_navigate {
            if !self.commit_active_data_edit(context.result) {
                return;
            }
            self.navigate_grid(
                ui,
                context.indexes,
                context.order,
                context.editable,
                context.selection_lookup,
            );
            return;
        }
        if commands.navigate {
            self.navigate_grid(
                ui,
                context.indexes,
                context.order,
                context.editable,
                context.selection_lookup,
            );
        }
    }

    fn navigate_grid(
        &mut self,
        ui: &mut egui::Ui,
        indexes: &[usize],
        order: &[usize],
        editable: bool,
        selection_lookup: &GridSelectionLookup,
    ) {
        let mut context = result_grid_selection::GridNavigationContext {
            data: &mut self.table.data,
            editing: &mut self.table.editing,
            feedback: &mut self.feedback,
        };
        result_grid_selection::handle_grid_navigation(ui, indexes, order, editable, selection_lookup, &mut context);
    }

    /// Paste-into-cell and Enter/F2-to-edit while the grid is editable.
    pub(crate) fn handle_grid_edit_input(&mut self, ui: &mut egui::Ui, result: &UiQueryResult, pasted: Option<String>) {
        if let (Some((row_index, column_index)), Some(text)) = (self.table.data.selected_cell, pasted) {
            if let Some(block) = self.blocked_write_for_cell(result, column_index) {
                self.feedback.copy_status = block.reason().to_owned();
                return;
            }
            self.table.editing.data_editing_cell = Some((row_index, column_index));
            self.table.editing.data_edit_value = text;
            self.submit_data_cell_edit(result, row_index, column_index);
        }
        if self.table.editing.data_editing_cell.is_none()
            && self.table.data.selected_cell.is_some()
            && ui.input(|input| input.key_pressed(egui::Key::Enter) || input.key_pressed(egui::Key::F2))
        {
            if let Some((row_index, column_index)) = self.table.data.selected_cell {
                if let Some(cell) = result.rows.get(row_index).and_then(|row| row.get(column_index)) {
                    self.begin_data_cell_edit(result, row_index, column_index, cell);
                }
            }
        }
    }

    /// The write policy for the column behind a grid cell, if it is blocked.
    pub(crate) fn blocked_write_for_cell(
        &self,
        result: &UiQueryResult,
        column_index: usize,
    ) -> Option<ColumnWriteBlock> {
        let column = result.columns.get(column_index)?;
        self.table.state.column_write_block(&column.name)
    }

    /// Scrollable grid: continuous spreadsheet header plus visible slice of rows.
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn draw_grid_body(
        &mut self,
        ui: &mut egui::Ui,
        result: &UiQueryResult,
        indexes: &[usize],
        order: &[usize],
        editable: bool,
        row_offset: u64,
        selection_lookup: &GridSelectionLookup,
    ) {
        let grid_height = ui.available_height().max(180.0);
        let grid_width = ui.available_width().max(0.0);
        let widths = self.table.data.column_widths(result.columns.len(), grid_width);

        ui.allocate_ui_with_layout(
            egui::vec2(grid_width, grid_height),
            Layout::top_down(Align::Min),
            |ui| {
                ui.spacing_mut().item_spacing = Vec2::ZERO;
                egui::ScrollArea::horizontal().show(ui, |ui| {
                    ui.spacing_mut().item_spacing = Vec2::ZERO;
                    let content_width = GRID_ROW_NUMBER_WIDTH + widths.iter().sum::<f32>();
                    ui.set_min_width(content_width);
                    self.draw_grid_header(ui, result, indexes, &widths, order);
                    let rows = GridRows {
                        indexes,
                        widths: &widths,
                        order,
                        editable,
                        row_offset,
                        selection_lookup,
                    };
                    let row_height = 28.0;
                    egui::ScrollArea::vertical()
                        .max_height((grid_height - 34.0).max(140.0))
                        .show_rows(ui, row_height, indexes.len(), |ui, range| {
                            ui.spacing_mut().item_spacing = Vec2::ZERO;
                            for position in range {
                                self.draw_grid_row(ui, result, &rows, position);
                            }
                        });
                });
            },
        );
    }

    /// One grid row: the row-number gutter plus every visible cell with continuous borders.
    pub(crate) fn draw_grid_row(
        &mut self,
        ui: &mut egui::Ui,
        result: &UiQueryResult,
        rows: &GridRows<'_>,
        position: usize,
    ) {
        let row_index = rows.indexes[position];
        let row = &result.rows[row_index];
        let row_selected = self.table.data.selected_rows.contains(&row_index);
        let row_dirty = self.staged_row_deleted(result, row_index)
            || (0..result.columns.len())
                .any(|column_index| self.staged_cell_value(result, row_index, column_index).is_some());
        let row_identity =
            self.table
                .data
                .row_identity_for_result(result, self.table.state.table_info.as_ref(), row_index);
        let row_mutation_error = row_identity
            .as_ref()
            .is_some_and(|identity| self.table.mutation.mutation_error_for_identity(identity));
        let cell_mutation_errors = (0..result.columns.len())
            .map(|column_index| {
                row_identity
                    .as_ref()
                    .is_some_and(|identity| self.table.mutation.mutation_error_for_cell(identity, column_index))
            })
            .collect::<Vec<_>>();
        let theme = self.theme;
        let mut renderer = GridRowRenderer { app: self, result };
        result_grid_row_view::draw_row(
            ui,
            result_grid_row_view::GridRowSurfaceContext {
                row,
                rows,
                row_index,
                display_position: position,
                row_selected,
                row_dirty,
                row_mutation_error,
                cell_mutation_errors: &cell_mutation_errors,
                theme,
            },
            &mut renderer,
        );
    }
}

struct GridRowRenderer<'a> {
    app: &'a mut DbProApp,
    result: &'a UiQueryResult,
}

impl result_grid_row_view::GridRowSurfaceRenderer for GridRowRenderer<'_> {
    fn on_gutter_click(&mut self, ui: &mut egui::Ui, rows: &GridRows<'_>, position: usize) -> bool {
        if self.app.table.editing.data_editing_cell.is_some() && !self.app.commit_active_data_edit(self.result) {
            return false;
        }
        self.app.table.data.selected_cell = None;
        let modifiers = ui.input(|input| input.modifiers);
        self.app.table.data.select_visible_row(
            rows.indexes,
            &rows.selection_lookup.row_positions,
            position,
            modifiers.shift,
            modifiers.command || modifiers.ctrl,
        );
        self.app.table.data.selection_anchor_cell = None;
        self.app.table.editing.data_editing_cell = None;
        self.app.table.editing.data_edit_value.clear();
        self.app.feedback.copy_status.clear();
        true
    }

    fn draw_cell(&mut self, ui: &mut egui::Ui, cell: GridCell<'_>) {
        self.app.draw_grid_cell(ui, self.result, cell);
    }
}

#[cfg(test)]
#[path = "result_grid_view_tests.rs"]
mod tests;
