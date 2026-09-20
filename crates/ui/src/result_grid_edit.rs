//! In-cell editor + advanced value inspector for the result grid (#228).
use super::*;
use crate::components::{Button, ButtonSize, ButtonVariant, SegmentedTabs};
use egui::Rect;

impl DbProApp {
    pub(super) fn draw_grid_cell_editor(
        &mut self,
        ui: &mut egui::Ui,
        result: &UiQueryResult,
        row_index: usize,
        column_index: usize,
        cell_rect: Rect,
    ) {
        let is_boolean = result
            .columns
            .get(column_index)
            .is_some_and(|column| column.data_type.to_ascii_lowercase().contains("bool"));
        let is_expanded = self.table.data.expanded_data_editor == Some((row_index, column_index));
        if !is_expanded {
            ui.allocate_new_ui(egui::UiBuilder::new().max_rect(cell_rect.shrink(1.0)), |ui| {
                let response = if is_boolean {
                    let mut checked = self.table.data.data_edit_value.eq_ignore_ascii_case("true");
                    let response = ui.checkbox(&mut checked, "");
                    if response.changed() {
                        self.table.data.data_edit_value = checked.to_string();
                        self.table.data.data_edit_error = None;
                    }
                    response
                } else {
                    ui.add_sized(
                        ui.available_size(),
                        TextEdit::singleline(&mut self.table.data.data_edit_value)
                            .margin(egui::Margin::symmetric(6.0, 2.0))
                            .text_color(self.theme.text_primary),
                    )
                };
                response.request_focus();
                if response.changed() {
                    self.table.data.data_edit_error = None;
                }
            });
            let commit = ui.input(|input| input.key_pressed(egui::Key::Enter));
            if commit {
                self.submit_data_cell_edit(result, row_index, column_index);
            } else if ui.input(|input| input.key_pressed(egui::Key::Escape)) {
                self.table.data.data_editing_cell = None;
                self.table.data.expanded_data_editor = None;
                self.table.data.data_edit_value.clear();
                self.table.data.data_edit_error = None;
            }
            return;
        }

        self.draw_advanced_cell_inspector(ui, result, row_index, column_index);
    }

    pub(super) fn draw_advanced_cell_inspector(
        &mut self,
        ui: &mut egui::Ui,
        result: &UiQueryResult,
        row_index: usize,
        column_index: usize,
    ) {
        let ctx = ui.ctx().clone();
        let mut open = true;
        let mut commit = false;
        let mut cancel = false;
        let column_name = result
            .columns
            .get(column_index)
            .map(|c| c.name.clone())
            .unwrap_or_else(|| format!("col{column_index}"));
        let data_type = result
            .columns
            .get(column_index)
            .map(|c| c.data_type.clone())
            .unwrap_or_default();
        let write_block = self
            .table
            .state
            .column_write_policy(&column_name)
            .and_then(|policy| policy.write_block());
        let writable = write_block.is_none() && self.table.data.data_editing_cell.is_some();
        let cell = result
            .rows
            .get(row_index)
            .and_then(|row| row.get(column_index))
            .cloned()
            .unwrap_or(UiCell::Null);
        let kind = cell_inspector::classify_cell(&cell, &data_type);
        let title = format!("Value inspector · {column_name}");

        egui::Window::new(title)
            .id(egui::Id::new(("advanced-cell-inspector", row_index, column_index)))
            .open(&mut open)
            .resizable(true)
            .default_width(560.0)
            .default_height(420.0)
            .show(&ctx, |ui| {
                ui.label(
                    RichText::new(format!(
                        "{data_type} · {} · {}",
                        if matches!(cell, UiCell::Null) { "NULL" } else { "value" },
                        write_block.map(|b| b.reason()).unwrap_or(if writable {
                            "editable via ChangeSet"
                        } else {
                            "view only"
                        })
                    ))
                    .small()
                    .color(self.theme.text_muted),
                );

                let modes: &[cell_inspector::CellInspectorMode] = match kind {
                    cell_inspector::CellInspectorKind::Json => &[
                        cell_inspector::CellInspectorMode::Raw,
                        cell_inspector::CellInspectorMode::Pretty,
                        cell_inspector::CellInspectorMode::Tree,
                    ],
                    cell_inspector::CellInspectorKind::Bytes => &[
                        cell_inspector::CellInspectorMode::Raw,
                        cell_inspector::CellInspectorMode::Hex,
                        cell_inspector::CellInspectorMode::Base64,
                    ],
                    _ => &[
                        cell_inspector::CellInspectorMode::Raw,
                        cell_inspector::CellInspectorMode::Pretty,
                    ],
                };
                let mut selected_mode = modes
                    .iter()
                    .position(|mode| *mode == self.table.data.cell_inspector_mode)
                    .unwrap_or_default();
                let mode_labels: Vec<&str> = modes.iter().map(|mode| mode.as_label()).collect();
                ui.horizontal(|ui| {
                    SegmentedTabs::new(&mut selected_mode, &mode_labels, self.theme).show(ui);
                    if let Some(mode) = modes.get(selected_mode) {
                        self.table.data.cell_inspector_mode = *mode;
                    }
                    if Button::new(self.theme)
                        .text("Copy raw")
                        .size(ButtonSize::Sm)
                        .variant(ButtonVariant::Ghost)
                        .show(ui)
                        .clicked()
                    {
                        ui.ctx().copy_text(self.table.data.data_edit_value.clone());
                        self.feedback.runtime_message = "Copied raw value".into();
                    }
                    if kind == cell_inspector::CellInspectorKind::Bytes
                        && Button::new(self.theme)
                            .text("Export bytes…")
                            .size(ButtonSize::Sm)
                            .variant(ButtonVariant::Secondary)
                            .show(ui)
                            .clicked()
                    {
                        self.export_inspected_bytes();
                    }
                });

                ui.add_space(6.0);
                match self.table.data.cell_inspector_mode {
                    cell_inspector::CellInspectorMode::Raw => {
                        if writable {
                            let response = ui.add(
                                TextEdit::multiline(&mut self.table.data.data_edit_value)
                                    .desired_width(ui.available_width())
                                    .desired_rows(16),
                            );
                            if response.changed() {
                                self.table.data.data_edit_error = None;
                            }
                        } else {
                            ui.add(
                                TextEdit::multiline(&mut self.table.data.data_edit_value)
                                    .desired_width(ui.available_width())
                                    .desired_rows(16)
                                    .interactive(false),
                            );
                        }
                    }
                    cell_inspector::CellInspectorMode::Pretty => {
                        let pretty = cell_inspector::pretty_json(&self.table.data.data_edit_value)
                            .unwrap_or_else(|| self.table.data.data_edit_value.clone());
                        ui.add(
                            TextEdit::multiline(&mut pretty.clone())
                                .desired_width(ui.available_width())
                                .desired_rows(16)
                                .interactive(false),
                        );
                        if writable
                            && Button::new(self.theme)
                                .text("Use pretty as edit buffer")
                                .size(ButtonSize::Sm)
                                .variant(ButtonVariant::Ghost)
                                .show(ui)
                                .clicked()
                        {
                            self.table.data.data_edit_value = pretty;
                            self.table.data.cell_inspector_mode = cell_inspector::CellInspectorMode::Raw;
                        }
                    }
                    cell_inspector::CellInspectorMode::Tree => {
                        egui::ScrollArea::vertical().max_height(280.0).show(ui, |ui| {
                            for line in cell_inspector::json_tree_lines(&self.table.data.data_edit_value, 400) {
                                ui.label(RichText::new(line).monospace().small());
                            }
                        });
                    }
                    cell_inspector::CellInspectorMode::Hex => {
                        let text = match cell_inspector::decode_bytes_payload(&self.table.data.data_edit_value) {
                            Ok(bytes) => {
                                ui.label(
                                    RichText::new(cell_inspector::bytes_metadata(&self.table.data.data_edit_value))
                                        .small()
                                        .color(self.theme.text_secondary),
                                );
                                cell_inspector::encode_hex(&bytes)
                            }
                            Err(err) => format!("(hex unavailable: {err})"),
                        };
                        ui.add(
                            TextEdit::multiline(&mut text.clone())
                                .desired_width(ui.available_width())
                                .desired_rows(14)
                                .interactive(false),
                        );
                    }
                    cell_inspector::CellInspectorMode::Base64 => {
                        let text = match cell_inspector::decode_bytes_payload(&self.table.data.data_edit_value) {
                            Ok(bytes) => {
                                ui.label(
                                    RichText::new(cell_inspector::bytes_metadata(&self.table.data.data_edit_value))
                                        .small()
                                        .color(self.theme.text_secondary),
                                );
                                cell_inspector::encode_base64(&bytes)
                            }
                            Err(err) => format!("(base64 unavailable: {err})"),
                        };
                        ui.add(
                            TextEdit::multiline(&mut text.clone())
                                .desired_width(ui.available_width())
                                .desired_rows(10)
                                .interactive(false),
                        );
                    }
                }

                if let Some(error) = self.table.data.data_edit_error.as_deref() {
                    ui.label(RichText::new(error).small().color(self.theme.danger));
                }
                ui.horizontal(|ui| {
                    if writable
                        && Button::new(self.theme)
                            .text("Apply to ChangeSet")
                            .size(ButtonSize::Sm)
                            .show(ui)
                            .clicked()
                    {
                        commit = true;
                    }
                    if Button::new(self.theme)
                        .text("Close")
                        .size(ButtonSize::Sm)
                        .variant(ButtonVariant::Ghost)
                        .show(ui)
                        .clicked()
                    {
                        cancel = true;
                    }
                });
            });

        if commit {
            self.submit_data_cell_edit(result, row_index, column_index);
        } else if cancel || !open || ctx.input(|input| input.key_pressed(egui::Key::Escape)) {
            self.table.data.data_editing_cell = None;
            self.table.data.expanded_data_editor = None;
            self.table.data.data_edit_value.clear();
            self.table.data.data_edit_error = None;
        }
    }

    pub(super) fn open_cell_inspector(&mut self, result: &UiQueryResult, row_index: usize, column_index: usize) {
        let Some(cell) = result.rows.get(row_index).and_then(|row| row.get(column_index)) else {
            return;
        };
        self.table.data.selected_cell = Some((row_index, column_index));
        self.table.data.selected_row = Some(row_index);
        let write_block = result
            .columns
            .get(column_index)
            .and_then(|column| self.table.state.column_write_policy(&column.name))
            .and_then(|policy| policy.write_block());
        if write_block.is_none() && self.can_mutate_active_connection() {
            self.table.data.data_editing_cell = Some((row_index, column_index));
        } else {
            self.table.data.data_editing_cell = None;
        }
        self.table.data.expanded_data_editor = Some((row_index, column_index));
        self.table.data.cell_inspector_mode = match cell_inspector::classify_cell(
            cell,
            result
                .columns
                .get(column_index)
                .map(|c| c.data_type.as_str())
                .unwrap_or(""),
        ) {
            cell_inspector::CellInspectorKind::Json => cell_inspector::CellInspectorMode::Pretty,
            cell_inspector::CellInspectorKind::Bytes => cell_inspector::CellInspectorMode::Hex,
            _ => cell_inspector::CellInspectorMode::Raw,
        };
        self.table.data.data_edit_error = None;
        self.table.data.data_edit_value = cell_inspector::cell_raw_text(cell);
        if let Some(reason) = write_block {
            self.feedback.runtime_message = reason.reason().to_owned();
        }
    }

    fn export_inspected_bytes(&mut self) {
        match cell_inspector::decode_bytes_payload(&self.table.data.data_edit_value) {
            Ok(bytes) => {
                let path = std::env::temp_dir().join(format!(
                    "db-pro-cell-export-{}.bin",
                    std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .map(|d| d.as_millis())
                        .unwrap_or(0)
                ));
                match std::fs::write(&path, &bytes) {
                    Ok(()) => {
                        self.feedback.runtime_message =
                            format!("Exported {} byte(s) → {}", bytes.len(), path.display());
                    }
                    Err(err) => self.feedback.runtime_message = format!("Export failed: {err}"),
                }
            }
            Err(err) => self.feedback.runtime_message = format!("Cannot export bytes: {err}"),
        }
    }

    pub(super) fn draw_record_inspector_panel(&mut self, ui: &mut egui::Ui, result: &UiQueryResult) {
        if !self.table.data.record_inspector_open {
            return;
        }
        let Some(row_index) = self
            .table
            .data
            .selected_row
            .or_else(|| self.table.data.selected_cell.map(|(r, _)| r))
        else {
            ui.label(
                RichText::new("Select a row to inspect the full record.")
                    .small()
                    .color(self.theme.text_muted),
            );
            return;
        };
        let Some(row) = result.rows.get(row_index) else {
            return;
        };
        ui.horizontal(|ui| {
            ui.label(RichText::new(format!("Record · row {row_index}")).strong());
            if ui.small_button("Close").clicked() {
                self.table.data.record_inspector_open = false;
            }
        });
        egui::ScrollArea::vertical().max_height(220.0).show(ui, |ui| {
            for (column_index, column) in result.columns.iter().enumerate() {
                let cell = row.get(column_index).unwrap_or(&UiCell::Null);
                ui.horizontal(|ui| {
                    ui.label(
                        RichText::new(format!("{}:", column.name))
                            .small()
                            .strong()
                            .color(self.theme.text_secondary),
                    );
                    let preview = {
                        let text = cell_inspector::cell_raw_text(cell);
                        if text.chars().count() > 80 {
                            format!("{}…", text.chars().take(79).collect::<String>())
                        } else {
                            text
                        }
                    };
                    ui.label(RichText::new(preview).small().monospace());
                    if ui.small_button("Inspect").clicked() {
                        self.open_cell_inspector(result, row_index, column_index);
                    }
                });
            }
        });
    }

    pub(super) fn commit_active_data_edit(&mut self, result: &UiQueryResult) -> bool {
        if let Some((row_index, column_index)) = self.table.data.data_editing_cell {
            self.submit_data_cell_edit(result, row_index, column_index)
        } else {
            true
        }
    }
}
