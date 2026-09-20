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
            .map(|cell| Self::format_cell_delimited(&cell, '\t'))
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
            .map(|column| Self::escape_delimited_field(&column.name, '\t'))
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

    pub(super) fn copy_selected_rows_as_insert(&mut self, ui: &mut egui::Ui, result: &UiQueryResult) {
        let indexes = self.table.data.selected_row_indexes();
        if indexes.is_empty() {
            self.feedback.copy_status = "Select one or more rows first".to_owned();
            return;
        }
        let table = self.schema.explorer.selected_table.as_deref().unwrap_or("table_name");
        let target =
            if self.workspace.active_tab == WorkspaceTab::Table && self.table.state.table_view == TableView::Data {
                format!(
                    "{}.{}",
                    Self::quote_sql_identifier(self.active_schema()),
                    Self::quote_sql_identifier(table)
                )
            } else {
                Self::quote_sql_identifier(table)
            };
        let columns = result
            .columns
            .iter()
            .map(|column| Self::quote_sql_identifier(&column.name))
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
                        Self::cell_sql_literal(&cell)
                    })
                    .collect::<Vec<_>>()
                    .join(", ");
                format!("INSERT INTO {target} ({columns}) VALUES ({values});")
            })
            .collect::<Vec<_>>();
        ui.output_mut(|output| output.copied_text = statements.join("\n"));
        self.feedback.copy_status = format!("{} INSERT statements copied", statements.len());
    }

    pub(super) fn quote_sql_identifier(identifier: &str) -> String {
        format!("\"{}\"", identifier.replace('"', "\"\""))
    }

    pub(super) fn cell_sql_literal(cell: &UiCell) -> String {
        match cell {
            UiCell::Null => "NULL".to_owned(),
            UiCell::Boolean(value) => value.to_string().to_uppercase(),
            UiCell::Number(value) if value.parse::<f64>().is_ok() => value.clone(),
            UiCell::Json(value) => format!("'{}'", value.replace('\'', "''")),
            UiCell::Bytes(value) => format!("'{}'", value.replace('\'', "''")),
            UiCell::Number(value) | UiCell::Text(value) => format!("'{}'", value.replace('\'', "''")),
        }
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
            map.insert(col.name.clone(), Self::cell_to_json_value(&cell));
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
            .map(|c| {
                if c.name.contains(',') || c.name.contains('"') {
                    format!("\"{}\"", c.name.replace('"', "\"\""))
                } else {
                    c.name.clone()
                }
            })
            .collect::<Vec<_>>()
            .join(",");
        let row_values = (0..result.columns.len())
            .map(|col_idx| {
                let cell = self
                    .copy_cell_value(result, row_index, col_idx)
                    .unwrap_or_else(|| row.get(col_idx).cloned().unwrap_or(UiCell::Null));
                Self::format_cell_csv(&cell)
            })
            .collect::<Vec<_>>()
            .join(",");
        let csv_text = format!("{header}\n{row_values}");
        ui.output_mut(|output| output.copied_text = csv_text);
        self.feedback.copy_status = "Row copied as CSV".to_owned();
    }

    pub(crate) fn copy_all_as_csv(&mut self, ui: &mut egui::Ui, result: &UiQueryResult, indexes: &[usize]) {
        let header = result
            .columns
            .iter()
            .map(|c| {
                if c.name.contains(',') || c.name.contains('"') {
                    format!("\"{}\"", c.name.replace('"', "\"\""))
                } else {
                    c.name.clone()
                }
            })
            .collect::<Vec<_>>()
            .join(",");
        let mut lines = vec![header];
        for &row_index in indexes {
            if let Some(row) = result.rows.get(row_index) {
                let row_values = (0..result.columns.len())
                    .map(|col_idx| {
                        let cell = self
                            .copy_cell_value(result, row_index, col_idx)
                            .unwrap_or_else(|| row.get(col_idx).cloned().unwrap_or(UiCell::Null));
                        Self::format_cell_csv(&cell)
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
        let mut rows_arr = Vec::new();
        for &row_index in indexes {
            if let Some(row) = result.rows.get(row_index) {
                let mut map = serde_json::Map::new();
                for (col_idx, col) in result.columns.iter().enumerate() {
                    let cell = self
                        .copy_cell_value(result, row_index, col_idx)
                        .unwrap_or_else(|| row.get(col_idx).cloned().unwrap_or(UiCell::Null));
                    map.insert(col.name.clone(), Self::cell_to_json_value(&cell));
                }
                rows_arr.push(serde_json::Value::Object(map));
            }
        }
        let json_text = serde_json::to_string_pretty(&serde_json::Value::Array(rows_arr)).unwrap_or_default();
        ui.output_mut(|output| output.copied_text = json_text);
        self.feedback.copy_status = format!("{} rows copied as JSON", indexes.len());
    }

    /// Quote a value so that the delimiter, quotes and line breaks inside it cannot
    /// change the shape of the copy/export.
    pub(crate) fn escape_delimited_field(value: &str, delimiter: char) -> String {
        if value.contains(delimiter) || value.contains('"') || value.contains('\n') || value.contains('\r') {
            format!("\"{}\"", value.replace('"', "\"\""))
        } else {
            value.to_owned()
        }
    }

    /// One `UiCell` as a single delimited field. Numbers are emitted verbatim so the
    /// exact digits survive; `Null` is an empty field, which is how both CSV and TSV
    /// represent a missing value.
    pub(crate) fn format_cell_delimited(cell: &crate::UiCell, delimiter: char) -> String {
        match cell {
            crate::UiCell::Null => String::new(),
            crate::UiCell::Boolean(b) => b.to_string(),
            crate::UiCell::Number(n) => n.clone(),
            crate::UiCell::Text(t) => Self::escape_delimited_field(t, delimiter),
            crate::UiCell::Json(j) => Self::escape_delimited_field(j, delimiter),
            crate::UiCell::Bytes(b) => Self::escape_delimited_field(b, delimiter),
        }
    }

    pub(crate) fn format_cell_csv(cell: &crate::UiCell) -> String {
        Self::format_cell_delimited(cell, ',')
    }

    /// The exact text written by the query-view export for one delimiter.
    pub(crate) fn format_result_delimited(result: &UiQueryResult, delimiter: &str) -> String {
        let separator = delimiter.chars().next().unwrap_or(',');
        let mut output = result
            .columns
            .iter()
            .map(|column| Self::escape_delimited_field(&column.name, separator))
            .collect::<Vec<_>>()
            .join(delimiter);
        output.push('\n');
        for row in &result.rows {
            output.push_str(
                &row.iter()
                    .map(|cell| Self::format_cell_delimited(cell, separator))
                    .collect::<Vec<_>>()
                    .join(delimiter),
            );
            output.push('\n');
        }
        output
    }

    /// SQL INSERT statements for result rows (#220).
    pub(crate) fn format_result_sql_insert(result: &UiQueryResult, table: &str) -> String {
        if result.columns.is_empty() {
            return String::new();
        }
        let cols = result
            .columns
            .iter()
            .map(|c| format!("\"{}\"", c.name.replace('"', "\"\"")))
            .collect::<Vec<_>>()
            .join(", ");
        let mut out = String::new();
        const BATCH: usize = 50;
        for chunk in result.rows.chunks(BATCH) {
            out.push_str(&format!("INSERT INTO \"{table}\" ({cols}) VALUES\n"));
            for (i, row) in chunk.iter().enumerate() {
                let values = row
                    .iter()
                    .map(Self::format_cell_sql_literal)
                    .collect::<Vec<_>>()
                    .join(", ");
                out.push_str("  (");
                out.push_str(&values);
                out.push(')');
                if i + 1 < chunk.len() {
                    out.push_str(",\n");
                } else {
                    out.push_str(";\n");
                }
            }
            out.push('\n');
        }
        out
    }

    /// PostgreSQL COPY text script for result rows (#220).
    pub(crate) fn format_result_copy(result: &UiQueryResult, table: &str) -> String {
        let cols = result
            .columns
            .iter()
            .map(|c| format!("\"{}\"", c.name.replace('"', "\"\"")))
            .collect::<Vec<_>>()
            .join(", ");
        let mut out = format!("COPY \"{table}\" ({cols}) FROM stdin;\n");
        for row in &result.rows {
            let line = row
                .iter()
                .map(Self::format_cell_copy_field)
                .collect::<Vec<_>>()
                .join("\t");
            out.push_str(&line);
            out.push('\n');
        }
        out.push_str("\\.\n");
        out
    }

    pub(super) fn format_cell_sql_literal(cell: &crate::UiCell) -> String {
        match cell {
            crate::UiCell::Null => "NULL".into(),
            crate::UiCell::Boolean(b) => if *b { "TRUE" } else { "FALSE" }.into(),
            crate::UiCell::Number(n) => n.clone(),
            crate::UiCell::Text(t) | crate::UiCell::Json(t) | crate::UiCell::Bytes(t) => {
                format!("'{}'", t.replace('\'', "''"))
            }
        }
    }

    pub(super) fn format_cell_copy_field(cell: &crate::UiCell) -> String {
        match cell {
            crate::UiCell::Null => "\\N".into(),
            crate::UiCell::Boolean(b) => if *b { "t" } else { "f" }.into(),
            crate::UiCell::Number(n) => n.clone(),
            crate::UiCell::Text(t) | crate::UiCell::Json(t) | crate::UiCell::Bytes(t) => t
                .replace('\\', "\\\\")
                .replace('\t', "\\t")
                .replace('\n', "\\n")
                .replace('\r', "\\r"),
        }
    }

    /// Copy-as-JSON must not lose a value. An integer inside the range every JSON
    /// reader represents exactly is written as a JSON integer; any other number is
    /// written as a JSON number only when the decimal text is exactly the shortest form
    /// of the parsed `f64`. Everything else (an integer past 2^53, an exact decimal with
    /// trailing zeroes) keeps its exact digits as a JSON string — the same choice the
    /// provider-value contract already makes for decimals — instead of being rounded
    /// through `f64`.
    pub(crate) fn cell_to_json_value(cell: &crate::UiCell) -> serde_json::Value {
        const JSON_EXACT_INTEGER_LIMIT: i64 = 1 << 53;
        match cell {
            crate::UiCell::Null => serde_json::Value::Null,
            crate::UiCell::Boolean(b) => serde_json::Value::Bool(*b),
            crate::UiCell::Number(n) => match n.parse::<i64>() {
                Ok(i) if (-JSON_EXACT_INTEGER_LIMIT..=JSON_EXACT_INTEGER_LIMIT).contains(&i) => {
                    serde_json::Value::Number(i.into())
                }
                _ => match n.parse::<f64>() {
                    Ok(f) if f.is_finite() && f.to_string() == *n => serde_json::Number::from_f64(f)
                        .map(serde_json::Value::Number)
                        .unwrap_or_else(|| serde_json::Value::String(n.clone())),
                    _ => serde_json::Value::String(n.clone()),
                },
            },
            crate::UiCell::Text(t) => serde_json::Value::String(t.clone()),
            crate::UiCell::Json(j) => serde_json::from_str(j).unwrap_or_else(|_| serde_json::Value::String(j.clone())),
            crate::UiCell::Bytes(b) => serde_json::Value::String(b.clone()),
        }
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
