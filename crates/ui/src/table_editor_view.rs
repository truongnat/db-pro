use super::*;

impl DbProApp {
    pub(crate) fn draw_table_data(&mut self, ui: &mut egui::Ui, table_name: &str) {
        let Some(result) = self.table_data_result.clone() else {
            card_frame(self.theme).show(ui, |ui| {
                ui.vertical_centered(|ui| {
                    ui.add_space(28.0);
                    let failed = self.table_data_error.as_deref();
                    ui.label(icon_text(
                        if failed.is_some() {
                            Icon::TriangleAlert
                        } else {
                            Icon::LoaderCircle
                        },
                        "",
                        if failed.is_some() {
                            self.theme.warning
                        } else {
                            self.theme.accent
                        },
                    ));
                    ui.add_space(8.0);
                    ui.label(
                        RichText::new(if failed.is_some() {
                            format!("Data for {table_name} could not be loaded")
                        } else {
                            format!("Loading data for {table_name}…")
                        })
                        .strong()
                        .color(self.theme.text_primary),
                    );
                    if let Some(error) = failed {
                        ui.label(RichText::new(error).small().color(self.theme.text_secondary));
                        ui.add_space(12.0);
                        if secondary_button_with_icon(ui, Icon::RotateCcw, "Retry", self.theme).clicked() {
                            self.table_data_error = None;
                            self.request_table_data();
                        }
                    } else {
                        ui.label(
                            RichText::new("Rows will appear here with the shared result-grid controls.")
                                .small()
                                .color(self.theme.text_secondary),
                        );
                    }
                    ui.add_space(28.0);
                });
            });
            return;
        };

        let can_mutate = self.can_mutate_active_connection();
        let total_rows = self.table_data_total_rows.unwrap_or(result.row_count);
        let page = self.table_data_offset / TABLE_PAGE_SIZE + 1;
        let total_pages = total_rows.div_ceil(TABLE_PAGE_SIZE).max(1);
        let page_range = if total_rows > 0 {
            let start = self.table_data_offset + 1;
            let end = (self.table_data_offset + result.row_count).min(total_rows);
            format!("{start}–{end} of {total_rows}")
        } else {
            "0 rows".to_owned()
        };
        let has_next = self.table_data_offset.saturating_add(TABLE_PAGE_SIZE) < total_rows;
        let has_previous = self.table_data_offset > 0;
        toolbar_frame(self.theme).show(ui, |ui| {
            ui.horizontal_wrapped(|ui| {
                ui.label(icon_text(Icon::Table2, "DATA EDITOR", self.theme.text_primary));
                badge(ui, table_name, self.theme.accent_soft, self.theme.accent);
                ui.label(RichText::new(page_range).small().color(self.theme.text_muted));
                ui.separator();
                if compact_button_with_icon(ui, Icon::RotateCcw, "Refresh", self.theme)
                    .on_hover_text("Reload the current page")
                    .clicked()
                {
                    self.table_data_result = None;
                    self.table_data_total_rows = None;
                    self.table_data_error = None;
                    self.selected_cell = None;
                    self.selected_row = None;
                    self.data_delete_confirmation = false;
                    self.request_table_data();
                }
                if can_mutate {
                    if compact_button_with_icon(ui, Icon::Plus, "New row", self.theme).clicked() {
                        self.open_insert_row();
                    }
                    if compact_button_with_icon(ui, Icon::Pencil, "Edit cell", self.theme).clicked() {
                        if let Some((row_index, column_index)) = self.selected_cell {
                            if let Some(cell) = result.rows.get(row_index).and_then(|row| row.get(column_index)) {
                                self.begin_data_cell_edit(row_index, column_index, cell);
                            }
                        } else {
                            self.runtime_message = "Select a cell before editing".to_owned();
                        }
                    }
                    if compact_button_with_icon(ui, Icon::Trash2, "Delete row", self.theme).clicked() {
                        self.request_delete_selected_data_row(&result);
                    }
                    if self.data_delete_confirmation {
                        ui.colored_label(self.theme.warning, "Delete selected row?");
                        if danger_button(ui, "Confirm", self.theme).clicked() {
                            self.submit_delete_selected_data_row(&result);
                        }
                        if compact_button(ui, "Cancel", self.theme).clicked() {
                            self.data_delete_confirmation = false;
                        }
                    }
                } else if self.connected {
                    ui.label(RichText::new("Read-only connection").small().color(self.theme.warning));
                }
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    if compact_icon_button_enabled(ui, Icon::ChevronRight, has_next, self.theme)
                        .on_hover_text("Next page")
                        .clicked()
                    {
                        self.table_data_result = None;
                        self.table_data_error = None;
                        self.selected_cell = None;
                        self.selected_row = None;
                        self.data_delete_confirmation = false;
                        self.table_data_offset = self.table_data_offset.saturating_add(TABLE_PAGE_SIZE);
                        self.request_table_data();
                    }
                    if compact_icon_button_enabled(ui, Icon::ChevronLeft, has_previous, self.theme)
                        .on_hover_text("Previous page")
                        .clicked()
                    {
                        self.table_data_result = None;
                        self.table_data_error = None;
                        self.selected_cell = None;
                        self.selected_row = None;
                        self.data_delete_confirmation = false;
                        self.table_data_offset = self.table_data_offset.saturating_sub(TABLE_PAGE_SIZE);
                        self.request_table_data();
                    }
                    ui.label(
                        RichText::new(format!("Page {page} of {total_pages}"))
                            .small()
                            .color(self.theme.text_muted),
                    );
                });
            });
        });
        ui.add_space(8.0);
        let column_names = result
            .columns
            .iter()
            .map(|column| column.name.clone())
            .collect::<Vec<_>>();
        let requires_order = !self.active_driver().eq_ignore_ascii_case("sqlite");
        if !column_names.is_empty() {
            toolbar_frame(self.theme).show(ui, |ui| {
                ui.horizontal_wrapped(|ui| {
                    section_label(ui, "DATABASE FILTER", self.theme);
                    ui.label(RichText::new("Column").small().color(self.theme.text_muted));
                    egui::ComboBox::from_id_salt(("table-data-filter-column", table_name))
                        .selected_text(if self.table_data_filter_column.is_empty() {
                            "Column"
                        } else {
                            self.table_data_filter_column.as_str()
                        })
                        .width(110.0)
                        .show_ui(ui, |ui| {
                            for column in &column_names {
                                ui.selectable_value(
                                    &mut self.table_data_filter_column,
                                    column.clone(),
                                    column.as_str(),
                                );
                            }
                        });
                    input(ui, &mut self.table_data_filter_value, "contains…", 180.0, self.theme);
                    if compact_button_with_icon(ui, Icon::Filter, "Apply", self.theme).clicked() {
                        self.reload_table_data_from_start();
                    }
                    if compact_button_with_icon(ui, Icon::FilterX, "Clear", self.theme).clicked() {
                        self.table_data_filter_value.clear();
                        self.reload_table_data_from_start();
                    }
                    ui.separator();
                    section_label(ui, "ORDER BY", self.theme);
                    egui::ComboBox::from_id_salt(("table-data-sort-column", table_name))
                        .selected_text(self.table_data_sort_column.as_deref().unwrap_or("None"))
                        .width(110.0)
                        .show_ui(ui, |ui| {
                            if !requires_order {
                                ui.selectable_value(&mut self.table_data_sort_column, None, "None");
                            }
                            for column in &column_names {
                                ui.selectable_value(
                                    &mut self.table_data_sort_column,
                                    Some(column.clone()),
                                    column.as_str(),
                                );
                            }
                        });
                    if self.table_data_sort_column.is_some() {
                        let direction = if self.table_data_sort_desc { "DESC" } else { "ASC" };
                        if compact_button_with_icon(ui, Icon::ArrowDownUp, direction, self.theme).clicked() {
                            self.table_data_sort_desc = !self.table_data_sort_desc;
                            self.reload_table_data_from_start();
                        }
                    }
                });
            });
            ui.add_space(8.0);
        }
        let data_width = ui.max_rect().width();
        panel_frame(self.theme).show(ui, |ui| {
            ui.set_min_width((data_width - 24.0).max(0.0));
            self.draw_result_grid(ui, &result);
        });
    }

    fn open_insert_row(&mut self) {
        if !self.can_mutate_active_connection() {
            self.runtime_message = "Connect with write access to insert rows".to_owned();
            return;
        }
        let Some(info) = self.table_info.clone() else {
            self.runtime_message = "Table structure is still loading".to_owned();
            return;
        };
        self.insert_row_values = vec![String::new(); info.columns.len()];
        self.insert_row_error.clear();
        self.insert_row_open = true;
    }

    pub(crate) fn parse_insert_value(raw: &str, data_type: &str) -> Result<Option<UiCell>, String> {
        let value = raw.trim();
        if value.is_empty() {
            return Ok(None);
        }
        if value.eq_ignore_ascii_case("null") {
            return Ok(Some(UiCell::Null));
        }
        let normalized_type = data_type.to_ascii_lowercase();
        if normalized_type.contains("bool") {
            return match value.to_ascii_lowercase().as_str() {
                "true" | "1" | "yes" => Ok(Some(UiCell::Boolean(true))),
                "false" | "0" | "no" => Ok(Some(UiCell::Boolean(false))),
                _ => Err("expected true or false".to_owned()),
            };
        }
        if normalized_type.contains("int") || normalized_type.contains("serial") {
            return value
                .parse::<i64>()
                .map(|number| Some(UiCell::Number(number.to_string())))
                .map_err(|_| format!("{value} is not a valid integer"));
        }
        if normalized_type.contains("real") || normalized_type.contains("float") || normalized_type.contains("double") {
            return value
                .parse::<f64>()
                .map(|number| Some(UiCell::Number(number.to_string())))
                .map_err(|_| format!("{value} is not a valid floating-point number"));
        }
        if Self::is_decimal_type(&normalized_type) {
            return Self::parse_decimal_value(value, data_type).map(Some);
        }
        if normalized_type.contains("json") {
            return serde_json::from_str::<serde_json::Value>(value)
                .map(|_| Some(UiCell::Json(value.to_owned())))
                .map_err(|_| "expected valid JSON".to_owned());
        }
        Ok(Some(UiCell::Text(value.to_owned())))
    }

    pub(crate) fn parse_update_value(raw: &str, data_type: &str) -> Result<UiCell, String> {
        if raw.trim().is_empty() {
            if Self::is_decimal_type(&data_type.to_ascii_lowercase()) {
                return Err("enter a decimal value or the literal NULL".to_owned());
            }
            return Ok(UiCell::Text(String::new()));
        }
        Self::parse_insert_value(raw, data_type).map(|value| value.unwrap_or_else(|| UiCell::Text(String::new())))
    }

    fn is_decimal_type(normalized_type: &str) -> bool {
        normalized_type
            .split('(')
            .next()
            .is_some_and(|name| matches!(name.trim(), "numeric" | "decimal"))
    }

    fn decimal_constraints(data_type: &str) -> Option<(u64, i64)> {
        let normalized_type = data_type.to_ascii_lowercase();
        if !Self::is_decimal_type(&normalized_type) {
            return None;
        }
        let arguments = normalized_type.split_once('(')?.1.split_once(')')?.0;
        let mut parts = arguments.split(',').map(str::trim);
        let precision = parts.next()?.parse::<u64>().ok()?;
        let scale = parts.next().and_then(|part| part.parse::<i64>().ok()).unwrap_or(0);
        Some((precision, scale))
    }

    fn parse_decimal_value(value: &str, data_type: &str) -> Result<UiCell, String> {
        let decimal = value
            .parse::<BigDecimal>()
            .map_err(|_| format!("{value} is not a valid exact decimal"))?;
        let actual_scale = decimal.fractional_digit_count();
        if let Some((precision, declared_scale)) = Self::decimal_constraints(data_type) {
            if actual_scale > declared_scale {
                return Err(format!("{value} has more than {declared_scale} fractional digits"));
            }
            let effective_precision = if actual_scale < 0 {
                decimal.digits().saturating_add((-actual_scale) as u64)
            } else {
                decimal.digits()
            };
            if effective_precision > precision {
                return Err(format!("{value} exceeds NUMERIC precision {precision}"));
            }
        }
        Ok(UiCell::Text(value.to_owned()))
    }

    fn submit_insert_row(&mut self) {
        let Some(table) = self.selected_table.clone() else {
            self.insert_row_error = "Select a table before inserting a row".to_owned();
            return;
        };
        let Some(info) = self.table_info.clone() else {
            self.insert_row_error = "Table structure is still loading".to_owned();
            return;
        };
        let mut columns = Vec::new();
        let mut values = Vec::new();
        for (column, raw) in info.columns.iter().zip(&self.insert_row_values) {
            match Self::parse_insert_value(raw, &column.data_type) {
                Ok(Some(value)) => {
                    columns.push(column.name.clone());
                    values.push(value);
                }
                Ok(None) => {}
                Err(error) => {
                    self.insert_row_error = format!("{}: {error}", column.name);
                    return;
                }
            }
        }
        if columns.is_empty() {
            self.insert_row_error = "Enter at least one value; leave defaulted columns empty".to_owned();
            return;
        }
        let Some(connection) = self.active_connection().cloned() else {
            self.insert_row_error = "Connect to a database before inserting a row".to_owned();
            return;
        };
        let request_id = self.task_bridge.next_request_id();
        let _ = self.task_bridge.send(UiCommand::InsertTableRow {
            request_id,
            connection_id: connection.id,
            schema: self.active_schema().to_owned(),
            table,
            columns,
            values,
        });
        self.table_mutation_request = Some(request_id);
        self.insert_row_open = false;
        self.insert_row_error.clear();
        self.runtime_message = "Inserting row…".to_owned();
    }

    pub(super) fn draw_insert_row_dialog(&mut self, ctx: &egui::Context) {
        let Some(info) = self.table_info.clone() else {
            self.insert_row_open = false;
            return;
        };
        if self.insert_row_values.len() != info.columns.len() {
            self.insert_row_values = vec![String::new(); info.columns.len()];
        }
        let mut open = self.insert_row_open;
        let mut submit = false;
        let mut cancel = false;
        egui::Window::new("Insert row")
            .open(&mut open)
            .collapsible(false)
            .resizable(false)
            .default_width(460.0)
            .show(ctx, |ui| {
                ui.label(
                    RichText::new(format!("Add a row to {}.{}", info.schema, info.name))
                        .color(self.theme.text_secondary),
                );
                ui.label(
                    RichText::new("Empty fields use the database default. Type NULL for a null value.")
                        .small()
                        .color(self.theme.text_muted),
                );
                ui.add_space(10.0);
                for (index, column) in info.columns.iter().enumerate() {
                    ui.horizontal(|ui| {
                        ui.add_sized(
                            [108.0, 24.0],
                            egui::Label::new(
                                RichText::new(format!("{} · {}", column.name, column.data_type))
                                    .small()
                                    .strong()
                                    .color(self.theme.text_secondary),
                            ),
                        );
                        input(
                            ui,
                            &mut self.insert_row_values[index],
                            &column.data_type,
                            300.0,
                            self.theme,
                        );
                    });
                }
                if !self.insert_row_error.is_empty() {
                    ui.add_space(6.0);
                    ui.colored_label(self.theme.danger, self.insert_row_error.as_str());
                }
                ui.separator();
                ui.horizontal(|ui| {
                    if primary_button_with_icon(ui, Icon::Plus, "Insert row", self.theme).clicked() {
                        submit = true;
                    }
                    if ghost_button(ui, "Cancel", self.theme).clicked() {
                        cancel = true;
                    }
                });
            });
        if submit {
            self.submit_insert_row();
        }
        if cancel || !open {
            self.insert_row_open = false;
            self.insert_row_error.clear();
        }
    }

    pub(crate) fn submit_ddl(&mut self) {
        if !self.can_mutate_active_connection() {
            self.runtime_message = "Connect with write access to execute DDL".to_owned();
            return;
        }
        if self.ddl_execution_request.is_some() {
            return;
        }
        let Some(sql) = self.table_ddl.clone() else {
            self.runtime_message = "Load the table DDL before executing it".to_owned();
            return;
        };
        if sql.trim().is_empty() {
            self.runtime_message = "DDL cannot be empty".to_owned();
            return;
        }
        let Some(connection) = self.active_connection().cloned() else {
            self.runtime_message = "Connect to a database before executing DDL".to_owned();
            return;
        };
        let request_id = self.task_bridge.next_request_id();
        let _ = self.task_bridge.send(UiCommand::ExecuteDdl {
            request_id,
            connection_id: connection.id,
            sql,
        });
        self.ddl_execution_request = Some(request_id);
        self.ddl_execute_confirmation = false;
        self.runtime_message = "Executing DDL…".to_owned();
    }

    pub(crate) fn draw_table_ddl(&mut self, ui: &mut egui::Ui, table_name: &str) {
        let Some(mut ddl) = self.table_ddl.clone() else {
            card_frame(self.theme).show(ui, |ui| {
                ui.vertical_centered(|ui| {
                    ui.add_space(28.0);
                    let failed = self.table_ddl_error.as_deref();
                    ui.label(icon_text(
                        if failed.is_some() {
                            Icon::TriangleAlert
                        } else {
                            Icon::Code2
                        },
                        "",
                        if failed.is_some() {
                            self.theme.warning
                        } else {
                            self.theme.accent
                        },
                    ));
                    ui.add_space(8.0);
                    ui.label(
                        RichText::new(if failed.is_some() {
                            format!("DDL for {table_name} could not be loaded")
                        } else {
                            format!("Loading DDL for {table_name}…")
                        })
                        .strong()
                        .color(self.theme.text_primary),
                    );
                    if let Some(error) = failed {
                        ui.label(RichText::new(error).small().color(self.theme.text_secondary));
                    }
                    ui.add_space(28.0);
                });
            });
            return;
        };

        let writable = self.can_mutate_active_connection();
        let mut request_execution = false;
        card_frame(self.theme).show(ui, |ui| {
            ui.horizontal_wrapped(|ui| {
                section_label(ui, "CREATE SCRIPT", self.theme);
                ui.label(
                    RichText::new(if writable {
                        "Editable preview · execution is confirmation-gated"
                    } else {
                        "Read-only preview"
                    })
                    .small()
                    .color(self.theme.text_muted),
                );
                if writable
                    && self.ddl_execution_request.is_none()
                    && primary_button_with_icon(ui, Icon::Play, "Apply DDL", self.theme).clicked()
                {
                    request_execution = true;
                }
            });
            ui.add_space(8.0);
            if let Some(error) = self.table_ddl_error.as_deref() {
                ui.label(
                    RichText::new(format!("DDL execution failed · {error}"))
                        .small()
                        .color(self.theme.danger),
                );
                ui.add_space(6.0);
            }
            editor_frame(self.theme).show(ui, |ui| {
                let response = ui.add(
                    TextEdit::multiline(&mut ddl)
                        .font(FontId::monospace(13.0))
                        .desired_width(ui.available_width())
                        .desired_rows(18)
                        .interactive(writable),
                );
                if response.changed() {
                    self.table_ddl_error = None;
                }
            });
        });
        let impact = ddl_impact_summary(&ddl, table_name);
        self.table_ddl = Some(ddl);
        if request_execution {
            self.ddl_execute_confirmation = true;
        }
        if self.ddl_execute_confirmation {
            let mut execute = false;
            let mut cancel = false;
            card_frame(self.theme).show(ui, |ui| {
                ui.horizontal_wrapped(|ui| {
                    ui.label(icon_text(
                        Icon::TriangleAlert,
                        "Review DDL before applying",
                        self.theme.warning,
                    ));
                    ui.label(
                        RichText::new("This changes the connected database and refreshes the Explorer.")
                            .small()
                            .color(self.theme.text_secondary),
                    );
                    ui.label(RichText::new(impact).small().color(self.theme.warning));
                    if primary_button_with_icon(ui, Icon::Check, "Execute", self.theme).clicked() {
                        execute = true;
                    }
                    if ghost_button(ui, "Cancel", self.theme).clicked() {
                        cancel = true;
                    }
                });
            });
            if execute {
                self.submit_ddl();
            }
            if cancel {
                self.ddl_execute_confirmation = false;
            }
        }
    }
    pub(crate) fn request_table_info(&mut self) {
        let (Some(connection_id), Some(table)) = (self.active_connection_id.clone(), self.selected_table.clone())
        else {
            return;
        };
        let request_id = self.task_bridge.next_request_id();
        self.table_info_request = Some(request_id);
        let _ = self.task_bridge.send(UiCommand::LoadTableInfo {
            request_id,
            connection_id,
            schema: self.active_schema().to_owned(),
            table,
        });
        self.runtime_message = "Loading table structure…".to_owned();
    }

    pub(crate) fn can_mutate_active_connection(&self) -> bool {
        self.connected && self.active_connection().is_some_and(|connection| !connection.readonly)
    }

    pub(crate) fn row_identity(
        result: &UiQueryResult,
        info: &UiTableInfo,
        row_index: usize,
    ) -> Result<(Vec<String>, Vec<UiCell>), String> {
        let Some(primary_key) = info.primary_key.as_ref() else {
            return Err("This table has no primary key for safe row editing".to_owned());
        };
        let mut pk_values = Vec::with_capacity(primary_key.len());
        for pk_column in primary_key {
            let Some(pk_index) = result.columns.iter().position(|item| item.name == *pk_column) else {
                return Err(format!(
                    "The primary-key column {pk_column} is not present in this result"
                ));
            };
            let Some(pk_cell) = result.rows.get(row_index).and_then(|row| row.get(pk_index)) else {
                return Err("The selected row is no longer available".to_owned());
            };
            if matches!(pk_cell, UiCell::Null) {
                return Err(format!("A NULL primary key ({pk_column}) cannot identify a row"));
            }
            pk_values.push(pk_cell.clone());
        }
        Ok((primary_key.clone(), pk_values))
    }

    pub(crate) fn begin_data_cell_edit(&mut self, row_index: usize, column_index: usize, cell: &UiCell) {
        if !self.can_mutate_active_connection() {
            self.runtime_message = "Connect with write access to edit rows".to_owned();
            return;
        }
        self.selected_cell = Some((row_index, column_index));
        self.selected_row = Some(row_index);
        self.data_editing_cell = Some((row_index, column_index));
        self.data_edit_value = match cell {
            UiCell::Null => String::new(),
            _ => crate::cell_text(cell),
        };
        self.copy_status.clear();
    }

    pub(crate) fn submit_data_cell_edit(&mut self, result: &UiQueryResult, row_index: usize, column_index: usize) {
        let Some(info) = self.table_info.clone() else {
            self.runtime_message = "Table structure is still loading".to_owned();
            return;
        };
        let Some(table) = self.selected_table.clone() else {
            self.data_editing_cell = None;
            return;
        };
        let Some(column) = result.columns.get(column_index).map(|column| column.name.clone()) else {
            self.data_editing_cell = None;
            return;
        };
        let Some(column_info) = info.columns.iter().find(|item| item.name == column) else {
            self.runtime_message = "The selected column is not present in the table metadata".to_owned();
            self.data_editing_cell = None;
            return;
        };
        let value = match Self::parse_update_value(&self.data_edit_value, &column_info.data_type) {
            Ok(value) => value,
            Err(error) => {
                self.runtime_message = format!("{}: {error}", column_info.name);
                self.data_editing_cell = None;
                return;
            }
        };
        let (pk_columns, pk_values) = match Self::row_identity(result, &info, row_index) {
            Ok(identity) => identity,
            Err(error) => {
                self.runtime_message = error;
                self.data_editing_cell = None;
                return;
            }
        };
        let Some(connection) = self.active_connection().cloned() else {
            self.data_editing_cell = None;
            return;
        };
        let request_id = self.task_bridge.next_request_id();
        let _ = self.task_bridge.send(UiCommand::UpdateTableRow {
            request_id,
            connection_id: connection.id,
            schema: self.active_schema().to_owned(),
            table,
            column,
            value,
            pk_columns,
            pk_values,
        });
        self.table_mutation_request = Some(request_id);
        self.data_editing_cell = None;
        self.runtime_message = "Saving cell…".to_owned();
    }

    fn request_delete_selected_data_row(&mut self, result: &UiQueryResult) {
        if !self.can_mutate_active_connection() {
            self.runtime_message = "Connect with write access to delete rows".to_owned();
            return;
        }
        let Some(row_index) = self.selected_row else {
            self.runtime_message = "Select a row before deleting".to_owned();
            return;
        };
        let Some(info) = self.table_info.clone() else {
            self.runtime_message = "Table structure is still loading".to_owned();
            return;
        };
        if let Err(error) = Self::row_identity(result, &info, row_index) {
            self.runtime_message = error;
            return;
        }
        self.data_delete_confirmation = true;
        self.data_editing_cell = None;
        self.data_edit_value.clear();
    }

    fn submit_delete_selected_data_row(&mut self, result: &UiQueryResult) {
        let Some(row_index) = self.selected_row else {
            self.data_delete_confirmation = false;
            self.runtime_message = "Select a row before deleting".to_owned();
            return;
        };
        let Some(info) = self.table_info.clone() else {
            self.data_delete_confirmation = false;
            self.runtime_message = "Table structure is still loading".to_owned();
            return;
        };
        let (pk_columns, pk_values) = match Self::row_identity(result, &info, row_index) {
            Ok(identity) => identity,
            Err(error) => {
                self.data_delete_confirmation = false;
                self.runtime_message = error;
                return;
            }
        };
        let Some(connection) = self.active_connection().cloned() else {
            self.data_delete_confirmation = false;
            self.runtime_message = "Connect to a database before deleting a row".to_owned();
            return;
        };
        let Some(table) = self.selected_table.clone() else {
            self.data_delete_confirmation = false;
            return;
        };
        let request_id = self.task_bridge.next_request_id();
        let _ = self.task_bridge.send(UiCommand::DeleteTableRow {
            request_id,
            connection_id: connection.id,
            schema: self.active_schema().to_owned(),
            table,
            pk_columns,
            pk_values,
        });
        self.table_mutation_request = Some(request_id);
        self.data_delete_confirmation = false;
        self.selected_cell = None;
        self.selected_row = None;
        self.runtime_message = "Deleting row…".to_owned();
    }

    pub(crate) fn request_table_ddl(&mut self) {
        let (Some(connection_id), Some(table)) = (self.active_connection_id.clone(), self.selected_table.clone())
        else {
            return;
        };
        let request_id = self.task_bridge.next_request_id();
        self.table_ddl_request = Some(request_id);
        let _ = self.task_bridge.send(UiCommand::LoadTableDdl {
            request_id,
            connection_id,
            schema: self.active_schema().to_owned(),
            table,
        });
        self.runtime_message = "Loading table DDL…".to_owned();
    }

    pub(crate) fn request_table_data(&mut self) {
        let Some(connection_id) = self.active_connection_id.clone() else {
            return;
        };
        let table = self
            .selected_table
            .clone()
            .or_else(|| match self.selected_schema_object.as_ref() {
                Some(SchemaObjectSelection::View(name)) => Some(name.clone()),
                _ => None,
            });
        let Some(table) = table else {
            return;
        };
        let request_id = self.task_bridge.next_request_id();
        self.table_data_request = Some(request_id);
        let filter = (!self.table_data_filter_value.trim().is_empty()
            && !self.table_data_filter_column.trim().is_empty())
        .then(|| UiTableDataFilter {
            column: self.table_data_filter_column.clone(),
            value: self.table_data_filter_value.trim().to_owned(),
        });
        let sort = self.table_data_sort_column.clone().map(|column| UiTableDataSort {
            column,
            descending: self.table_data_sort_desc,
        });
        let _ = self.task_bridge.send(UiCommand::LoadTableData {
            request_id,
            connection_id,
            schema: self.active_schema().to_owned(),
            table,
            limit: TABLE_PAGE_SIZE,
            offset: self.table_data_offset,
            filter,
            sort,
        });
        self.runtime_message = "Loading table data…".to_owned();
    }

    pub(crate) fn reload_table_data_from_start(&mut self) {
        self.table_data_offset = 0;
        self.table_data_result = None;
        self.table_data_total_rows = None;
        self.table_data_error = None;
        self.grid_filter.clear();
        self.grid_sort_column = None;
        self.grid_sort_desc = false;
        self.request_table_data();
    }
}
