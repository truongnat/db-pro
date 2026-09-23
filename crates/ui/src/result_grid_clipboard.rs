//! Result-grid clipboard, export formatting, and copy helpers.
use super::*;

impl DbProApp {
    pub(super) fn copy_selected_cell(&mut self, ui: &mut egui::Ui, result: &UiQueryResult) {
        let Some((row_index, column_index)) = self.table.data.selected_cell else {
            self.feedback.copy_status = "Select a cell first".to_owned();
            return;
        };
        self.copy_cell_at(ui, result, row_index, column_index);
    }

    pub(super) fn copy_cell_at(
        &mut self,
        ui: &mut egui::Ui,
        result: &UiQueryResult,
        row_index: usize,
        column_index: usize,
    ) {
        let Some(cell) = self.copy_cell_value(result, row_index, column_index) else {
            self.feedback.copy_status = "Selected cell is no longer available".to_owned();
            return;
        };
        ui.output_mut(|output| output.copied_text = crate::cell_text(&cell));
        self.feedback.copy_status = "Cell copied".to_owned();
    }

    /// One row as a delimited line, with each cell escaped so an embedded tab,
    /// quote or newline cannot be mistaken for a field or row boundary.
    pub(super) fn copied_row_text(&self, result: &UiQueryResult, row_index: usize, row: &[UiCell]) -> String {
        (0..result.columns.len())
            .map(|column_index| {
                self.copy_cell_value(result, row_index, column_index)
                    .unwrap_or_else(|| row.get(column_index).cloned().unwrap_or(UiCell::Null))
            })
            .map(|cell| result_grid_export::format_cell_delimited(&cell, '\t'))
            .collect::<Vec<_>>()
            .join("\t")
    }

    pub(super) fn copy_selected_row(&mut self, ui: &mut egui::Ui, result: &UiQueryResult) {
        let Some(row_index) = self.table.data.selected_row else {
            self.feedback.copy_status = "Select a row first".to_owned();
            return;
        };
        let Some(row) = result.rows.get(row_index) else {
            self.feedback.copy_status = "Selected row is no longer available".to_owned();
            return;
        };
        let row_text = self.copied_row_text(result, row_index, row);
        ui.output_mut(|output| output.copied_text = row_text);
        self.feedback.copy_status = "Row copied".to_owned();
    }

    pub(super) fn copy_selected_rows(&mut self, ui: &mut egui::Ui, result: &UiQueryResult) {
        let row_indexes = self.table.data.selected_row_indexes();
        if row_indexes.is_empty() {
            self.feedback.copy_status = "Select one or more rows first".to_owned();
            return;
        }

        let rows = row_indexes
            .iter()
            .filter_map(|&row_index| result.rows.get(row_index).map(|row| (row_index, row)))
            .map(|(row_index, row)| self.copied_row_text(result, row_index, row))
            .collect::<Vec<_>>();
        ui.output_mut(|output| output.copied_text = rows.join("\n"));
        self.feedback.copy_status = format!("{} rows copied", rows.len());
    }

    pub(super) fn copy_selected_rows_with_headers(&mut self, ui: &mut egui::Ui, result: &UiQueryResult) {
        let row_indexes = self.table.data.selected_row_indexes();
        if row_indexes.is_empty() {
            self.feedback.copy_status = "Select one or more rows first".to_owned();
            return;
        }
        let header = result
            .columns
            .iter()
            .map(|column| result_grid_export::escape_delimited_field(&column.name, '\t'))
            .collect::<Vec<_>>()
            .join("\t");
        let rows = row_indexes
            .iter()
            .filter_map(|&row_index| result.rows.get(row_index).map(|row| (row_index, row)))
            .map(|(row_index, row)| self.copied_row_text(result, row_index, row))
            .collect::<Vec<_>>();
        let mut lines = Vec::with_capacity(rows.len() + 1);
        lines.push(header);
        lines.extend(rows);
        ui.output_mut(|output| output.copied_text = lines.join("\n"));
        self.feedback.copy_status = format!("{} rows copied with headers", row_indexes.len());
    }

    pub(super) fn copy_selected_rows_as_json(&mut self, ui: &mut egui::Ui, result: &UiQueryResult) {
        let indexes = self.table.data.selected_row_indexes();
        self.copy_all_as_json(ui, result, &indexes);
    }

    pub(super) fn copy_selected_rows_as_markdown(&mut self, ui: &mut egui::Ui, result: &UiQueryResult) {
        let indexes = self.table.data.selected_row_indexes();
        self.copy_all_as_markdown(ui, result, &indexes);
    }

    pub(super) fn copy_selected_rows_as_insert(&mut self, ui: &mut egui::Ui, result: &UiQueryResult) {
        let indexes = self.table.data.selected_row_indexes();
        if indexes.is_empty() {
            self.feedback.copy_status = "Select one or more rows first".to_owned();
            return;
        }
        self.copy_all_as_insert(ui, result, &indexes);
    }

    pub(crate) fn copy_all_as_insert(&mut self, ui: &mut egui::Ui, result: &UiQueryResult, indexes: &[usize]) {
        let table = self.schema.explorer.selected_table.as_deref().unwrap_or("table_name");
        let target =
            if self.workspace.active_tab == WorkspaceTab::Table && self.table.state.table_view == TableView::Data {
                format!(
                    "{}.{}",
                    result_grid_export::quote_sql_identifier(self.active_schema()),
                    result_grid_export::quote_sql_identifier(table)
                )
            } else {
                result_grid_export::quote_sql_identifier(table)
            };
        let columns = result
            .columns
            .iter()
            .map(|column| result_grid_export::quote_sql_identifier(&column.name))
            .collect::<Vec<_>>()
            .join(", ");
        let statements = indexes
            .iter()
            .filter_map(|&row_index| result.rows.get(row_index).map(|row| (row_index, row)))
            .map(|(row_index, row)| {
                let values = (0..result.columns.len())
                    .map(|column_index| {
                        let cell = self
                            .copy_cell_value(result, row_index, column_index)
                            .unwrap_or_else(|| row.get(column_index).cloned().unwrap_or(UiCell::Null));
                        result_grid_export::cell_sql_literal(&cell)
                    })
                    .collect::<Vec<_>>()
                    .join(", ");
                format!("INSERT INTO {target} ({columns}) VALUES ({values});")
            })
            .collect::<Vec<_>>();
        ui.output_mut(|output| output.copied_text = statements.join("\n"));
        self.feedback.copy_status = format!("{} INSERT statements copied", statements.len());
    }

    pub(crate) fn copy_all_as_markdown(&mut self, ui: &mut egui::Ui, result: &UiQueryResult, indexes: &[usize]) {
        let header = format!(
            "| {} |",
            result
                .columns
                .iter()
                .map(|c| c.name.replace('|', "\\|"))
                .collect::<Vec<_>>()
                .join(" | ")
        );
        let separator = format!(
            "| {} |",
            result
                .columns
                .iter()
                .map(|_| "---")
                .collect::<Vec<_>>()
                .join(" | ")
        );
        let mut lines = vec![header, separator];
        for &row_index in indexes {
            if let Some(row) = result.rows.get(row_index) {
                let row_str = format!(
                    "| {} |",
                    (0..result.columns.len())
                        .map(|col_idx| {
                            let cell = self
                                .copy_cell_value(result, row_index, col_idx)
                                .unwrap_or_else(|| row.get(col_idx).cloned().unwrap_or(UiCell::Null));
                            crate::result_grid::cell_text_as_str(&cell).replace('|', "\\|")
                        })
                        .collect::<Vec<_>>()
                        .join(" | ")
                );
                lines.push(row_str);
            }
        }
        ui.output_mut(|output| output.copied_text = lines.join("\n"));
        self.feedback.copy_status = format!("{} rows copied as Markdown", indexes.len());
    }

    pub(crate) fn copy_column_name(&mut self, ui: &mut egui::Ui, result: &UiQueryResult, column_index: usize) {
        if let Some(col) = result.columns.get(column_index) {
            ui.output_mut(|output| output.copied_text = col.name.clone());
            self.feedback.copy_status = format!("Column name '{}' copied", col.name);
        }
    }

    pub(crate) fn copy_column_values(
        &mut self,
        ui: &mut egui::Ui,
        result: &UiQueryResult,
        column_index: usize,
        indexes: &[usize],
    ) {
        let values = indexes
            .iter()
            .filter_map(|&row_index| result.rows.get(row_index))
            .map(|row| {
                let cell = row.get(column_index).unwrap_or(&UiCell::Null);
                crate::result_grid::cell_text_as_str(cell)
            })
            .collect::<Vec<_>>();
        ui.output_mut(|output| output.copied_text = values.join("\n"));
        self.feedback.copy_status = format!("{} column values copied", values.len());
    }

    pub(crate) fn copy_row_as_json(&mut self, ui: &mut egui::Ui, result: &UiQueryResult, row_index: usize) {
        let Some(row) = result.rows.get(row_index) else {
            return;
        };
        let mut map = serde_json::Map::new();
        for (col_idx, col) in result.columns.iter().enumerate() {
            let cell = self
                .copy_cell_value(result, row_index, col_idx)
                .unwrap_or_else(|| row.get(col_idx).cloned().unwrap_or(UiCell::Null));
            map.insert(col.name.clone(), result_grid_export::cell_to_json_value(&cell));
        }
        let json_text = serde_json::to_string_pretty(&serde_json::Value::Object(map)).unwrap_or_default();
        ui.output_mut(|output| output.copied_text = json_text);
        self.feedback.copy_status = "Row copied as JSON".to_owned();
    }

    pub(crate) fn copy_row_as_csv(&mut self, ui: &mut egui::Ui, result: &UiQueryResult, row_index: usize) {
        let Some(row) = result.rows.get(row_index) else {
            return;
        };
        let header = result
            .columns
            .iter()
            .map(|column| result_grid_export::escape_delimited_field(&column.name, ','))
            .collect::<Vec<_>>()
            .join(",");
        let row_values = (0..result.columns.len())
            .map(|col_idx| {
                let cell = self
                    .copy_cell_value(result, row_index, col_idx)
                    .unwrap_or_else(|| row.get(col_idx).cloned().unwrap_or(UiCell::Null));
                result_grid_export::format_cell_csv(&cell)
            })
            .collect::<Vec<_>>()
            .join(",");
        ui.output_mut(|output| output.copied_text = format!("{header}\n{row_values}"));
        self.feedback.copy_status = "Row copied as CSV".to_owned();
    }

    pub(crate) fn copy_all_as_csv(&mut self, ui: &mut egui::Ui, result: &UiQueryResult, indexes: &[usize]) {
        let header = result
            .columns
            .iter()
            .map(|column| result_grid_export::escape_delimited_field(&column.name, ','))
            .collect::<Vec<_>>()
            .join(",");
        let mut lines = vec![header];
        for &row_index in indexes {
            if let Some(row) = result.rows.get(row_index) {
                let row_values = (0..result.columns.len())
                    .map(|column_index| {
                        let cell = self
                            .copy_cell_value(result, row_index, column_index)
                            .unwrap_or_else(|| row.get(column_index).cloned().unwrap_or(UiCell::Null));
                        result_grid_export::format_cell_csv(&cell)
                    })
                    .collect::<Vec<_>>()
                    .join(",");
                lines.push(row_values);
            }
        }
        ui.output_mut(|output| output.copied_text = lines.join("\n"));
        self.feedback.copy_status = format!("{} rows copied as CSV", indexes.len());
    }

    pub(crate) fn copy_all_as_json(&mut self, ui: &mut egui::Ui, result: &UiQueryResult, indexes: &[usize]) {
        let mut rows = Vec::new();
        for &row_index in indexes {
            if let Some(row) = result.rows.get(row_index) {
                let mut map = serde_json::Map::new();
                for (col_idx, col) in result.columns.iter().enumerate() {
                    let cell = self
                        .copy_cell_value(result, row_index, col_idx)
                        .unwrap_or_else(|| row.get(col_idx).cloned().unwrap_or(UiCell::Null));
                    map.insert(col.name.clone(), result_grid_export::cell_to_json_value(&cell));
                }
                rows.push(serde_json::Value::Object(map));
            }
        }
        let json_text = serde_json::to_string_pretty(&serde_json::Value::Array(rows)).unwrap_or_default();
        ui.output_mut(|output| output.copied_text = json_text);
        self.feedback.copy_status = format!("{} rows copied as JSON", indexes.len());
    }

    pub(crate) fn copy_cell_value(
        &self,
        result: &UiQueryResult,
        row_index: usize,
        column_index: usize,
    ) -> Option<crate::UiCell> {
        let cell = result.rows.get(row_index).and_then(|row| row.get(column_index))?;
        if self.workspace.active_tab == WorkspaceTab::Table && self.table.state.table_view == TableView::Data {
            Some(
                self.staged_cell_value(result, row_index, column_index)
                    .unwrap_or_else(|| cell.clone()),
            )
        } else {
            Some(cell.clone())
        }
    }
}
