//! Result-grid cell painting and context menus.
use super::result_grid_view::{GridCell, GridSelectionLookup};
use super::*;
use egui::{Align2, Pos2, Rounding, Stroke};

#[derive(Default)]
struct GridCellMenuRequests {
    copy_cell: bool,
    copy_row: bool,
    copy_selected_rows: bool,
    copy_selected_rows_headers: bool,
    copy_selected_rows_json: bool,
    copy_selected_rows_insert: bool,
    copy_json: bool,
    copy_csv: bool,
    edit_cell: bool,
    set_null: bool,
    revert_cell: bool,
    revert_row: bool,
    duplicate_row: bool,
    delete_row: bool,
    filter_this_val: bool,
    sort_asc: bool,
    sort_desc: bool,
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
        let cell_selected = self.is_cell_selected(selection_lookup, (row_index, column_index));
        let editing = editable && self.data_editing_cell == Some((row_index, column_index));
        let validation_error = editing && self.data_edit_error.is_some();
        let conflict_error = (cell_mutation_error || row_mutation_error)
            && self
                .table_mutation_error
                .as_ref()
                .is_some_and(|failure| failure.code == "CONFLICT");

        let (cell_rect, cell_resp) = ui.allocate_exact_size(egui::vec2(width, 28.0), Sense::click());

        let fill = if validation_error || (cell_mutation_error && !conflict_error) {
            self.theme.danger.linear_multiply(0.14)
        } else if conflict_error {
            self.theme.warning.linear_multiply(0.16)
        } else if row_mutation_error {
            self.theme.danger.linear_multiply(0.08)
        } else if row_selected {
            if cell_selected {
                self.theme.accent.linear_multiply(0.20)
            } else {
                self.theme.accent.linear_multiply(0.08)
            }
        } else if cell_selected {
            self.theme.accent.linear_multiply(0.14)
        } else if row_dirty {
            self.theme.warning.linear_multiply(0.12)
        } else if cell_resp.hovered() {
            self.theme.surface_hover.linear_multiply(0.35)
        } else if display_position % 2 == 1 {
            self.theme.surface_panel.linear_multiply(0.25)
        } else {
            self.theme.surface_elevated
        };

        ui.painter().rect_filled(cell_rect, Rounding::ZERO, fill);

        // Bottom and right border lines for continuous spreadsheet appearance
        let border_stroke = Stroke::new(1.0, self.theme.border_subtle.linear_multiply(0.35));
        ui.painter()
            .hline(cell_rect.x_range(), cell_rect.bottom(), border_stroke);
        ui.painter()
            .vline(cell_rect.right(), cell_rect.y_range(), border_stroke);

        if cell_selected {
            ui.painter()
                .rect_stroke(cell_rect, Rounding::ZERO, Stroke::new(1.5, self.theme.accent));
        }
        if validation_error {
            ui.painter().rect_stroke(
                cell_rect.shrink(1.0),
                Rounding::ZERO,
                Stroke::new(1.5, self.theme.danger),
            );
            if let Some(error) = self.data_edit_error.as_deref() {
                cell_resp.clone().on_hover_text(error);
            }
        }
        if cell_mutation_error || row_mutation_error {
            ui.painter().rect_stroke(
                cell_rect.shrink(1.0),
                Rounding::ZERO,
                Stroke::new(
                    1.5,
                    if conflict_error {
                        self.theme.warning
                    } else {
                        self.theme.danger
                    },
                ),
            );
            if let Some(error) = self.table_mutation_error.as_ref() {
                cell_resp.clone().on_hover_text(error.message.as_str());
            }
        }

        if editing {
            self.draw_grid_cell_editor(ui, result, row_index, column_index, cell_rect);
        } else {
            let text_rect = cell_rect.shrink2(egui::vec2(8.0, 3.0));
            let is_null = matches!(display_cell, crate::UiCell::Null);
            let text_val = Self::cell_label(display_cell);
            let font = match display_cell {
                crate::UiCell::Null => FontId::proportional(11.5),
                crate::UiCell::Number(_) | crate::UiCell::Json(_) | crate::UiCell::Bytes(_) => FontId::monospace(11.5),
                _ => FontId::proportional(12.0),
            };
            let text_color = match display_cell {
                crate::UiCell::Null => self.theme.text_muted.linear_multiply(0.55),
                crate::UiCell::Boolean(val) => {
                    if *val {
                        self.theme.success
                    } else {
                        self.theme.danger
                    }
                }
                crate::UiCell::Number(_) => self.theme.text_primary,
                crate::UiCell::Json(_) | crate::UiCell::Bytes(_) => self.theme.text_secondary,
                crate::UiCell::Text(_) => self.theme.text_primary,
            };

            let painter = ui.painter().with_clip_rect(text_rect);
            if is_null {
                // Subtle italic badge for NULL
                let galley = painter.layout_no_wrap(
                    "NULL".to_owned(),
                    FontId::new(11.0, egui::FontFamily::Proportional),
                    text_color,
                );
                painter.galley(
                    Pos2::new(text_rect.left(), text_rect.center().y - galley.size().y * 0.5),
                    galley,
                    text_color,
                );
            } else {
                painter.text(
                    Pos2::new(text_rect.left(), text_rect.center().y),
                    Align2::LEFT_CENTER,
                    text_val,
                    font,
                    text_color,
                );
            }
            let raw_value = crate::cell_text(display_cell);
            if raw_value.chars().count() > 40 {
                cell_resp.clone().on_hover_text(raw_value);
            }

            let is_ctx = self.run_grid_cell_context_menu(
                ui,
                &cell_resp,
                result,
                selection_lookup,
                row_index,
                column_index,
                editable,
                display_cell,
            );

            if cell_resp.double_clicked() && editable {
                if self.data_editing_cell.is_some() && !self.commit_active_data_edit(result) {
                    return;
                }
                self.begin_data_cell_edit(result, row_index, column_index, display_cell);
            } else if cell_resp.clicked() && !is_ctx {
                if self.data_editing_cell.is_some() && !self.commit_active_data_edit(result) {
                    return;
                }
                let modifiers = ui.input(|input| input.modifiers);
                self.select_cell_range(
                    visible_indexes,
                    &selection_lookup.row_positions,
                    (row_index, column_index),
                    modifiers.shift,
                );
                self.copy_status.clear();
            }
        }
    }

    // allow: grid cell context menu requires full context (result, selection, row/col, editable)
    // to paint and dispatch; kept flat for direct readability at render call site.
    #[allow(clippy::too_many_arguments)]
    fn run_grid_cell_context_menu(
        &mut self,
        ui: &mut egui::Ui,
        cell_resp: &egui::Response,
        result: &UiQueryResult,
        selection_lookup: &GridSelectionLookup,
        row_index: usize,
        column_index: usize,
        editable: bool,
        display_cell: &UiCell,
    ) -> bool {
        let is_ctx = is_context_menu_triggered(cell_resp, ui);
        let mut req = GridCellMenuRequests::default();
        let theme = self.theme;
        let modifier = Self::primary_modifier_label();

        context_action_menu(ui, cell_resp, theme, |ui, close_menu| {
            let copy_sc = format!("{modifier}C");
            if ctx_menu_item(
                ui,
                Some(Icon::Copy),
                "Copy Cell Value",
                Some(&copy_sc),
                theme.text_primary,
                theme,
            )
            .clicked()
            {
                req.copy_cell = true;
                *close_menu = true;
            }
            if ctx_menu_item(
                ui,
                Some(Icon::Table2),
                "Copy Row (TSV)",
                Some(&format!("{modifier}Shift+C")),
                theme.text_primary,
                theme,
            )
            .clicked()
            {
                req.copy_row = true;
                *close_menu = true;
            }
            if ctx_menu_item(
                ui,
                Some(Icon::Rows3),
                "Copy Selected Rows",
                None,
                theme.text_primary,
                theme,
            )
            .clicked()
            {
                req.copy_selected_rows = true;
                *close_menu = true;
            }
            if ctx_menu_item(
                ui,
                Some(Icon::Table2),
                "Copy Selected Rows with Headers",
                None,
                theme.text_primary,
                theme,
            )
            .clicked()
            {
                req.copy_selected_rows_headers = true;
                *close_menu = true;
            }
            if ctx_menu_item(
                ui,
                Some(Icon::Braces),
                "Copy Selected Rows as JSON",
                None,
                theme.text_primary,
                theme,
            )
            .clicked()
            {
                req.copy_selected_rows_json = true;
                *close_menu = true;
            }
            if ctx_menu_item(
                ui,
                Some(Icon::Code),
                "Copy Selected Rows as INSERT SQL",
                None,
                theme.text_primary,
                theme,
            )
            .clicked()
            {
                req.copy_selected_rows_insert = true;
                *close_menu = true;
            }
            if ctx_menu_item(
                ui,
                Some(Icon::Braces),
                "Copy Row as JSON",
                None,
                theme.text_primary,
                theme,
            )
            .clicked()
            {
                req.copy_json = true;
                *close_menu = true;
            }
            if ctx_menu_item(
                ui,
                Some(Icon::FileSpreadsheet),
                "Copy Row as CSV",
                None,
                theme.text_primary,
                theme,
            )
            .clicked()
            {
                req.copy_csv = true;
                *close_menu = true;
            }

            if editable {
                ui.separator();
                let write_block = self.blocked_write_for_cell(result, column_index);
                let edit_clicked = match write_block {
                    Some(block) => {
                        ctx_menu_item(ui, Some(Icon::Lock), "Read-only Column", None, theme.text_muted, theme)
                            .on_hover_text(block.reason());
                        false
                    }
                    None => ctx_menu_item(
                        ui,
                        Some(Icon::Pencil),
                        "Edit Cell",
                        Some("Enter / F2"),
                        theme.text_primary,
                        theme,
                    )
                    .clicked(),
                };
                if edit_clicked {
                    req.edit_cell = true;
                    *close_menu = true;
                }
                if write_block.is_none()
                    && ctx_menu_item(ui, Some(Icon::Eraser), "Set to NULL", None, theme.text_secondary, theme).clicked()
                {
                    req.set_null = true;
                    *close_menu = true;
                }
                if self.staged_cell_value(result, row_index, column_index).is_some()
                    && ctx_menu_item(ui, Some(Icon::Undo2), "Revert Cell", None, theme.text_primary, theme).clicked()
                {
                    req.revert_cell = true;
                    *close_menu = true;
                }
                let has_row_change = self
                    .row_identity_for_result(result, row_index)
                    .as_ref()
                    .map(|identity| self.staged_changes.row_has_changes(identity))
                    .unwrap_or(false);
                if has_row_change
                    && ctx_menu_item(
                        ui,
                        Some(Icon::Undo2),
                        if self.staged_row_deleted(result, row_index) {
                            "Undo Delete"
                        } else {
                            "Revert Row"
                        },
                        None,
                        theme.text_primary,
                        theme,
                    )
                    .clicked()
                {
                    req.revert_row = true;
                    *close_menu = true;
                }
                if ctx_menu_item(
                    ui,
                    Some(Icon::CopyPlus),
                    "Duplicate Row",
                    None,
                    theme.text_primary,
                    theme,
                )
                .clicked()
                {
                    req.duplicate_row = true;
                    *close_menu = true;
                }
                if ctx_menu_item(
                    ui,
                    Some(Icon::Trash2),
                    "Delete Row",
                    Some("Delete / Backspace"),
                    theme.danger,
                    theme,
                )
                .clicked()
                {
                    req.delete_row = true;
                    *close_menu = true;
                }
            }

            ui.separator();
            if ctx_menu_item(
                ui,
                Some(Icon::Filter),
                "Filter by this value",
                None,
                theme.text_primary,
                theme,
            )
            .clicked()
            {
                req.filter_this_val = true;
                *close_menu = true;
            }
            if ctx_menu_item(
                ui,
                Some(Icon::ArrowUp),
                "Sort Ascending",
                None,
                theme.text_primary,
                theme,
            )
            .clicked()
            {
                req.sort_asc = true;
                *close_menu = true;
            }
            if ctx_menu_item(
                ui,
                Some(Icon::ArrowDown),
                "Sort Descending",
                None,
                theme.text_primary,
                theme,
            )
            .clicked()
            {
                req.sort_desc = true;
                *close_menu = true;
            }
        });

        if is_ctx
            && self
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
            let is_inside_range = self.is_cell_selected(selection_lookup, (row_index, column_index));
            if !is_inside_range {
                self.select_single_cell((row_index, column_index));
            }
            if !self.selected_rows.contains(&row_index) {
                self.select_single_row(row_index);
            } else if !is_inside_range {
                self.selected_row = Some(row_index);
                self.selection_anchor_row = Some(row_index);
            }
        }
        self.apply_grid_cell_menu_requests(
            ui,
            result,
            selection_lookup,
            row_index,
            column_index,
            editable,
            display_cell,
            req,
            is_ctx,
        );
        is_ctx
    }

    // allow: applier handles menu selections with the exact same context as menu rendering —
    // flat parameters enable 1:1 comparison with run_grid_cell_context_menu.
    #[allow(clippy::too_many_arguments)]
    fn apply_grid_cell_menu_requests(
        &mut self,
        ui: &mut egui::Ui,
        result: &UiQueryResult,
        _selection_lookup: &GridSelectionLookup,
        row_index: usize,
        column_index: usize,
        editable: bool,
        display_cell: &UiCell,
        req: GridCellMenuRequests,
        _is_ctx: bool,
    ) {
        if req.copy_cell {
            self.copy_cell_at(ui, result, row_index, column_index);
        }
        if req.copy_row {
            self.select_single_row(row_index);
            self.copy_selected_row(ui, result);
        }
        if req.copy_selected_rows {
            if !self.selected_rows.contains(&row_index) {
                self.select_single_row(row_index);
            }
            self.copy_selected_rows(ui, result);
        }
        if req.copy_selected_rows_headers {
            if !self.selected_rows.contains(&row_index) {
                self.select_single_row(row_index);
            }
            self.copy_selected_rows_with_headers(ui, result);
        }
        if req.copy_selected_rows_json {
            if !self.selected_rows.contains(&row_index) {
                self.select_single_row(row_index);
            }
            self.copy_selected_rows_as_json(ui, result);
        }
        if req.copy_selected_rows_insert {
            if !self.selected_rows.contains(&row_index) {
                self.select_single_row(row_index);
            }
            self.copy_selected_rows_as_insert(ui, result);
        }
        if req.copy_json {
            self.selected_row = Some(row_index);
            self.copy_row_as_json(ui, result, row_index);
        }
        if req.copy_csv {
            self.selected_row = Some(row_index);
            self.copy_row_as_csv(ui, result, row_index);
        }
        if req.edit_cell && editable {
            self.begin_data_cell_edit(result, row_index, column_index, display_cell);
        }
        if req.set_null && editable {
            self.data_editing_cell = Some((row_index, column_index));
            self.data_edit_value = "NULL".to_owned();
            self.data_edit_error = None;
            self.submit_data_cell_edit(result, row_index, column_index);
        }
        if req.revert_cell && editable {
            self.revert_staged_cell(result, row_index, column_index);
        }
        if req.revert_row && editable {
            self.revert_staged_row(result, row_index);
        }
        if req.duplicate_row && editable {
            self.open_duplicate_row(result, row_index);
        }
        if req.delete_row && editable {
            if !self.selected_rows.contains(&row_index) {
                self.select_single_row(row_index);
            }
            self.request_delete_selected_data_rows(result);
        }
        if req.filter_this_val {
            if self.active_tab == WorkspaceTab::Table && self.table_view == TableView::Data {
                self.table_data_filter_column = result
                    .columns
                    .get(column_index)
                    .map(|column| column.name.clone())
                    .unwrap_or_default();
                if matches!(display_cell, UiCell::Null) {
                    self.table_data_filter_operator = UiTableFilterOperator::IsNull;
                    self.table_data_filter_value.clear();
                } else {
                    self.table_data_filter_operator = UiTableFilterOperator::Equals;
                    self.table_data_filter_value = crate::cell_text(display_cell);
                }
                self.commit_table_filter_draft();
            } else {
                self.grid_filter = crate::cell_text(display_cell);
            }
        }
        if req.sort_asc {
            self.set_table_or_grid_sort(result, column_index, Some(false));
        }
        if req.sort_desc {
            self.set_table_or_grid_sort(result, column_index, Some(true));
        }
    }
}
