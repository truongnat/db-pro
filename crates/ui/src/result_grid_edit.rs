//! In-cell editor + advanced value inspector for the result grid (#228).
use super::*;
use egui::Rect;

#[path = "result_grid_inspector_surface_view.rs"]
mod result_grid_inspector_surface_view;
#[path = "result_grid_record_surface_view.rs"]
mod result_grid_record_surface_view;

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
        let is_expanded = self.table.editing.expanded_data_editor == Some((row_index, column_index));
        if !is_expanded {
            ui.allocate_new_ui(egui::UiBuilder::new().max_rect(cell_rect.shrink(1.0)), |ui| {
                let response = if is_boolean {
                    let mut checked = self.table.editing.data_edit_value.eq_ignore_ascii_case("true");
                    let response = ui.checkbox(&mut checked, "");
                    if response.changed() {
                        self.table.editing.data_edit_value = checked.to_string();
                        self.table.editing.data_edit_error = None;
                    }
                    response
                } else {
                    ui.add_sized(
                        ui.available_size(),
                        TextEdit::singleline(&mut self.table.editing.data_edit_value)
                            .margin(egui::Margin::symmetric(6.0, 2.0))
                            .text_color(self.theme.text_primary),
                    )
                };
                response.request_focus();
                if response.changed() {
                    self.table.editing.data_edit_error = None;
                }
            });
            let commit = ui.input(|input| input.key_pressed(egui::Key::Enter));
            if commit {
                self.submit_data_cell_edit(result, row_index, column_index);
            } else if ui.input(|input| input.key_pressed(egui::Key::Escape)) {
                self.table.editing.data_editing_cell = None;
                self.table.editing.expanded_data_editor = None;
                self.table.editing.data_edit_value.clear();
                self.table.editing.data_edit_error = None;
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
        let writable = write_block.is_none() && self.table.editing.data_editing_cell.is_some();
        let cell = result
            .rows
            .get(row_index)
            .and_then(|row| row.get(column_index))
            .cloned()
            .unwrap_or(UiCell::Null);
        let kind = cell_inspector::classify_cell(&cell, &data_type);
        let action = {
            let mut context = result_grid_inspector_surface_view::ValueInspectorContext {
                theme: self.theme,
                row_index,
                column_index,
                column_name: &column_name,
                data_type: &data_type,
                cell,
                kind,
                write_block,
                writable,
                mode: &mut self.table.editing.cell_inspector_mode,
                value: &mut self.table.editing.data_edit_value,
                error: &mut self.table.editing.data_edit_error,
            };
            result_grid_inspector_surface_view::draw(&mut context, &ctx)
        };

        match action {
            Some(result_grid_inspector_surface_view::ValueInspectorAction::CopyRaw) => {
                ctx.copy_text(self.table.editing.data_edit_value.clone());
                self.feedback.runtime_message = "Copied raw value".into();
            }
            Some(result_grid_inspector_surface_view::ValueInspectorAction::ExportBytes) => {
                self.export_inspected_bytes();
            }
            Some(result_grid_inspector_surface_view::ValueInspectorAction::Apply) => {
                self.submit_data_cell_edit(result, row_index, column_index);
            }
            Some(result_grid_inspector_surface_view::ValueInspectorAction::Close) => {
                self.close_data_inspector();
            }
            None if ctx.input(|input| input.key_pressed(egui::Key::Escape)) => {
                self.close_data_inspector();
            }
            None => {}
        }
    }

    fn close_data_inspector(&mut self) {
        self.table.editing.data_editing_cell = None;
        self.table.editing.expanded_data_editor = None;
        self.table.editing.data_edit_value.clear();
        self.table.editing.data_edit_error = None;
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
            self.table.editing.data_editing_cell = Some((row_index, column_index));
        } else {
            self.table.editing.data_editing_cell = None;
        }
        self.table.editing.expanded_data_editor = Some((row_index, column_index));
        self.table.editing.cell_inspector_mode = match cell_inspector::classify_cell(
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
        self.table.editing.data_edit_error = None;
        self.table.editing.data_edit_value = cell_inspector::cell_raw_text(cell);
        if let Some(reason) = write_block {
            self.feedback.runtime_message = reason.reason().to_owned();
        }
    }

    fn export_inspected_bytes(&mut self) {
        match cell_inspector::decode_bytes_payload(&self.table.editing.data_edit_value) {
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
        if !self.table.editing.record_inspector_open {
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
        let context = result_grid_record_surface_view::RecordInspectorContext {
            theme: self.theme,
            row_index,
            columns: &result.columns,
            row,
        };
        match result_grid_record_surface_view::draw(&context, ui) {
            Some(result_grid_record_surface_view::RecordInspectorAction::Close) => {
                self.table.editing.record_inspector_open = false;
            }
            Some(result_grid_record_surface_view::RecordInspectorAction::Inspect(column_index)) => {
                self.open_cell_inspector(result, row_index, column_index);
            }
            None => {}
        }
    }

    pub(super) fn commit_active_data_edit(&mut self, result: &UiQueryResult) -> bool {
        if let Some((row_index, column_index)) = self.table.editing.data_editing_cell {
            self.submit_data_cell_edit(result, row_index, column_index)
        } else {
            true
        }
    }
}
