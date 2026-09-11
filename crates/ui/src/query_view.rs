use super::*;

impl DbProApp {
    pub(super) fn draw_query(&mut self, ui: &mut egui::Ui) {
        self.refresh_diagnostics();
        let modifier = Self::primary_modifier_label();
        let query_title = self
            .query_documents
            .get(self.active_query_document)
            .map(|document| document.title.clone())
            .unwrap_or_else(|| "Query".to_owned());
        ui.horizontal(|ui| {
            ui.label(RichText::new(query_title).strong().color(self.theme.text_primary));
            ui.label(icon_text(Icon::ChevronRight, "", self.theme.text_muted));
            ui.label(
                RichText::new(format!("{} / {}", self.active_connection_name(), self.active_schema()))
                    .small()
                    .color(self.theme.accent),
            );
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                let running = self.next_query_request.is_some();
                let run_button = if running {
                    secondary_button_with_icon(ui, Icon::Square, "Stop", self.theme).on_hover_text("Stop query (Esc)")
                } else {
                    primary_button_with_icon(ui, Icon::Play, "Run", self.theme)
                        .on_hover_text(format!("Run query ({modifier}↵)"))
                };
                if run_button.clicked() {
                    if let Some(request_id) = self.next_query_request {
                        self.cancel_query(request_id);
                    } else {
                        self.dispatch_query();
                    }
                }
                if self.query_tools_open {
                    if secondary_button_with_icon(ui, Icon::WandSparkles, "Format", self.theme).clicked() {
                        self.query_text = Self::format_sql(&self.query_text);
                    }
                    if secondary_button_with_icon(ui, Icon::ChartNoAxesCombined, "Explain", self.theme)
                        .on_hover_text("Run a read-only query plan")
                        .clicked()
                    {
                        self.explain_query();
                    }
                    if secondary_button_with_icon(ui, Icon::Bot, "Ask Agent", self.theme).clicked() {
                        self.open_agent_prompt(
                            if self.selected_query.trim().is_empty() {
                                "Explain the current SQL and suggest improvements"
                            } else {
                                "Explain the selected SQL and suggest improvements"
                            },
                            ui.ctx(),
                        );
                    }
                    if secondary_button_with_icon(ui, Icon::Save, "Save", self.theme).clicked() {
                        if let Some(connection) = self.active_connection().cloned() {
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
                                folder: (!self.query_folder.trim().is_empty())
                                    .then(|| self.query_folder.trim().to_owned()),
                            });
                            self.runtime_message = "Saving query…".to_owned();
                        }
                    }
                }
                if compact_icon_button(ui, Icon::MoreHorizontal, self.theme)
                    .on_hover_text("More query actions")
                    .clicked()
                {
                    self.query_tools_open = !self.query_tools_open;
                }
                if self.query_tools_open
                    && ghost_button(
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
        });
        ui.add_space(8.0);
        ui.horizontal_wrapped(|ui| {
            if self.query_tools_open {
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
                input(ui, &mut self.query_folder, "folder (optional)", 150.0, self.theme);
                if ghost_button(ui, "New folder", self.theme).clicked() {
                    if let Some(connection) = self.active_connection().cloned() {
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
                ui.label(
                    RichText::new(format!("{} px", self.editor_font_size))
                        .small()
                        .color(self.theme.text_muted),
                );
            }
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
        self.draw_query_editor(ui);
        if self.completion_open {
            card_frame(self.theme).show(ui, |ui| {
                ui.label(RichText::new("SQL completion").strong());
                let is_sqlite = self.active_driver().eq_ignore_ascii_case("sqlite");
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
                candidates.extend(self.active_schema_table_names());
                candidates.extend(self.active_schema_column_names());
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
        ui.horizontal(|ui| {
            for (tab, label) in [
                (OutputTab::Results, "Results"),
                (OutputTab::Messages, "Messages"),
                (OutputTab::Explain, "Explain"),
                (OutputTab::History, "History"),
            ] {
                let selected = self.output_tab == tab;
                if tab_frame(self.theme, selected)
                    .show(ui, |ui| ui.selectable_label(selected, label))
                    .inner
                    .clicked()
                {
                    self.output_tab = tab;
                }
            }
            if let Some(request_id) = self.explain_request {
                ui.label(
                    RichText::new(format!("Explain request {}…", request_id.0))
                        .small()
                        .color(self.theme.text_muted),
                );
            }
        });
        ui.add_space(6.0);
        let result = self.query_result.clone();
        ui.add_space(8.0);
        match self.output_tab {
            OutputTab::Results => {
                let results_width = ui.max_rect().width();
                grid_frame(self.theme).show(ui, |ui| {
                    ui.set_min_width((results_width - 24.0).max(0.0));
                    ui.horizontal(|ui| {
                        let row_label = result
                            .as_ref()
                            .map(|value| format!("{} rows · {} ms", value.row_count, value.duration_ms))
                            .unwrap_or_else(|| "No result".to_owned());
                        ui.label(RichText::new(row_label).small().color(self.theme.text_muted));
                        if result.is_some() && compact_button(ui, "Export", self.theme).clicked() {
                            self.export_open = true;
                        }
                    });
                    if let Some(result) = result.as_ref() {
                        self.draw_result_grid(ui, result);
                    } else {
                        ui.centered_and_justified(|ui| {
                            ui.label(RichText::new("Run a query to see results").color(self.theme.text_muted));
                        });
                    }
                });
                self.draw_export_dialog(ui, result.as_ref());
            }
            OutputTab::Messages => {
                card_frame(self.theme).show(ui, |ui| {
                    if self.query_messages.is_empty() {
                        ui.label(RichText::new("No messages yet").color(self.theme.text_muted));
                    } else {
                        for message in self.query_messages.iter().rev().take(20) {
                            ui.label(RichText::new(message).small().color(self.theme.text_secondary));
                        }
                    }
                });
            }
            OutputTab::Explain => {
                card_frame(self.theme).show(ui, |ui| {
                    if let Some(plan) = self.explain_plan.as_deref() {
                        egui::ScrollArea::vertical().max_height(300.0).show(ui, |ui| {
                            ui.label(RichText::new(plan).monospace().color(self.theme.text_secondary));
                        });
                    } else {
                        ui.label(RichText::new("Run Explain to inspect the query plan").color(self.theme.text_muted));
                    }
                });
            }
            OutputTab::History => {
                card_frame(self.theme).show(ui, |ui| {
                    if self.query_history.is_empty() {
                        ui.label(RichText::new("No query history yet").color(self.theme.text_muted));
                    } else {
                        for query in self.query_history.iter().rev().take(20) {
                            ui.label(
                                RichText::new(query)
                                    .monospace()
                                    .small()
                                    .color(self.theme.text_secondary),
                            );
                        }
                    }
                });
            }
        }
    }

    fn draw_query_editor(&mut self, ui: &mut egui::Ui) {
        let editor_width = ui.max_rect().width();
        let editor_height = if self.query_result.is_some() {
            ui.available_height().clamp(260.0, 360.0)
        } else {
            ui.available_height().clamp(320.0, 480.0)
        };
        ui.allocate_ui_with_layout(
            egui::vec2(editor_width, editor_height),
            Layout::top_down(Align::Min),
            |ui| {
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
                        let editor_size = egui::vec2(editor_text_width, (editor_height - 40.0).max(260.0));
                        let theme = self.theme;
                        let output = ui.allocate_ui(editor_size, |ui| {
                            TextEdit::multiline(&mut self.query_text)
                                .font(FontId::monospace(self.editor_font_size))
                                .desired_width(f32::INFINITY)
                                .min_size(editor_size)
                                .desired_rows(14)
                                .layouter(&mut |ui, text, wrap_width| Self::sql_layouter(ui, text, wrap_width, theme))
                                .lock_focus(true)
                                .show(ui)
                        });
                        self.query_editor_focused = output.inner.response.has_focus();
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
            },
        );
    }

    fn draw_export_dialog(&mut self, ui: &mut egui::Ui, result: Option<&UiQueryResult>) {
        if !self.export_open {
            return;
        }
        card_frame(self.theme).show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label("Export results");
                ui.selectable_value(&mut self.export_format, "CSV".to_owned(), "CSV");
                ui.selectable_value(&mut self.export_format, "TSV".to_owned(), "TSV");
                input(ui, &mut self.export_path, "output path", 260.0, self.theme);
                if compact_button(ui, "Export", self.theme).clicked() {
                    if let Some(result) = result {
                        self.export_result(result);
                    }
                }
                if compact_button(ui, "Cancel", self.theme).clicked() {
                    self.export_open = false;
                }
            });
        });
    }

    pub(crate) fn explain_query(&mut self) {
        if self.explain_request.is_some() {
            return;
        }
        let Some(capabilities) = self.active_capabilities() else {
            self.runtime_message = "Explain is unavailable until a supported connection is active".to_owned();
            return;
        };
        if !capabilities.query.explain {
            self.runtime_message = format!("{} does not support Explain", self.active_driver());
            return;
        }
        let Some(connection_id) = self.active_connection_id.clone() else {
            self.runtime_message = "Connect to a database before explaining a query".to_owned();
            return;
        };
        let sql = if self.selected_query.trim().is_empty() {
            self.query_text.trim().to_owned()
        } else {
            self.selected_query.trim().to_owned()
        };
        if sql.is_empty() {
            self.runtime_message = "Enter a query before explaining it".to_owned();
            return;
        }
        let request_id = self.task_bridge.next_request_id();
        if self
            .task_bridge
            .send(UiCommand::ExplainQuery {
                request_id,
                connection_id,
                sql,
            })
            .is_ok()
        {
            self.explain_request = Some(request_id);
            self.explain_plan = None;
            self.output_tab = OutputTab::Explain;
            self.runtime_message = "Explaining query…".to_owned();
        }
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

    pub(crate) fn format_sql(sql: &str) -> String {
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

    pub(crate) fn parse_sql_diagnostics(sql: &str, driver: &str) -> Vec<String> {
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
