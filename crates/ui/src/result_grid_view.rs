use super::*;
use crate::GridProjectionKey;
use egui::{Align2, Pos2, Rounding, Stroke, Vec2};
use std::collections::HashMap;

/// Coordinate lookup for the current visible grid slice.
///
/// Selection is rendered once per visible cell, so resolving coordinates must
/// not scan the filtered row list and reordered column list for every cell.
pub struct GridSelectionLookup {
    pub row_positions: HashMap<usize, usize>,
    pub column_positions: HashMap<usize, usize>,
}

impl GridSelectionLookup {
    /// Build a lookup from the filtered row projection and the visual column order.
    ///
    /// This is the measured per-frame cost: two `HashMap`s, one entry per filtered row index
    /// and one per column. `GridSelectionCache` memoizes the result so a frame that changes neither
    /// the projection nor the column order reuses it instead of rebuilding.
    pub fn new(indexes: &[usize], order: &[usize]) -> Self {
        Self {
            row_positions: indexes
                .iter()
                .enumerate()
                .map(|(position, &row)| (row, position))
                .collect(),
            column_positions: order
                .iter()
                .enumerate()
                .map(|(position, &column)| (column, position))
                .collect(),
        }
    }
}

/// One-entry memo for [`GridSelectionLookup`].
///
/// `draw_grid_body` built a `GridSelectionLookup` on every frame before this cache — two `HashMap`s,
/// one entry per filtered row index and one per column — even though the lookup only changes when
/// the projection or the column order changes. With the projection cached, this per-frame
/// map build dominated the steady-state frame cost of a large result: ~36.5 ms of a
/// ~40 ms frame at 200k rows × 4 columns in debug.
///
/// The cache is keyed on `(GridProjectionKey, column_order)` and follows the same take/restore
/// idiom as [`GridProjectionCache`]: the lookup is moved out for the frame and handed back, so a
/// sorted large result reuses it until one of its inputs really changes.
#[derive(Default)]
pub struct GridSelectionCache {
    key: Option<(GridProjectionKey, Vec<usize>)>,
    lookup: Option<GridSelectionLookup>,
    rebuilds: u64,
}

impl GridSelectionCache {
    /// Take the selection lookup for `(projection_key, order)`, reusing the cached entry when the
    /// key matches. Returns `None` on a miss — the caller must rebuild and [`Self::restore`] it.
    pub fn take(&mut self, projection_key: &GridProjectionKey, order: &[usize]) -> Option<GridSelectionLookup> {
        let hit = self
            .key
            .as_ref()
            .map(|(key, cached_order)| key == projection_key && cached_order.as_slice() == order)
            .unwrap_or(false);
        if hit {
            self.key = None;
            return self.lookup.take();
        }
        self.key = None;
        self.lookup = None;
        self.rebuilds = self.rebuilds.saturating_add(1);
        None
    }

    /// Store a selection lookup the caller got from [`Self::take`] so the next frame can reuse it.
    pub fn restore(&mut self, projection_key: GridProjectionKey, order: Vec<usize>, lookup: GridSelectionLookup) {
        if self.key.is_some() {
            return;
        }
        self.key = Some((projection_key, order));
        self.lookup = Some(lookup);
    }

    /// Number of selection lookups built by this cache. Asserted by tests to pin the
    /// "no per-frame rebuild" property.
    #[cfg(test)]
    fn rebuilds(&self) -> u64 {
        self.rebuilds
    }
}

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
                        self.copy_all_as_csv(ui, result, &indexes);
                    }
                    result_grid_toolbar_view::ResultGridToolbarAction::CopyVisibleJson => {
                        self.copy_all_as_json(ui, result, &indexes);
                    }
                    result_grid_toolbar_view::ResultGridToolbarAction::InspectSelectedCell {
                        row_index,
                        column_index,
                    } => self.open_cell_inspector(result, row_index, column_index),
                }
            }
        }
        self.draw_record_inspector_panel(ui, result);

        let row_offset = if is_table_data { self.table.data_query.offset } else { 0 };
        self.draw_grid_body(ui, result, &indexes, &order, editable, row_offset, &selection_lookup);

        self.restore_grid_cache(projection_key, indexes, order, selection_lookup);
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
        let modifier = Self::primary_modifier_pressed_ui(ui);
        let shift = ui.input(|i| i.modifiers.shift);

        if !ui.ctx().wants_keyboard_input()
            && ui.input(|input| input.key_pressed(egui::Key::A) && Self::primary_modifier_pressed(input))
        {
            self.table.data.select_all_visible_cells(indexes, order);
            self.feedback.copy_status.clear();
            return;
        }

        if !ui.ctx().wants_keyboard_input() && ui.input(|input| input.key_pressed(egui::Key::Escape)) {
            self.table.data.selected_cell = None;
            self.table.data.selected_row = None;
            self.table.data.selected_rows.clear();
            self.table.data.selection_anchor_row = None;
            self.table.data.selection_anchor_cell = None;
            self.feedback.copy_status.clear();
            return;
        }

        if !ui.ctx().wants_keyboard_input() && !self.connection.dialog.is_open() {
            if ui.input(|i| i.key_pressed(egui::Key::C)) && modifier && shift {
                self.copy_selected_rows(ui, result);
            } else if ui.input(|i| i.key_pressed(egui::Key::C)) && modifier {
                self.copy_selected_cell(ui, result);
            }

            if ui.input(|input| input.key_pressed(egui::Key::S) && Self::primary_modifier_pressed(input)) {
                self.apply_staged_changes();
            }
            if ui.input(|input| input.key_pressed(egui::Key::Z) && Self::primary_modifier_pressed(input)) {
                if self.table.mutation.staged_changes.counts().total() > 1 {
                    self.table.editing.discard_changes_confirmation = true;
                } else {
                    self.discard_staged_changes();
                }
            }
            if editable
                && self.table.editing.data_editing_cell.is_none()
                && ui.input(|input| input.key_pressed(egui::Key::Delete) || input.key_pressed(egui::Key::Backspace))
            {
                self.request_delete_selected_data_rows(result);
            }
        }
        let pasted = ui.input(|input| {
            input.events.iter().find_map(|event| match event {
                egui::Event::Paste(text) => Some(text.clone()),
                _ => None,
            })
        });
        if editable {
            self.handle_grid_edit_input(ui, result, pasted);
        }
        if self.table.editing.data_editing_cell.is_some() && ui.input(|input| input.key_pressed(egui::Key::Tab)) {
            if !self.commit_active_data_edit(result) {
                return;
            }
            let mut context = result_grid_selection::GridNavigationContext {
                data: &mut self.table.data,
                editing: &mut self.table.editing,
                feedback: &mut self.feedback,
            };
            result_grid_selection::handle_grid_navigation(ui, indexes, order, editable, selection_lookup, &mut context);
            return;
        }
        if !ui.ctx().wants_keyboard_input() {
            let mut context = result_grid_selection::GridNavigationContext {
                data: &mut self.table.data,
                editing: &mut self.table.editing,
                feedback: &mut self.feedback,
            };
            result_grid_selection::handle_grid_navigation(ui, indexes, order, editable, selection_lookup, &mut context);
        }
    }

    pub(crate) fn primary_modifier_pressed_ui(ui: &egui::Ui) -> bool {
        ui.input(Self::primary_modifier_pressed)
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
        let row_mutation_error = self
            .table
            .data
            .row_identity_for_result(result, self.table.state.table_info.as_ref(), row_index)
            .is_some_and(|identity| self.table.mutation.mutation_error_for_identity(&identity));

        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing = Vec2::ZERO;
            let row_number = crate::displayed_row_number(rows.row_offset, row_index);
            let (gutter_rect, gutter_resp) =
                ui.allocate_exact_size(egui::vec2(GRID_ROW_NUMBER_WIDTH, 28.0), Sense::click());

            let gutter_fill = if row_selected {
                self.theme.accent.linear_multiply(0.18)
            } else if gutter_resp.hovered() {
                self.theme.surface_hover.linear_multiply(0.5)
            } else {
                self.theme.surface_panel.linear_multiply(0.5)
            };
            ui.painter().rect_filled(gutter_rect, Rounding::ZERO, gutter_fill);
            ui.painter().hline(
                gutter_rect.x_range(),
                gutter_rect.bottom(),
                Stroke::new(1.0, self.theme.border_subtle.linear_multiply(0.4)),
            );
            ui.painter().vline(
                gutter_rect.right(),
                gutter_rect.y_range(),
                Stroke::new(1.0, self.theme.border_subtle.linear_multiply(0.4)),
            );
            ui.painter().text(
                Pos2::new(gutter_rect.right() - 8.0, gutter_rect.center().y),
                Align2::RIGHT_CENTER,
                row_number.to_string(),
                FontId::monospace(11.0),
                if row_selected {
                    self.theme.accent
                } else {
                    self.theme.text_muted
                },
            );

            if gutter_resp.clicked() {
                if self.table.editing.data_editing_cell.is_some() && !self.commit_active_data_edit(result) {
                    return;
                }
                self.table.data.selected_cell = None;
                let modifiers = ui.input(|input| input.modifiers);
                self.table.data.select_visible_row(
                    rows.indexes,
                    &rows.selection_lookup.row_positions,
                    position,
                    modifiers.shift,
                    modifiers.command || modifiers.ctrl,
                );
                self.table.data.selection_anchor_cell = None;
                self.table.editing.data_editing_cell = None;
                self.table.editing.data_edit_value.clear();
                self.feedback.copy_status.clear();
            }

            for &column_index in rows.order {
                let cell = row.get(column_index).unwrap_or(&UiCell::Null);
                let width = rows.widths.get(column_index).copied().unwrap_or(180.0);
                self.draw_grid_cell(
                    ui,
                    result,
                    GridCell {
                        visible_indexes: rows.indexes,
                        selection_lookup: rows.selection_lookup,
                        row_index,
                        column_index,
                        display_position: position,
                        row_selected,
                        row_dirty,
                        row_mutation_error,
                        cell_mutation_error: self
                            .table
                            .data
                            .row_identity_for_result(result, self.table.state.table_info.as_ref(), row_index)
                            .is_some_and(|identity| {
                                self.table.mutation.mutation_error_for_cell(&identity, column_index)
                            }),
                        editable: rows.editable,
                        width,
                        cell,
                    },
                );
            }
        });
    }

    pub(crate) fn cell_label(cell: &crate::UiCell) -> String {
        match cell {
            crate::UiCell::Null => "NULL".to_owned(),
            crate::UiCell::Boolean(value) => value.to_string(),
            crate::UiCell::Number(value) => value.clone(),
            crate::UiCell::Text(value) => value.clone(),
            crate::UiCell::Json(value) => serde_json::from_str::<serde_json::Value>(value)
                .ok()
                .and_then(|json| serde_json::to_string_pretty(&json).ok())
                .unwrap_or_else(|| value.clone()),
            crate::UiCell::Bytes(value) => value.clone(),
        }
    }
}

#[cfg(test)]
#[path = "result_grid_view_tests.rs"]
mod tests;
