use super::*;

impl DbProApp {
    pub(super) fn draw_query(&mut self, ui: &mut egui::Ui) {
        self.refresh_diagnostics();
        let modifier = Self::primary_modifier_label();
        ui.horizontal_wrapped(|ui| {
            ui.label(RichText::new("Query").strong().color(self.theme.text_primary));
            ui.label(icon_text(Icon::ChevronRight, "", self.theme.text_muted));
            ui.label(RichText::new(self.active_connection_name()).color(self.theme.accent));
            let running = self.next_query_request.is_some();
            let run_button = if running {
                secondary_button_with_icon(ui, Icon::Square, "Stop  Esc", self.theme)
            } else {
                primary_button_with_icon(ui, Icon::Play, &format!("Run  {modifier}↵"), self.theme)
            };
            if run_button.clicked() {
                if let Some(request_id) = self.next_query_request {
                    self.cancel_query(request_id);
                } else {
                    self.dispatch_query();
                }
            }
            if secondary_button_with_icon(ui, Icon::WandSparkles, "Format", self.theme).clicked() {
                self.query_text = Self::format_sql(&self.query_text);
            }
            input(ui, &mut self.query_folder, "folder (optional)", 150.0, self.theme);
            if ghost_button(ui, "New folder", self.theme).clicked() {
                if let Some(connection) = self.connections.first() {
                    if !self.query_folder.trim().is_empty() {
                        let request_id = self.task_bridge.next_request_id();
                        let _ = self.task_bridge.send(UiCommand::CreateQueryFolder {
                            request_id,
                            connection_id: connection.id.clone(),
                            name: self.query_folder.trim().to_owned(),
                        });
                        self.runtime_message = "Creating query folder…".to_owned();
                    }
                }
            }
            if secondary_button_with_icon(ui, Icon::Save, "Save", self.theme).clicked() {
                if let Some(connection) = self.connections.first() {
                    let request_id = self.task_bridge.next_request_id();
                    let name = self
                        .query_documents
                        .get(self.active_query_document)
                        .map(|document| document.title.clone())
                        .unwrap_or_else(|| "Saved query".to_owned());
                    let _ = self.task_bridge.send(UiCommand::SaveQuery {
                        request_id,
                        connection_id: connection.id.clone(),
                        name,
                        sql: self.query_text.clone(),
                        folder: (!self.query_folder.trim().is_empty()).then(|| self.query_folder.trim().to_owned()),
                    });
                    self.runtime_message = "Saving query…".to_owned();
                }
            }
            if ghost_button(
                ui,
                if self.selected_query.is_empty() {
                    "Run statement"
                } else {
                    "Run selection"
                },
                self.theme,
            )
            .clicked()
            {
                if self.selected_query.is_empty() {
                    let statement = self.query_text.split(';').next().unwrap_or_default().trim().to_owned();
                    if !statement.is_empty() {
                        self.query_text = statement;
                        self.dispatch_query();
                    }
                } else {
                    self.dispatch_query();
                }
            }
        });
        ui.add_space(8.0);
        ui.horizontal_wrapped(|ui| {
            if ghost_button_with_icon(ui, Icon::Search, "Search", self.theme).clicked() {
                self.editor_search_open = !self.editor_search_open;
            }
            if ghost_button(ui, "A−", self.theme).clicked() {
                self.editor_font_size = (self.editor_font_size - 1.0).max(10.0);
            }
            if ghost_button(ui, "A+", self.theme).clicked() {
                self.editor_font_size = (self.editor_font_size + 1.0).min(24.0);
            }
            if ghost_button(ui, "Completion", self.theme).clicked() {
                self.completion_open = !self.completion_open;
            }
            if ghost_button(ui, "Snippets", self.theme).clicked() {
                self.snippets_open = !self.snippets_open;
            }
            ui.label(
                RichText::new(format!("{} px", self.editor_font_size))
                    .small()
                    .color(self.theme.text_muted),
            );
            if self.editor_search_open {
                input(ui, &mut self.editor_search, "Find in SQL…", 240.0, self.theme);
                if !self.editor_search.is_empty() {
                    let matches = self.query_text.matches(&self.editor_search).count();
                    ui.label(
                        RichText::new(format!("{matches} matches"))
                            .small()
                            .color(self.theme.text_muted),
                    );
                }
            }
        });
        let editor_width = ui.max_rect().width();
        ui.allocate_ui_with_layout(egui::vec2(editor_width, 300.0), Layout::top_down(Align::Min), |ui| {
            editor_frame(self.theme).show(ui, |ui| {
                ui.set_min_width((editor_width - 24.0).max(0.0));
                ui.horizontal_top(|ui| {
                    let line_count = self.query_text.lines().count().max(1);
                    ui.vertical(|ui| {
                        for line in 1..=line_count {
                            ui.label(
                                RichText::new(format!("{line:>3}"))
                                    .monospace()
                                    .color(self.theme.text_muted),
                            );
                        }
                    });
                    ui.separator();
                    let editor_text_width = (editor_width - 72.0).max(280.0);
                    let editor_size = egui::vec2(editor_text_width, 260.0);
                    let theme = self.theme;
                    let output = ui.allocate_ui(editor_size, |ui| {
                        TextEdit::multiline(&mut self.query_text)
                            .font(egui::TextStyle::Monospace)
                            .desired_width(f32::INFINITY)
                            .min_size(editor_size)
                            .desired_rows(14)
                            .layouter(&mut |ui, text, wrap_width| Self::sql_layouter(ui, text, wrap_width, theme))
                            .lock_focus(true)
                            .show(ui)
                    });
                    if let Some(cursor_range) = output.inner.cursor_range {
                        let range = cursor_range.as_sorted_char_range();
                        if range.start < range.end && range.end <= self.query_text.len() {
                            self.selected_query = self
                                .query_text
                                .chars()
                                .skip(range.start)
                                .take(range.end - range.start)
                                .collect();
                        } else {
                            self.selected_query.clear();
                        }
                    }
                });
            });
        });
        if self.completion_open {
            card_frame(self.theme).show(ui, |ui| {
                ui.label(RichText::new("SQL completion").strong());
                let is_sqlite = self
                    .connections
                    .first()
                    .map(|connection| connection.driver.eq_ignore_ascii_case("sqlite"))
                    .unwrap_or(false);
                let mut candidates = vec![
                    "SELECT".to_owned(),
                    "FROM".to_owned(),
                    "WHERE".to_owned(),
                    "JOIN".to_owned(),
                    "GROUP BY".to_owned(),
                    "ORDER BY".to_owned(),
                    "LIMIT".to_owned(),
                    "COUNT(*)".to_owned(),
                ];
                if is_sqlite {
                    candidates.extend(["GLOB", "strftime", "WITHOUT ROWID"].into_iter().map(str::to_owned));
                } else {
                    candidates.extend(
                        ["ILIKE", "RETURNING", "jsonb_build_object"]
                            .into_iter()
                            .map(str::to_owned),
                    );
                }
                candidates.extend(self.schema.tables.iter().cloned());
                candidates.extend(self.schema.columns.iter().cloned());
                candidates.extend(self.schema.views.iter().map(|view| view.name.clone()));
                candidates.extend(self.schema.functions.iter().map(|function| function.name.clone()));
                for keyword in candidates.iter() {
                    if ui
                        .selectable_label(false, keyword)
                        .on_hover_text("Insert SQL keyword or expression")
                        .clicked()
                    {
                        self.query_text.push_str(keyword);
                        self.completion_open = false;
                    }
                }
            });
        }
        if self.snippets_open {
            card_frame(self.theme).show(ui, |ui| {
                ui.label(RichText::new("SQL snippets").strong());
                if compact_button(ui, "SELECT table", self.theme).clicked() {
                    self.insert_snippet("SELECT *\nFROM table_name\nLIMIT 100;");
                    self.snippets_open = false;
                }
                if compact_button(ui, "UPDATE by primary key", self.theme).clicked() {
                    self.insert_snippet("UPDATE table_name\nSET column_name = value\nWHERE id = 1;");
                    self.snippets_open = false;
                }
            });
        }
        if !self.diagnostics.is_empty() {
            ui.colored_label(self.theme.warning, format!("Diagnostics · {}", self.diagnostics.len()));
            for diagnostic in &self.diagnostics {
                ui.colored_label(self.theme.warning, format!("• {diagnostic}"));
            }
        }
        ui.add_space(12.0);
        let result = self.query_result.clone();
        ui.horizontal_wrapped(|ui| {
            ui.label(RichText::new("Results").strong().color(self.theme.text_primary));
            let row_label = result
                .as_ref()
                .map(|value| format!("{} rows · {} ms", value.row_count, value.duration_ms))
                .unwrap_or_else(|| "No result".to_owned());
            ui.label(RichText::new(row_label).small().color(self.theme.text_muted));
            ui.label(
                RichText::new(self.runtime_message.as_str())
                    .small()
                    .color(self.theme.text_muted),
            );
            if result.is_some() && compact_button(ui, "Export", self.theme).clicked() {
                self.export_open = true;
            }
        });
        if self.export_open {
            card_frame(self.theme).show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label("Export results");
                    ui.selectable_value(&mut self.export_format, "CSV".to_owned(), "CSV");
                    ui.selectable_value(&mut self.export_format, "TSV".to_owned(), "TSV");
                    input(ui, &mut self.export_path, "output path", 260.0, self.theme);
                    if compact_button(ui, "Export", self.theme).clicked() {
                        if let Some(result) = result.as_ref() {
                            self.export_result(result);
                        }
                    }
                    if compact_button(ui, "Cancel", self.theme).clicked() {
                        self.export_open = false;
                    }
                });
            });
        }
        toolbar_frame(self.theme).show(ui, |ui| {
            ui.horizontal(|ui| {
                section_label(ui, "ROW MUTATION", self.theme);
                ui.label(
                    RichText::new("Requires a primary key")
                        .small()
                        .color(self.theme.text_muted),
                );
                if !self.can_mutate_active_connection() {
                    ui.label(icon_text(
                        Icon::TriangleAlert,
                        if self.active_connection().is_some_and(|connection| connection.readonly) {
                            "Read-only connection: row changes are disabled"
                        } else {
                            "Connect with write access to change rows"
                        },
                        self.theme.warning,
                    ));
                }
            });
            ui.add_space(4.0);
            ui.horizontal_wrapped(|ui| {
                input(ui, &mut self.mutation_table, "table", 104.0, self.theme);
                input(ui, &mut self.mutation_pk_column, "pk column", 104.0, self.theme);
                input(ui, &mut self.mutation_pk_value, "pk value", 104.0, self.theme);
                input(ui, &mut self.mutation_column, "column", 104.0, self.theme);
                input(ui, &mut self.mutation_value, "new value", 104.0, self.theme);
                if compact_button_enabled(ui, "Update", self.can_mutate_active_connection(), self.theme).clicked() {
                    if let Some(connection) = self.active_connection().cloned() {
                        let request_id = self.task_bridge.next_request_id();
                        let _ = self.task_bridge.send(UiCommand::UpdateTableRow {
                            request_id,
                            connection_id: connection.id.clone(),
                            schema: self.active_schema().to_owned(),
                            table: self.mutation_table.clone(),
                            column: self.mutation_column.clone(),
                            value: UiCell::Text(self.mutation_value.clone()),
                            pk_columns: vec![self.mutation_pk_column.clone()],
                            pk_values: vec![UiCell::Text(self.mutation_pk_value.clone())],
                        });
                    }
                }
                if compact_button_enabled(ui, "Delete", self.can_mutate_active_connection(), self.theme).clicked() {
                    self.mutation_delete_confirmation = true;
                }
                if self.mutation_delete_confirmation {
                    ui.colored_label(self.theme.warning, "Confirm delete?");
                    if compact_button(ui, "Yes", self.theme).clicked() {
                        if let Some(connection) = self.active_connection().cloned() {
                            let request_id = self.task_bridge.next_request_id();
                            let _ = self.task_bridge.send(UiCommand::DeleteTableRow {
                                request_id,
                                connection_id: connection.id.clone(),
                                schema: self.active_schema().to_owned(),
                                table: self.mutation_table.clone(),
                                pk_columns: vec![self.mutation_pk_column.clone()],
                                pk_values: vec![UiCell::Text(self.mutation_pk_value.clone())],
                            });
                        }
                        self.mutation_delete_confirmation = false;
                    }
                    if compact_button(ui, "No", self.theme).clicked() {
                        self.mutation_delete_confirmation = false;
                    }
                }
            });
        });
        ui.add_space(8.0);
        let results_width = ui.max_rect().width();
        panel_frame(self.theme).show(ui, |ui| {
            ui.set_min_width((results_width - 24.0).max(0.0));
            if let Some(result) = result {
                self.draw_result_grid(ui, &result);
            } else {
                ui.centered_and_justified(|ui| {
                    ui.label(RichText::new("Run a query to see results").color(self.theme.text_muted));
                });
            }
        });
    }

    fn export_result(&mut self, result: &UiQueryResult) {
        let path = self.export_path.trim();
        if path.is_empty() {
            self.runtime_message = "Choose an export path first".to_owned();
            return;
        }
        let delimiter = if self.export_format == "CSV" { "," } else { "\t" };
        let mut output = result
            .columns
            .iter()
            .map(|column| column.name.clone())
            .collect::<Vec<_>>()
            .join(delimiter);
        output.push('\n');
        for row in &result.rows {
            output.push_str(
                &row.iter()
                    .map(|cell| match cell {
                        UiCell::Null => String::new(),
                        UiCell::Boolean(v) => v.to_string(),
                        UiCell::Number(v) | UiCell::Text(v) | UiCell::Json(v) | UiCell::Bytes(v) => v.clone(),
                    })
                    .collect::<Vec<_>>()
                    .join(delimiter),
            );
            output.push('\n');
        }
        match std::fs::write(path, output) {
            Ok(()) => self.runtime_message = format!("Exported {} rows to {path}", result.rows.len()),
            Err(error) => self.runtime_message = format!("Export failed: {error}"),
        }
        self.export_open = false;
    }

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

        if ui.input(|input| input.key_pressed(egui::Key::C) && input.modifiers.command) {
            self.copy_selected_cell(ui, result);
        }
        ui.horizontal(|ui| {
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
                    "Double-click a cell to edit · drag the divider to resize"
                } else {
                    "Click a cell to select · drag the divider to resize"
                })
                .small()
                .color(self.theme.text_muted),
            );
        });
        ui.add_space(6.0);

        let indexes =
            crate::filtered_sorted_indexes(result, &self.grid_filter, self.grid_sort_column, self.grid_sort_desc);
        ui.label(
            RichText::new(format!("{} matching rows · virtualized", indexes.len()))
                .small()
                .color(self.theme.text_muted),
        );
        ui.add_space(4.0);

        let widths = self.column_widths(result.columns.len());
        let row_offset = if self.active_tab == WorkspaceTab::Table && self.table_view == TableView::Data {
            self.table_data_offset
        } else {
            0
        };
        egui::ScrollArea::horizontal().show(ui, |ui| {
            ui.set_min_width(GRID_ROW_NUMBER_WIDTH + widths.iter().sum::<f32>());
            self.draw_grid_header(ui, result, &widths);
            egui::ScrollArea::vertical()
                .max_height(250.0)
                .show_rows(ui, 24.0, indexes.len(), |ui, range| {
                    for position in range {
                        let row_index = indexes[position];
                        let row = &result.rows[row_index];
                        let row_selected = self.selected_row == Some(row_index);
                        ui.horizontal(|ui| {
                            let row_number = crate::displayed_row_number(row_offset, row_index);
                            let row_response = ui.add_sized(
                                [GRID_ROW_NUMBER_WIDTH, 24.0],
                                egui::SelectableLabel::new(
                                    row_selected,
                                    RichText::new(row_number.to_string())
                                        .monospace()
                                        .small()
                                        .color(if row_selected {
                                            self.theme.accent
                                        } else {
                                            self.theme.text_muted
                                        }),
                                ),
                            );
                            if row_response.clicked() {
                                self.selected_cell = None;
                                self.selected_row = Some(row_index);
                                self.data_editing_cell = None;
                                self.data_edit_value.clear();
                                self.copy_status.clear();
                            }
                            for (column_index, cell) in row.iter().enumerate().take(result.columns.len()) {
                                let width = widths.get(column_index).copied().unwrap_or(180.0);
                                let fill = if row_selected {
                                    self.theme.surface_active
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
                                                row_selected || self.selected_cell == Some((row_index, column_index));
                                            let editing =
                                                editable && self.data_editing_cell == Some((row_index, column_index));
                                            if editing {
                                                let response = ui.add_sized(
                                                    [width - 12.0, 22.0],
                                                    TextEdit::singleline(&mut self.data_edit_value)
                                                        .margin(egui::Margin::symmetric(6.0, 2.0))
                                                        .text_color(self.theme.text_primary),
                                                );
                                                response.request_focus();
                                                let commit = response.lost_focus()
                                                    && ui.input(|input| input.key_pressed(egui::Key::Enter));
                                                if commit {
                                                    self.submit_data_cell_edit(result, row_index, column_index);
                                                } else if ui.input(|input| input.key_pressed(egui::Key::Escape)) {
                                                    self.data_editing_cell = None;
                                                    self.data_edit_value.clear();
                                                }
                                            } else {
                                                let response = ui.add_sized(
                                                    [width - 12.0, 22.0],
                                                    egui::SelectableLabel::new(selected, Self::cell_label(cell)),
                                                );
                                                if response.double_clicked() && editable {
                                                    self.begin_data_cell_edit(row_index, column_index, cell);
                                                } else if response.clicked() {
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
    }

    fn copy_selected_cell(&mut self, ui: &mut egui::Ui, result: &UiQueryResult) {
        let Some((row_index, column_index)) = self.selected_cell else {
            self.copy_status = "Select a cell first".to_owned();
            return;
        };
        let Some(cell) = result.rows.get(row_index).and_then(|row| row.get(column_index)) else {
            self.copy_status = "Selected cell is no longer available".to_owned();
            return;
        };
        ui.output_mut(|output| output.copied_text = crate::cell_text(cell));
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
        let row_text = row.iter().map(crate::cell_text).collect::<Vec<_>>().join("\t");
        ui.output_mut(|output| output.copied_text = row_text);
        self.copy_status = "Row copied".to_owned();
    }

    fn column_widths(&mut self, count: usize) -> Vec<f32> {
        if self.grid_column_widths.len() != count {
            self.grid_column_widths = vec![180.0; count];
        }
        self.grid_column_widths.clone()
    }

    fn draw_grid_header(&mut self, ui: &mut egui::Ui, result: &UiQueryResult, widths: &[f32]) {
        ui.horizontal(|ui| {
            ui.add_sized(
                [GRID_ROW_NUMBER_WIDTH, 28.0],
                egui::Button::new(RichText::new("#").strong().color(self.theme.text_muted))
                    .fill(self.theme.surface_hover),
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
                        RichText::new(format!("{}{}", column.name, sort_marker))
                            .strong()
                            .color(self.theme.text_primary),
                    )
                    .fill(self.theme.surface_hover),
                );
                if response.clicked() {
                    if self.grid_sort_column == Some(index) {
                        self.grid_sort_desc = !self.grid_sort_desc;
                    } else {
                        self.grid_sort_column = Some(index);
                        self.grid_sort_desc = false;
                    }
                }
                let (_divider_rect, divider) = ui.allocate_exact_size(egui::vec2(4.0, 28.0), Sense::drag());
                if divider.drag_started() {
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
    fn sql_layouter(ui: &egui::Ui, text: &str, wrap_width: f32, theme: DbProTheme) -> Arc<egui::Galley> {
        let keywords = [
            "select",
            "from",
            "where",
            "and",
            "or",
            "join",
            "left",
            "right",
            "inner",
            "group",
            "by",
            "order",
            "limit",
            "offset",
            "insert",
            "into",
            "values",
            "update",
            "set",
            "delete",
            "create",
            "table",
            "alter",
            "drop",
            "as",
            "on",
            "is",
            "null",
            "not",
            "returning",
            "with",
            "explain",
        ];
        let mut job = LayoutJob::default();
        job.wrap.max_width = wrap_width;
        let mut current = String::new();
        let mut in_string = false;
        let mut in_comment = false;
        let flush = |job: &mut LayoutJob, value: &mut String, color: Color32| {
            if !value.is_empty() {
                job.append(
                    value,
                    0.0,
                    TextFormat {
                        font_id: FontId::monospace(14.0),
                        color,
                        ..Default::default()
                    },
                );
                value.clear();
            }
        };
        let chars: Vec<char> = text.chars().collect();
        let mut index = 0;
        while index < chars.len() {
            let ch = chars[index];
            if !in_string && !in_comment && ch == '-' && chars.get(index + 1) == Some(&'-') {
                flush(&mut job, &mut current, theme.text_secondary);
                in_comment = true;
                current.push(ch);
            } else if in_comment {
                current.push(ch);
                if ch == '\n' {
                    flush(&mut job, &mut current, theme.code_comment);
                    in_comment = false;
                }
            } else if ch == '\'' {
                current.push(ch);
                if in_string {
                    flush(&mut job, &mut current, theme.code_string);
                    in_string = false;
                } else {
                    flush(&mut job, &mut current, theme.code_string);
                    in_string = true;
                }
            } else if in_string || ch.is_alphanumeric() || ch == '_' {
                current.push(ch);
            } else {
                let word = current.to_lowercase();
                let color = if keywords.contains(&word.as_str()) {
                    theme.code_keyword
                } else if current.chars().all(|value| value.is_ascii_digit()) && !current.is_empty() {
                    theme.code_number
                } else {
                    theme.text_primary
                };
                flush(&mut job, &mut current, color);
                job.append(
                    &ch.to_string(),
                    0.0,
                    TextFormat {
                        font_id: FontId::monospace(14.0),
                        color: theme.text_primary,
                        ..Default::default()
                    },
                );
            }
            index += 1;
        }
        if in_string {
            flush(&mut job, &mut current, theme.code_string);
        } else if in_comment {
            flush(&mut job, &mut current, theme.code_comment);
        } else {
            let word = current.to_lowercase();
            let color = if keywords.contains(&word.as_str()) {
                theme.code_keyword
            } else {
                theme.text_primary
            };
            flush(&mut job, &mut current, color);
        }
        ui.fonts(|fonts| fonts.layout_job(job))
    }

    fn format_sql(sql: &str) -> String {
        let keywords = [
            "select", "from", "where", "group by", "order by", "limit", "values", "set",
        ];
        let mut formatted = sql.trim().to_owned();
        for keyword in keywords {
            formatted = formatted.replace(keyword, &keyword.to_uppercase());
        }
        formatted = formatted
            .replace(" FROM ", "\nFROM ")
            .replace(" WHERE ", "\nWHERE ")
            .replace(" GROUP BY ", "\nGROUP BY ")
            .replace(" ORDER BY ", "\nORDER BY ")
            .replace(" LIMIT ", "\nLIMIT ");
        formatted
    }

    fn parse_sql_diagnostics(sql: &str, driver: &str) -> Vec<String> {
        let mut diagnostics = Vec::new();
        let parse_result = if driver.eq_ignore_ascii_case("sqlite") {
            Parser::parse_sql(&SQLiteDialect {}, sql)
        } else if driver.eq_ignore_ascii_case("postgres") {
            Parser::parse_sql(&PostgreSqlDialect {}, sql)
        } else {
            Parser::parse_sql(&GenericDialect {}, sql)
        };
        if let Err(error) = parse_result {
            diagnostics.push(format!("SQL parser: {error}"));
        }
        if sql.trim().is_empty() {
            diagnostics.push("Query is empty".to_owned());
            return diagnostics;
        }
        let mut tokens = Vec::new();
        let mut current = String::new();
        let mut in_string = false;
        let mut parentheses = 0i32;
        for ch in sql.chars() {
            if ch == '\'' {
                in_string = !in_string;
                current.push(ch);
            } else if in_string {
                current.push(ch);
            } else if ch == '(' {
                parentheses += 1;
                tokens.push(current.to_lowercase());
                current.clear();
            } else if ch == ')' {
                parentheses -= 1;
                tokens.push(current.to_lowercase());
                current.clear();
                if parentheses < 0 {
                    diagnostics.push("Unexpected closing parenthesis".to_owned());
                    parentheses = 0;
                }
            } else if ch.is_whitespace() || ch == ';' || ch == ',' {
                if !current.is_empty() {
                    tokens.push(current.to_lowercase());
                    current.clear();
                }
            } else {
                current.push(ch);
            }
        }
        if !current.is_empty() {
            tokens.push(current.to_lowercase());
        }
        if in_string {
            diagnostics.push("Unclosed string literal".to_owned());
        }
        if parentheses > 0 {
            diagnostics.push("Unclosed parenthesis".to_owned());
        }
        if tokens.first().map(String::as_str) == Some("select") && !tokens.iter().any(|token| token == "from") {
            diagnostics.push("SELECT statement is missing FROM".to_owned());
        }
        if tokens.first().map(String::as_str) == Some("update") && !tokens.iter().any(|token| token == "where") {
            diagnostics.push("UPDATE without WHERE will affect every row".to_owned());
        }
        let lower = sql.to_lowercase();
        if driver.eq_ignore_ascii_case("sqlite") && tokens.iter().any(|token| token == "ilike") {
            diagnostics.push("SQLite does not support ILIKE; use LIKE or lower()".to_owned());
        }
        if driver.eq_ignore_ascii_case("postgres") && tokens.iter().any(|token| token == "glob") {
            diagnostics.push("GLOB is SQLite-specific; use LIKE for PostgreSQL".to_owned());
        }
        if lower.contains("select * from") && lower.contains("select * from select") {
            diagnostics.push("Subquery must be enclosed in parentheses".to_owned());
        }
        diagnostics
    }

    fn refresh_diagnostics(&mut self) {
        let driver = self.active_driver();
        self.diagnostics = Self::parse_sql_diagnostics(&self.query_text, driver);
    }

    fn insert_snippet(&mut self, snippet: &str) {
        if !self.query_text.trim().is_empty() {
            self.query_text.push_str("\n\n");
        }
        self.query_text.push_str(snippet);
    }
}
