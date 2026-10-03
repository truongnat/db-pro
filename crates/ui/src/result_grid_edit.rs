//! In-cell editor + advanced value inspector for the result grid (#228).
use super::*;
use egui::Rect;

#[path = "result_grid_cell_editor_surface_view.rs"]
mod result_grid_cell_editor_surface_view;
#[path = "result_grid_date_picker_view.rs"]
mod result_grid_date_picker_view;
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
        let kind = self.classify_cell_editor(result, column_index);
        let is_expanded = self.table.editing.expanded_data_editor == Some((row_index, column_index));
        if !is_expanded {
            let action = {
                let mut context = result_grid_cell_editor_surface_view::CellEditorContext {
                    theme: self.theme,
                    kind,
                    value: &mut self.table.editing.data_edit_value,
                    error: &mut self.table.editing.data_edit_error,
                };
                result_grid_cell_editor_surface_view::draw(&mut context, ui, cell_rect)
            };
            match action {
                Some(result_grid_cell_editor_surface_view::CellEditorAction::Commit) => {
                    self.submit_data_cell_edit(result, row_index, column_index);
                }
                Some(result_grid_cell_editor_surface_view::CellEditorAction::Cancel) => {
                    self.close_data_inspector();
                }
                None => {}
            }
            return;
        }

        self.draw_advanced_cell_inspector(ui, result, row_index, column_index);
    }

    /// Picks the inline editor widget from the column's declared data type.
    /// For the table data view, enum labels come from the introspected table
    /// metadata; for other enum-typed results the select falls back to the
    /// distinct values already present in the result.
    fn classify_cell_editor(
        &self,
        result: &UiQueryResult,
        column_index: usize,
    ) -> result_grid_cell_editor_surface_view::CellEditorKind {
        use result_grid_cell_editor_surface_view::CellEditorKind;
        let Some(column) = result.columns.get(column_index) else {
            return CellEditorKind::Text;
        };
        let data_type = column.data_type.to_ascii_lowercase();
        if data_type.contains("bool") {
            return CellEditorKind::Boolean {
                nullable: column.nullable,
            };
        }
        if let Some(labels) = self
            .table
            .state
            .table_info
            .as_ref()
            .and_then(|info| info.columns.iter().find(|item| item.name == column.name))
            .filter(|item| !item.enum_labels.is_empty())
            .map(|item| item.enum_labels.clone())
        {
            let mut values = labels;
            if column.nullable {
                values.push("NULL".to_owned());
            }
            return CellEditorKind::Select(values);
        }
        if data_type.contains("enum") {
            let mut seen = std::collections::BTreeSet::new();
            for row in &result.rows {
                if let Some(UiCell::Text(value)) = row.get(column_index) {
                    seen.insert(value.clone());
                }
                if seen.len() >= 200 {
                    break;
                }
            }
            let mut values: Vec<String> = seen.into_iter().collect();
            if column.nullable {
                values.push("NULL".to_owned());
            }
            return CellEditorKind::Select(values);
        }
        if data_type.starts_with("date") || data_type.contains("timestamp") || data_type.contains("datetime") {
            return CellEditorKind::Temporal {
                date_only: data_type.starts_with("date") && !data_type.contains("time"),
            };
        }
        CellEditorKind::Text
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

#[cfg(test)]
mod tests {
    use super::result_grid_cell_editor_surface_view::CellEditorKind;
    use super::*;
    use crate::{UiColumn, UiTableColumn, UiTableInfo};

    fn result(columns: &[(&str, &str, bool)], rows: Vec<Vec<UiCell>>) -> UiQueryResult {
        UiQueryResult {
            columns: columns
                .iter()
                .map(|(name, data_type, nullable)| UiColumn {
                    name: name.to_string(),
                    data_type: data_type.to_string(),
                    nullable: *nullable,
                })
                .collect(),
            row_count: rows.len() as u64,
            rows,
            duration_ms: 0,
        }
    }

    #[test]
    fn boolean_columns_use_the_select_editor() {
        let app = DbProApp::default();
        let result = result(&[("active", "boolean", true)], Vec::new());
        assert_eq!(
            app.classify_cell_editor(&result, 0),
            CellEditorKind::Boolean { nullable: true }
        );
    }

    #[test]
    fn enum_columns_use_labels_from_table_metadata() {
        let mut app = DbProApp::default();
        app.table.state.table_info = Some(UiTableInfo {
            schema: "main".to_owned(),
            name: "customers".to_owned(),
            row_count: None,
            columns: vec![UiTableColumn {
                name: "status".to_owned(),
                data_type: "order_status".to_owned(),
                nullable: true,
                enum_labels: vec!["pending".to_owned(), "shipped".to_owned()],
                ..Default::default()
            }],
            primary_key: None,
            indexes: Vec::new(),
            foreign_keys: Vec::new(),
            check_constraints: Vec::new(),
            dependencies: Vec::new(),
        });
        let result = result(&[("status", "order_status", true)], Vec::new());
        assert_eq!(
            app.classify_cell_editor(&result, 0),
            CellEditorKind::Select(vec!["pending".to_owned(), "shipped".to_owned(), "NULL".to_owned()])
        );
    }

    #[test]
    fn enum_named_types_fall_back_to_distinct_result_values() {
        let app = DbProApp::default();
        let result = result(
            &[("status", "enum", false)],
            vec![
                vec![UiCell::Text("open".to_owned())],
                vec![UiCell::Text("closed".to_owned())],
                vec![UiCell::Text("open".to_owned())],
                vec![UiCell::Null],
            ],
        );
        assert_eq!(
            app.classify_cell_editor(&result, 0),
            CellEditorKind::Select(vec!["closed".to_owned(), "open".to_owned()])
        );
    }

    #[test]
    fn timestamp_columns_use_the_temporal_editor() {
        let app = DbProApp::default();
        let result = result(&[("created_at", "timestamp without time zone", false)], Vec::new());
        assert_eq!(
            app.classify_cell_editor(&result, 0),
            CellEditorKind::Temporal { date_only: false }
        );
    }

    #[test]
    fn date_columns_use_a_date_only_temporal_editor() {
        let app = DbProApp::default();
        let result = result(&[("due_on", "date", false)], Vec::new());
        assert_eq!(
            app.classify_cell_editor(&result, 0),
            CellEditorKind::Temporal { date_only: true }
        );
    }

    #[test]
    fn text_and_number_columns_keep_the_text_input() {
        let app = DbProApp::default();
        let result = result(&[("name", "text", false), ("total", "numeric", false)], Vec::new());
        assert_eq!(app.classify_cell_editor(&result, 0), CellEditorKind::Text);
        assert_eq!(app.classify_cell_editor(&result, 1), CellEditorKind::Text);
    }
}
