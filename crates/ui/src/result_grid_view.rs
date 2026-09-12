use super::*;
use egui::{Align2, Pos2, Rect, Rounding, Stroke, Vec2};

/// Per-cell render context for the result grid.
struct GridCell<'a> {
    row_index: usize,
    column_index: usize,
    display_position: usize,
    row_selected: bool,
    row_dirty: bool,
    editable: bool,
    width: f32,
    cell: &'a UiCell,
}

/// Context shared by every visible grid row.
struct GridRows<'a> {
    indexes: &'a [usize],
    widths: &'a [f32],
    editable: bool,
    row_offset: u64,
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

        let editable = self.active_tab == WorkspaceTab::Table
            && self.table_view == TableView::Data
            && self.can_mutate_active_connection();
        let indexes =
            crate::filtered_sorted_indexes(result, &self.grid_filter, self.grid_sort_column, self.grid_sort_desc);

        self.handle_grid_keyboard(ui, result, &indexes, editable);

        let is_table_data = self.active_tab == WorkspaceTab::Table && self.table_view == TableView::Data;
        if !is_table_data {
            self.draw_grid_toolbar(ui, result, editable, indexes.len());
        }

        let row_offset = if is_table_data { self.table_data_offset } else { 0 };
        self.draw_grid_body(ui, result, &indexes, editable, row_offset);
    }

    /// Copy, paste-to-edit, staged-changes, and keyboard navigation for the result grid.
    fn handle_grid_keyboard(&mut self, ui: &mut egui::Ui, result: &UiQueryResult, indexes: &[usize], editable: bool) {
        if ui.input(|input| input.key_pressed(egui::Key::C) && Self::primary_modifier_pressed(input)) {
            self.copy_selected_cell(ui, result);
        }
        if ui.input(|input| input.key_pressed(egui::Key::S) && Self::primary_modifier_pressed(input)) {
            self.apply_staged_changes();
        }
        if ui.input(|input| input.key_pressed(egui::Key::Z) && Self::primary_modifier_pressed(input)) {
            self.discard_staged_changes();
        }
        if editable
            && self.data_editing_cell.is_none()
            && ui.input(|input| input.key_pressed(egui::Key::Delete) || input.key_pressed(egui::Key::Backspace))
        {
            self.request_delete_selected_data_row(result);
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
        if !ui.ctx().wants_keyboard_input() {
            self.handle_grid_navigation(ui, indexes, result.columns.len(), editable, result);
        }
    }

    /// Paste-into-cell and Enter/F2-to-edit while the grid is editable.
    fn handle_grid_edit_input(&mut self, ui: &mut egui::Ui, result: &UiQueryResult, pasted: Option<String>) {
        if let (Some((row_index, column_index)), Some(text)) = (self.selected_cell, pasted) {
            self.data_editing_cell = Some((row_index, column_index));
            self.data_edit_value = text;
            self.submit_data_cell_edit(result, row_index, column_index);
        }
        if self.data_editing_cell.is_none()
            && self.selected_cell.is_some()
            && ui.input(|input| input.key_pressed(egui::Key::Enter) || input.key_pressed(egui::Key::F2))
        {
            if let Some((row_index, column_index)) = self.selected_cell {
                if let Some(cell) = result.rows.get(row_index).and_then(|row| row.get(column_index)) {
                    self.begin_data_cell_edit(row_index, column_index, cell);
                }
            }
        }
    }

    /// Arrow / Tab / Home / End navigation over the visible (filtered, sorted) indexes.
    fn handle_grid_navigation(
        &mut self,
        ui: &mut egui::Ui,
        indexes: &[usize],
        column_count: usize,
        editable: bool,
        result: &UiQueryResult,
    ) {
        let is_tab = ui.input(|input| input.key_pressed(egui::Key::Tab));
        let is_shift_tab = is_tab && ui.input(|input| input.modifiers.shift);

        if is_tab && column_count > 0 && !indexes.is_empty() {
            if let Some((curr_row, curr_col)) = self.selected_cell {
                if editable && self.data_editing_cell.is_some() {
                    self.commit_active_data_edit(result);
                }
                let next_cell = if is_shift_tab {
                    if curr_col > 0 {
                        Some((curr_row, curr_col - 1))
                    } else if let Some(pos) = indexes.iter().position(|&r| r == curr_row) {
                        if pos > 0 {
                            Some((indexes[pos - 1], column_count - 1))
                        } else {
                            Some((curr_row, curr_col))
                        }
                    } else {
                        Some((curr_row, curr_col))
                    }
                } else if curr_col + 1 < column_count {
                    Some((curr_row, curr_col + 1))
                } else if let Some(pos) = indexes.iter().position(|&r| r == curr_row) {
                    if pos + 1 < indexes.len() {
                        Some((indexes[pos + 1], 0))
                    } else {
                        Some((curr_row, curr_col))
                    }
                } else {
                    Some((curr_row, curr_col))
                };
                if let Some(selection) = next_cell {
                    self.selected_cell = Some(selection);
                    self.selected_row = Some(selection.0);
                    self.data_editing_cell = None;
                    self.data_edit_value.clear();
                    self.copy_status.clear();
                }
                return;
            }
        }

        let navigation_key = ui.input(|input| {
            [
                egui::Key::ArrowUp,
                egui::Key::ArrowDown,
                egui::Key::ArrowLeft,
                egui::Key::ArrowRight,
                egui::Key::Home,
                egui::Key::End,
            ]
            .into_iter()
            .find(|key| input.key_pressed(*key))
        });
        let Some(key) = navigation_key else {
            return;
        };
        if let Some(selection) = crate::grid_keyboard_selection(self.selected_cell, indexes, column_count, key) {
            self.selected_cell = Some(selection);
            self.selected_row = Some(selection.0);
            self.data_editing_cell = None;
            self.data_edit_value.clear();
            self.copy_status.clear();
        }
    }

    /// Filter box, copy buttons and the row-count hint above the grid.
    fn draw_grid_toolbar(&mut self, ui: &mut egui::Ui, result: &UiQueryResult, editable: bool, matching_rows: usize) {
        toolbar_frame(self.theme).show(ui, |ui| {
            ui.horizontal(|ui| {
                input(ui, &mut self.grid_filter, "Filter visible rows…", 220.0, self.theme);
                if !self.grid_filter.is_empty()
                    && compact_icon_button(ui, Icon::X, self.theme)
                        .on_hover_text("Clear filter")
                        .clicked()
                {
                    self.grid_filter.clear();
                }

                crate::components::badge::Badge::new(&format!("{matching_rows} rows"), self.theme)
                    .variant(crate::components::badge::BadgeVariant::Secondary)
                    .compact(true)
                    .show(ui);

                ui.separator();

                if compact_button_with_icon(ui, Icon::Copy, "Copy Cell", self.theme)
                    .on_hover_text("Copy selected cell value (Cmd/Ctrl+C)")
                    .clicked()
                {
                    self.copy_selected_cell(ui, result);
                }
                if compact_button_with_icon(ui, Icon::Table2, "Copy Row", self.theme)
                    .on_hover_text("Copy entire selected row as tab-separated text")
                    .clicked()
                {
                    self.copy_selected_row(ui, result);
                }

                if !self.copy_status.is_empty() {
                    crate::components::badge::Badge::new(&self.copy_status, self.theme)
                        .variant(crate::components::badge::BadgeVariant::Success)
                        .compact(true)
                        .show(ui);
                }

                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    ui.label(
                        RichText::new(if editable {
                            "Double-click / Enter to edit · Drag divider to resize"
                        } else {
                            "Click cell to select · Drag divider to resize"
                        })
                        .font(font_caption())
                        .color(self.theme.text_muted),
                    );
                });
            });
        });
        ui.add_space(4.0);
    }

    /// Scrollable grid: continuous spreadsheet header plus visible slice of rows.
    fn draw_grid_body(
        &mut self,
        ui: &mut egui::Ui,
        result: &UiQueryResult,
        indexes: &[usize],
        editable: bool,
        row_offset: u64,
    ) {
        let grid_height = ui.available_height().max(180.0);
        let grid_width = ui.available_width().max(0.0);
        let widths = self.column_widths(result.columns.len(), grid_width);

        ui.allocate_ui_with_layout(
            egui::vec2(grid_width, grid_height),
            Layout::top_down(Align::Min),
            |ui| {
                ui.spacing_mut().item_spacing = Vec2::ZERO;
                egui::ScrollArea::horizontal().show(ui, |ui| {
                    ui.spacing_mut().item_spacing = Vec2::ZERO;
                    let content_width = GRID_ROW_NUMBER_WIDTH + widths.iter().sum::<f32>();
                    ui.set_min_width(content_width);
                    self.draw_grid_header(ui, result, &widths);
                    let rows = GridRows {
                        indexes,
                        widths: &widths,
                        editable,
                        row_offset,
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
    fn draw_grid_row(&mut self, ui: &mut egui::Ui, result: &UiQueryResult, rows: &GridRows<'_>, position: usize) {
        let row_index = rows.indexes[position];
        let row = &result.rows[row_index];
        let row_selected = self.selected_row == Some(row_index);
        let row_dirty = self.staged_row_deleted(row_index)
            || (0..result.columns.len()).any(|column_index| self.staged_cell_value(row_index, column_index).is_some());

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
                self.commit_active_data_edit(result);
                self.selected_cell = None;
                self.selected_row = Some(row_index);
                self.data_editing_cell = None;
                self.data_edit_value.clear();
                self.copy_status.clear();
            }

            for (column_index, cell) in row.iter().enumerate().take(result.columns.len()) {
                let width = rows.widths.get(column_index).copied().unwrap_or(180.0);
                self.draw_grid_cell(
                    ui,
                    result,
                    GridCell {
                        row_index,
                        column_index,
                        display_position: position,
                        row_selected,
                        row_dirty,
                        editable: rows.editable,
                        width,
                        cell,
                    },
                );
            }
        });
    }

    /// One grid cell: crisp background, grid borders, active cell highlight, and formatted value.
    fn draw_grid_cell(&mut self, ui: &mut egui::Ui, result: &UiQueryResult, cell_ctx: GridCell<'_>) {
        let GridCell {
            row_index,
            column_index,
            display_position,
            row_selected,
            row_dirty,
            editable,
            width,
            cell,
        } = cell_ctx;

        let staged_cell = self.staged_cell_value(row_index, column_index);
        let display_cell = staged_cell.as_ref().unwrap_or(cell);
        let cell_selected = self.selected_cell == Some((row_index, column_index));

        let (cell_rect, cell_resp) = ui.allocate_exact_size(egui::vec2(width, 28.0), Sense::click());

        let fill = if row_selected {
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

        let editing = editable && self.data_editing_cell == Some((row_index, column_index));
        if editing {
            self.draw_grid_cell_editor(ui, result, row_index, column_index, cell_rect);
        } else {
            let text_rect = cell_rect.shrink2(egui::vec2(8.0, 3.0));
            let text_val = Self::cell_label(display_cell);
            let font = match display_cell {
                crate::UiCell::Number(_) | crate::UiCell::Json(_) | crate::UiCell::Bytes(_) => FontId::monospace(11.5),
                _ => FontId::proportional(12.0),
            };
            let text_color = match display_cell {
                crate::UiCell::Null => self.theme.text_muted.linear_multiply(0.7),
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
            painter.text(
                Pos2::new(text_rect.left(), text_rect.center().y),
                Align2::LEFT_CENTER,
                text_val,
                font,
                text_color,
            );

            let is_ctx = is_context_menu_triggered(&cell_resp, ui);
            let mut copy_cell_req = false;
            let mut copy_row_req = false;
            let mut edit_cell_req = false;
            let mut set_null_req = false;
            let mut delete_row_req = false;
            let theme = self.theme;
            let modifier = Self::primary_modifier_label();

            context_action_menu(ui, &cell_resp, theme, |ui, close_menu| {
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
                    copy_cell_req = true;
                    *close_menu = true;
                }
                if ctx_menu_item(
                    ui,
                    Some(Icon::Table2),
                    "Copy Row",
                    Some(&format!("{modifier}Shift+C")),
                    theme.text_primary,
                    theme,
                )
                .clicked()
                {
                    copy_row_req = true;
                    *close_menu = true;
                }
                if editable {
                    ui.separator();
                    if ctx_menu_item(
                        ui,
                        Some(Icon::Pencil),
                        "Edit Cell",
                        Some("Enter / F2"),
                        theme.text_primary,
                        theme,
                    )
                    .clicked()
                    {
                        edit_cell_req = true;
                        *close_menu = true;
                    }
                    if ctx_menu_item(ui, Some(Icon::Eraser), "Set to NULL", None, theme.danger, theme).clicked() {
                        set_null_req = true;
                        *close_menu = true;
                    }
                    ui.separator();
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
                        delete_row_req = true;
                        *close_menu = true;
                    }
                }
            });

            if is_ctx {
                self.selected_cell = Some((row_index, column_index));
                self.selected_row = Some(row_index);
            }
            if copy_cell_req {
                self.selected_cell = Some((row_index, column_index));
                self.copy_selected_cell(ui, result);
            }
            if copy_row_req {
                self.selected_row = Some(row_index);
                self.copy_selected_row(ui, result);
            }
            if edit_cell_req && editable {
                self.begin_data_cell_edit(row_index, column_index, display_cell);
            }
            if set_null_req && editable {
                self.data_editing_cell = Some((row_index, column_index));
                self.data_edit_value = "NULL".to_owned();
                self.submit_data_cell_edit(result, row_index, column_index);
            }
            if delete_row_req && editable {
                self.selected_row = Some(row_index);
                self.request_delete_selected_data_row(result);
            }

            if cell_resp.double_clicked() && editable {
                self.begin_data_cell_edit(row_index, column_index, display_cell);
            } else if cell_resp.clicked() && !is_ctx {
                self.commit_active_data_edit(result);
                self.selected_cell = Some((row_index, column_index));
                self.selected_row = Some(row_index);
                self.copy_status.clear();
            }
        }
    }

    /// Inline text editor for the cell currently being edited.
    fn draw_grid_cell_editor(
        &mut self,
        ui: &mut egui::Ui,
        result: &UiQueryResult,
        row_index: usize,
        column_index: usize,
        cell_rect: Rect,
    ) {
        ui.allocate_new_ui(egui::UiBuilder::new().max_rect(cell_rect.shrink(1.0)), |ui| {
            let response = ui.add_sized(
                ui.available_size(),
                TextEdit::singleline(&mut self.data_edit_value)
                    .margin(egui::Margin::symmetric(6.0, 2.0))
                    .text_color(self.theme.text_primary),
            );
            response.request_focus();
        });
        let commit = ui.input(|input| input.key_pressed(egui::Key::Enter));
        if commit {
            self.submit_data_cell_edit(result, row_index, column_index);
        } else if ui.input(|input| input.key_pressed(egui::Key::Escape)) {
            self.data_editing_cell = None;
            self.data_edit_value.clear();
        }
    }

    fn commit_active_data_edit(&mut self, result: &UiQueryResult) {
        if let Some((row_index, column_index)) = self.data_editing_cell {
            self.submit_data_cell_edit(result, row_index, column_index);
        }
    }

    fn copy_selected_cell(&mut self, ui: &mut egui::Ui, result: &UiQueryResult) {
        let Some((row_index, column_index)) = self.selected_cell else {
            self.copy_status = "Select a cell first".to_owned();
            return;
        };
        let Some(cell) = self.copy_cell_value(result, row_index, column_index) else {
            self.copy_status = "Selected cell is no longer available".to_owned();
            return;
        };
        ui.output_mut(|output| output.copied_text = crate::cell_text(&cell));
        self.copy_status = "Cell copied".to_owned();
    }

    fn copy_selected_row(&mut self, ui: &mut egui::Ui, result: &UiQueryResult) {
        let Some(row_index) = self.selected_row else {
            self.copy_status = "Select a row first".to_owned();
            return;
        };
        let Some(row) = result.rows.get(row_index) else {
            self.copy_status = "Selected row is no longer available".to_owned();
            return;
        };
        let row_text = row
            .iter()
            .enumerate()
            .map(|(column_index, cell)| {
                let copied_cell = self
                    .copy_cell_value(result, row_index, column_index)
                    .unwrap_or_else(|| cell.clone());
                crate::cell_text(&copied_cell)
            })
            .collect::<Vec<_>>()
            .join("\t");
        ui.output_mut(|output| output.copied_text = row_text);
        self.copy_status = "Row copied".to_owned();
    }

    pub(crate) fn copy_cell_value(
        &self,
        result: &UiQueryResult,
        row_index: usize,
        column_index: usize,
    ) -> Option<crate::UiCell> {
        let cell = result.rows.get(row_index).and_then(|row| row.get(column_index))?;
        if self.active_tab == WorkspaceTab::Table && self.table_view == TableView::Data {
            Some(
                self.staged_cell_value(row_index, column_index)
                    .unwrap_or_else(|| cell.clone()),
            )
        } else {
            Some(cell.clone())
        }
    }

    pub(super) fn column_widths(&mut self, count: usize, available_width: f32) -> Vec<f32> {
        if self.grid_column_widths.len() != count {
            self.grid_column_widths = vec![180.0; count];
            self.grid_columns_user_resized = false;
        }
        let mut widths = self.grid_column_widths.clone();
        if count > 0 && !self.grid_columns_user_resized {
            let usable_width = (available_width - GRID_ROW_NUMBER_WIDTH - 4.0 * count as f32).max(0.0);
            let default_width = (usable_width / count as f32).clamp(180.0, 520.0);
            widths.fill(default_width);
        }
        widths
    }

    fn draw_grid_header(&mut self, ui: &mut egui::Ui, result: &UiQueryResult, widths: &[f32]) {
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing = Vec2::ZERO;
            let (gutter_rect, _) = ui.allocate_exact_size(egui::vec2(GRID_ROW_NUMBER_WIDTH, 34.0), Sense::hover());
            ui.painter()
                .rect_filled(gutter_rect, Rounding::ZERO, self.theme.surface_panel);
            ui.painter().hline(
                gutter_rect.x_range(),
                gutter_rect.bottom(),
                Stroke::new(1.0, self.theme.border_subtle.linear_multiply(0.7)),
            );
            ui.painter().vline(
                gutter_rect.right(),
                gutter_rect.y_range(),
                Stroke::new(1.0, self.theme.border_subtle.linear_multiply(0.7)),
            );
            ui.painter().text(
                gutter_rect.center(),
                Align2::CENTER_CENTER,
                "#",
                FontId::monospace(11.5),
                self.theme.text_muted,
            );

            for (index, column) in result.columns.iter().enumerate() {
                let width = widths.get(index).copied().unwrap_or(180.0);
                let (col_rect, col_resp) = ui.allocate_exact_size(egui::vec2(width, 34.0), Sense::click());

                // Resize divider on the right edge (4px grab target)
                let divider_rect = Rect::from_min_max(
                    Pos2::new(col_rect.right() - 3.0, col_rect.top()),
                    Pos2::new(col_rect.right() + 3.0, col_rect.bottom()),
                );
                let divider_id = ui.id().with(("grid_col_resize", index));
                let divider = ui.interact(divider_rect, divider_id, Sense::drag());
                let is_resizing = divider.hovered() || divider.dragged();
                if is_resizing {
                    ui.ctx().set_cursor_icon(egui::CursorIcon::ResizeHorizontal);
                }

                let col_hovered = col_resp.hovered() && !is_resizing;
                let bg_fill = if col_hovered {
                    self.theme.surface_hover.linear_multiply(0.4)
                } else {
                    self.theme.surface_panel
                };
                ui.painter().rect_filled(col_rect, Rounding::ZERO, bg_fill);

                // Bottom and right border lines
                ui.painter().hline(
                    col_rect.x_range(),
                    col_rect.bottom(),
                    Stroke::new(1.0, self.theme.border_subtle.linear_multiply(0.7)),
                );
                ui.painter().vline(
                    col_rect.right(),
                    col_rect.y_range(),
                    Stroke::new(
                        if divider.dragged() { 2.0 } else { 1.0 },
                        if is_resizing {
                            self.theme.accent
                        } else {
                            self.theme.border_subtle.linear_multiply(0.7)
                        },
                    ),
                );

                // Column title + data type + sort icon
                let sort_marker = match self.grid_sort_column {
                    Some(active) if active == index && self.grid_sort_desc => " ↓",
                    Some(active) if active == index => " ↑",
                    _ => "",
                };

                let header_text_rect = col_rect.shrink2(egui::vec2(10.0, 4.0));
                let painter = ui.painter().with_clip_rect(header_text_rect);

                // Draw column name and data type
                let col_name_galley = painter.layout_no_wrap(
                    column.name.clone(),
                    DbProTheme::ui_medium_font(12.5),
                    self.theme.text_primary,
                );
                let name_width = col_name_galley.size().x;
                painter.galley(
                    Pos2::new(
                        header_text_rect.left(),
                        header_text_rect.center().y - col_name_galley.size().y * 0.5,
                    ),
                    col_name_galley,
                    self.theme.text_primary,
                );

                let type_text = format!(" {}{}", column.data_type, sort_marker);
                let type_galley = painter.layout_no_wrap(
                    type_text,
                    FontId::monospace(10.5),
                    if self.grid_sort_column == Some(index) {
                        self.theme.accent
                    } else {
                        self.theme.text_muted
                    },
                );
                painter.galley(
                    Pos2::new(
                        header_text_rect.left() + name_width + 4.0,
                        header_text_rect.center().y - type_galley.size().y * 0.5,
                    ),
                    type_galley,
                    if self.grid_sort_column == Some(index) {
                        self.theme.accent
                    } else {
                        self.theme.text_muted
                    },
                );

                if col_resp.clicked() && !divider.dragged() {
                    if self.grid_sort_column == Some(index) {
                        self.grid_sort_desc = !self.grid_sort_desc;
                    } else {
                        self.grid_sort_column = Some(index);
                        self.grid_sort_desc = false;
                    }
                }

                if divider.drag_started() {
                    self.grid_column_widths = widths.to_vec();
                    self.grid_columns_user_resized = true;
                }
                if divider.dragged() {
                    self.grid_column_widths[index] =
                        (self.grid_column_widths[index] + divider.drag_delta().x).clamp(60.0, 1000.0);
                }
            }
        });
    }

    fn cell_label(cell: &crate::UiCell) -> String {
        match cell {
            crate::UiCell::Null => "NULL".to_owned(),
            crate::UiCell::Boolean(value) => value.to_string(),
            crate::UiCell::Number(value) => value.clone(),
            crate::UiCell::Text(value) => value.clone(),
            crate::UiCell::Json(value) => value.clone(),
            crate::UiCell::Bytes(value) => value.clone(),
        }
    }
}
