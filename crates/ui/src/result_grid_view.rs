use super::*;

impl DbProApp {
    pub(super) fn draw_result_grid(&mut self, ui: &mut egui::Ui, result: &UiQueryResult) {
        if result.columns.is_empty() {
            ui.centered_and_justified(|ui| {
                ui.label(RichText::new("Statement completed without rows").color(self.theme.text_muted));
            });
            return;
        }

        let editable = self.active_tab == WorkspaceTab::Table
            && self.table_view == TableView::Data
            && self.can_mutate_active_connection();
        let indexes =
            crate::filtered_sorted_indexes(result, &self.grid_filter, self.grid_sort_column, self.grid_sort_desc);

        if ui.input(|input| input.key_pressed(egui::Key::C) && Self::primary_modifier_pressed(input)) {
            self.copy_selected_cell(ui, result);
        }
        let pasted = ui.input(|input| {
            input.events.iter().find_map(|event| match event {
                egui::Event::Paste(text) => Some(text.clone()),
                _ => None,
            })
        });
        if editable {
            if let (Some((row_index, column_index)), Some(text)) = (self.selected_cell, pasted) {
                self.data_editing_cell = Some((row_index, column_index));
                self.data_edit_value = text;
                self.submit_data_cell_edit(result, row_index, column_index);
            }
            if self.data_editing_cell.is_none()
                && self.selected_cell.is_some()
                && ui.input(|input| input.key_pressed(egui::Key::Enter))
            {
                if let Some((row_index, column_index)) = self.selected_cell {
                    if let Some(cell) = result.rows.get(row_index).and_then(|row| row.get(column_index)) {
                        self.begin_data_cell_edit(row_index, column_index, cell);
                    }
                }
            }
        }
        if !ui.ctx().wants_keyboard_input() {
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
            if let Some(key) = navigation_key {
                if let Some(selection) =
                    crate::grid_keyboard_selection(self.selected_cell, &indexes, result.columns.len(), key)
                {
                    self.selected_cell = Some(selection);
                    self.selected_row = Some(selection.0);
                    self.data_editing_cell = None;
                    self.data_edit_value.clear();
                    self.copy_status.clear();
                }
            }
        }
        ui.horizontal_wrapped(|ui| {
            ui.label(RichText::new("Filter").small().color(self.theme.text_secondary));
            input(ui, &mut self.grid_filter, "Search visible rows…", 240.0, self.theme);
            if compact_button(ui, "Clear", self.theme).clicked() {
                self.grid_filter.clear();
            }
            if compact_button(ui, "Copy cell", self.theme).clicked() {
                self.copy_selected_cell(ui, result);
            }
            if compact_button(ui, "Copy row", self.theme).clicked() {
                self.copy_selected_row(ui, result);
            }
            if !self.copy_status.is_empty() {
                ui.label(
                    RichText::new(self.copy_status.as_str())
                        .small()
                        .color(self.theme.success),
                );
            }
            ui.label(
                RichText::new(if editable {
                    "Arrows move · Enter or double-click to edit · drag divider to resize"
                } else {
                    "Click a cell or use arrows to select · drag divider to resize"
                })
                .small()
                .color(self.theme.text_muted),
            );
        });
        ui.add_space(6.0);

        ui.label(
            RichText::new(format!("{} matching rows", indexes.len()))
                .small()
                .color(self.theme.text_muted),
        );
        ui.add_space(4.0);
        let grid_height = ui.available_height().clamp(220.0, 520.0);
        let grid_width = ui.available_width().max(0.0);

        let widths = self.column_widths(result.columns.len(), grid_width);
        let row_offset = if self.active_tab == WorkspaceTab::Table && self.table_view == TableView::Data {
            self.table_data_offset
        } else {
            0
        };
        ui.allocate_ui_with_layout(
            egui::vec2(grid_width, grid_height),
            Layout::top_down(Align::Min),
            |ui| {
                egui::ScrollArea::horizontal().show(ui, |ui| {
                    let content_width =
                        GRID_ROW_NUMBER_WIDTH + widths.iter().sum::<f32>() + 4.0 * result.columns.len() as f32;
                    ui.set_min_width(content_width);
                    self.draw_grid_header(ui, result, &widths);
                    egui::ScrollArea::vertical()
                        .max_height((grid_height - 28.0).max(192.0))
                        .show_rows(ui, 24.0, indexes.len(), |ui, range| {
                            for position in range {
                                let row_index = indexes[position];
                                let row = &result.rows[row_index];
                                let row_selected = self.selected_row == Some(row_index);
                                let row_dirty = self.staged_row_deleted(row_index)
                                    || (0..result.columns.len())
                                        .any(|column_index| self.staged_cell_value(row_index, column_index).is_some());
                                ui.horizontal(|ui| {
                                    let row_number = crate::displayed_row_number(row_offset, row_index);
                                    let row_response = ui.add_sized(
                                        [GRID_ROW_NUMBER_WIDTH, 24.0],
                                        egui::SelectableLabel::new(
                                            row_selected,
                                            RichText::new(row_number.to_string()).monospace().small().color(
                                                if row_selected {
                                                    self.theme.accent
                                                } else {
                                                    self.theme.text_muted
                                                },
                                            ),
                                        ),
                                    );
                                    if row_response.clicked() {
                                        self.commit_active_data_edit(result);
                                        self.selected_cell = None;
                                        self.selected_row = Some(row_index);
                                        self.data_editing_cell = None;
                                        self.data_edit_value.clear();
                                        self.copy_status.clear();
                                    }
                                    for (column_index, cell) in row.iter().enumerate().take(result.columns.len()) {
                                        let staged_cell = self.staged_cell_value(row_index, column_index);
                                        let display_cell = staged_cell.as_ref().unwrap_or(cell);
                                        let width = widths.get(column_index).copied().unwrap_or(180.0);
                                        let cell_selected = self.selected_cell == Some((row_index, column_index));
                                        let fill = if row_selected {
                                            if cell_selected {
                                                self.theme.accent.linear_multiply(0.30)
                                            } else {
                                                self.theme.surface_active
                                            }
                                        } else if row_dirty {
                                            self.theme.warning.linear_multiply(0.10)
                                        } else if position % 2 == 0 {
                                            self.theme.surface_panel
                                        } else {
                                            self.theme.surface_elevated
                                        };
                                        egui::Frame::default().fill(fill).show(ui, |ui| {
                                            ui.allocate_ui_with_layout(
                                                egui::vec2(width, 24.0),
                                                Layout::left_to_right(Align::Center),
                                                |ui| {
                                                    ui.add_space(8.0);
                                                    let selected =
                                                        cell_selected || (row_selected && self.selected_cell.is_none());
                                                    let editing = editable
                                                        && self.data_editing_cell == Some((row_index, column_index));
                                                    if editing {
                                                        let response = ui.add_sized(
                                                            [width - 12.0, 22.0],
                                                            TextEdit::singleline(&mut self.data_edit_value)
                                                                .margin(egui::Margin::symmetric(6.0, 2.0))
                                                                .text_color(self.theme.text_primary),
                                                        );
                                                        response.request_focus();
                                                        let commit =
                                                            ui.input(|input| input.key_pressed(egui::Key::Enter));
                                                        if commit {
                                                            self.submit_data_cell_edit(result, row_index, column_index);
                                                        } else if ui.input(|input| input.key_pressed(egui::Key::Escape))
                                                        {
                                                            self.data_editing_cell = None;
                                                            self.data_edit_value.clear();
                                                        }
                                                    } else {
                                                        let response = ui.add_sized(
                                                            [width - 12.0, 22.0],
                                                            egui::SelectableLabel::new(
                                                                selected,
                                                                Self::cell_label(display_cell),
                                                            ),
                                                        );
                                                        if response.double_clicked() && editable {
                                                            self.begin_data_cell_edit(
                                                                row_index,
                                                                column_index,
                                                                display_cell,
                                                            );
                                                        } else if response.clicked() {
                                                            self.commit_active_data_edit(result);
                                                            self.selected_cell = Some((row_index, column_index));
                                                            self.selected_row = Some(row_index);
                                                            self.copy_status.clear();
                                                        }
                                                    }
                                                },
                                            );
                                        });
                                    }
                                });
                            }
                        });
                });
            },
        );
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
            ui.add_sized(
                [GRID_ROW_NUMBER_WIDTH, 28.0],
                egui::Button::new(RichText::new("#").strong().color(self.theme.text_muted))
                    .fill(self.theme.surface_hover)
                    .stroke(egui::Stroke::NONE)
                    .rounding(egui::Rounding::ZERO),
            );
            for (index, column) in result.columns.iter().enumerate() {
                let width = widths.get(index).copied().unwrap_or(180.0);
                let sort_marker = match self.grid_sort_column {
                    Some(active) if active == index && self.grid_sort_desc => " ↓",
                    Some(active) if active == index => " ↑",
                    _ => "",
                };
                let response = ui.add_sized(
                    [width, 28.0],
                    egui::Button::new(
                        RichText::new(format!("{} · {}{}", column.name, column.data_type, sort_marker))
                            .strong()
                            .color(self.theme.text_primary),
                    )
                    .fill(self.theme.surface_hover)
                    .stroke(egui::Stroke::NONE)
                    .rounding(egui::Rounding::ZERO),
                );
                if response.clicked() {
                    if self.grid_sort_column == Some(index) {
                        self.grid_sort_desc = !self.grid_sort_desc;
                    } else {
                        self.grid_sort_column = Some(index);
                        self.grid_sort_desc = false;
                    }
                }
                let (divider_rect, divider) = ui.allocate_exact_size(egui::vec2(4.0, 28.0), Sense::drag());
                ui.painter().vline(
                    divider_rect.center().x,
                    divider_rect.y_range(),
                    egui::Stroke::new(1.0, self.theme.border_subtle),
                );
                if divider.drag_started() {
                    self.grid_column_widths = widths.to_vec();
                    self.grid_columns_user_resized = true;
                    self.grid_resize_start = Some((index, width));
                }
                if divider.dragged() {
                    let start_width = self
                        .grid_resize_start
                        .filter(|(column, _)| *column == index)
                        .map(|(_, start)| start)
                        .unwrap_or(width);
                    self.grid_column_widths[index] = (start_width + divider.drag_delta().x).clamp(90.0, 520.0);
                }
            }
        });
    }

    fn cell_label(cell: &crate::UiCell) -> RichText {
        match cell {
            crate::UiCell::Null => RichText::new("NULL").italics(),
            crate::UiCell::Boolean(value) => RichText::new(value.to_string()),
            crate::UiCell::Number(value) => RichText::new(value.as_str()).monospace(),
            crate::UiCell::Text(value) => RichText::new(value.as_str()),
            crate::UiCell::Json(value) => RichText::new(value.as_str()).monospace(),
            crate::UiCell::Bytes(value) => RichText::new(value.as_str()).monospace(),
        }
    }
}
