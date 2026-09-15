use super::*;
use crate::editor::{
    CompletionItemKind, CompletionTriggerKind, Diagnostic, EditorSnapshot, SelectionRange, SqlDialect, SqlEditor,
};
use crate::query::{CompletionContext, SchemaCompletionProvider};
use std::path::PathBuf;
use std::time::Instant;

/// Egress note shown with the AI prediction control (#242).
///
/// Inline prediction is the one AI feature that runs without any user action — `PredictionMode`
/// defaults to `Eager` (`crates/ui/src/editor/prediction.rs:5-11`) and a scheduled request follows
/// 300 ms after an edit or cursor move (`query_document.rs:13`) — so the control that chooses the
/// mode is where its data flow has to be stated. The registry entry recording the same facts is
/// `docs/release/known-limitations.md` LIM-019; with no key configured the runtime answers
/// "AI provider is not configured" and nothing leaves the machine (`crates/runtime/src/worker.rs:1118`).
const AI_PREDICTION_EGRESS_NOTE: &str =
    "Sends the SQL around your cursor and its schema context to your configured AI provider.";

/// Distinguishes concurrent temp files of one process (see [`write_file_atomically`]).
static ATOMIC_WRITE_SEQUENCE: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);

/// Write `contents` to `path`, publishing the file in one step (#244, E-1).
///
/// `std::fs::write(path, …)` streams straight at the destination, so a failure part-way through —
/// disk full, I/O error, volume removed — leaves a **truncated file that looks like a complete
/// export**. This writes a sibling temp file first (same directory, so the rename stays on one
/// filesystem), removes it if anything fails, and only then renames it over the destination: the
/// destination either keeps its previous content or holds the complete new bytes, never a prefix of
/// them.
pub(crate) fn write_file_atomically(path: &std::path::Path, contents: &[u8]) -> std::io::Result<()> {
    use std::io::Write as _;

    let parent = match path.parent() {
        Some(parent) if !parent.as_os_str().is_empty() => parent,
        _ => std::path::Path::new("."),
    };
    let Some(file_name) = path.file_name() else {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "export path has no file name",
        ));
    };
    let temp_path = parent.join(format!(
        ".{}.tmp.{}.{}",
        file_name.to_string_lossy(),
        std::process::id(),
        ATOMIC_WRITE_SEQUENCE.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
    ));

    let result = (|| -> std::io::Result<()> {
        let mut file = std::fs::File::create(&temp_path)?;
        file.write_all(contents)?;
        file.sync_all()?;
        drop(file);
        std::fs::rename(&temp_path, path)
    })();

    if result.is_err() {
        // Best effort: a failed export must not leave its scratch file behind either.
        let _ = std::fs::remove_file(&temp_path);
    }
    result
}

impl DbProApp {
    pub(super) fn draw_query(&mut self, ui: &mut egui::Ui) {
        ui.add_space(SPACE_SM);
        egui::Frame::none()
            .inner_margin(egui::Margin::symmetric(SPACE_MD, 0.0))
            .show(ui, |ui| {
                self.refresh_diagnostics();
                let more_anchor = self.draw_query_header(ui);
                if self.query_tools_open {
                    if let Some(anchor) = more_anchor {
                        self.draw_query_actions_menu(ui.ctx(), anchor);
                    }
                }
                if self.editor_search_open {
                    self.draw_editor_search_bar(ui);
                }
                self.draw_query_editor(ui);
                self.draw_floating_completion_popup(ui.ctx());
                if self.completion_open {
                    self.draw_sql_completion(ui);
                }
                if self.snippets_open {
                    self.draw_sql_snippets(ui);
                }
                self.draw_diagnostics(ui);
                self.draw_sql_parameters_panel(ui);
                self.draw_output_tabs(ui);

                let result = self.active_query_result().cloned();
                ui.add_space(8.0);
                self.draw_output_pane(ui, result.as_ref());
                self.draw_dirty_close_dialog(ui.ctx());
                self.draw_save_as_dialog(ui.ctx());
            });
    }

    /// Query title, connection breadcrumb, run/stop and the overflow button.
    /// Returns the overflow button rect so the actions menu can anchor to it.
    fn draw_query_header(&mut self, ui: &mut egui::Ui) -> Option<egui::Rect> {
        let modifier = Self::primary_modifier_label();
        let mut more_anchor = None;
        let doc_idx = self.active_query_document;
        let query_title = self
            .query_documents
            .get(doc_idx)
            .map(|document| document.title.clone())
            .unwrap_or_else(|| "Query".to_owned());
        let current_conn_id = self
            .query_documents
            .get(doc_idx)
            .and_then(|d| d.connection_id.clone())
            .or_else(|| self.active_connection_id.clone());
        let conn_label = self.active_query_connection_name().to_owned();
        let current_schema = self.active_query_schema().to_owned();

        let mut next_conn_id = None;
        let mut next_schema = None;

        ui.horizontal(|ui| {
            ui.label(
                RichText::new(query_title)
                    .font(font_subheading())
                    .strong()
                    .color(self.theme.text_primary),
            );
            ui.label(icon_text(Icon::ChevronRight, "", self.theme.text_muted));

            egui::ComboBox::from_id_salt(("query_header_conn", doc_idx))
                .selected_text(RichText::new(&conn_label).font(font_caption()).color(self.theme.accent))
                .show_ui(ui, |ui| {
                    for conn in &self.connections {
                        let is_selected = current_conn_id.as_deref() == Some(conn.id.as_str());
                        if ui.selectable_label(is_selected, &conn.name).clicked() {
                            next_conn_id = Some(conn.id.clone());
                        }
                    }
                });

            ui.label(icon_text(Icon::ChevronRight, "", self.theme.text_muted));

            let available_schemas = if !self.schema.schemas.is_empty() {
                self.schema.schemas.clone()
            } else if !self.query_capabilities().allows(|caps| caps.schema.schemas) {
                // Engines without named schemas (SQLite) expose a single default catalog.
                vec!["main".to_string()]
            } else {
                vec!["public".to_string()]
            };
            egui::ComboBox::from_id_salt(("query_header_schema", doc_idx))
                .selected_text(
                    RichText::new(&current_schema)
                        .font(font_caption())
                        .color(self.theme.text_secondary),
                )
                .show_ui(ui, |ui| {
                    for sch in &available_schemas {
                        let is_selected = &current_schema == sch;
                        if ui.selectable_label(is_selected, sch).clicked() {
                            next_schema = Some(sch.clone());
                        }
                    }
                });
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                let active_doc_running = self
                    .query_documents
                    .get(self.active_query_document)
                    .and_then(|doc| match doc.execution_state {
                        QueryExecutionState::Running(req) => Some(req),
                        _ => None,
                    });
                let running = active_doc_running.is_some();
                let cancel_supported = self.query_capabilities().allows(|c| c.query.cancel);
                let cancel_reason = self
                    .query_capabilities()
                    .feature_limitation(db_pro_core::domain::capabilities::CapabilityFeature::Cancel);
                let run_button = if running {
                    if cancel_supported {
                        secondary_button_with_icon(ui, Icon::Square, "Stop", self.theme)
                            .on_hover_text("Stop query (Esc)")
                    } else {
                        let tip = cancel_reason
                            .as_deref()
                            .unwrap_or("Query running (cancellation is unsupported by this provider)");
                        secondary_button_with_icon(ui, Icon::Loader, "Running…", self.theme).on_hover_text(tip)
                    }
                } else {
                    primary_button_with_icon(ui, Icon::Play, "Run", self.theme)
                        .on_hover_text(format!("Run query ({modifier}↵)"))
                };
                if run_button.clicked() {
                    if let Some(request_id) = active_doc_running {
                        if cancel_supported {
                            self.cancel_query(request_id);
                        } else {
                            self.runtime_message = cancel_reason
                                .unwrap_or_else(|| "Query cancellation is not supported for this provider".to_owned());
                        }
                    } else {
                        self.dispatch_query();
                    }
                }
                let more_response =
                    compact_icon_button(ui, Icon::MoreHorizontal, self.theme).on_hover_text("More query actions");
                if more_response.clicked() {
                    self.query_tools_open = !self.query_tools_open;
                }
                more_anchor = Some(more_response.rect);
            });
        });

        if let Some(cid) = next_conn_id {
            self.set_document_connection(doc_idx, Some(cid));
        }
        if let Some(sch) = next_schema {
            self.set_document_schema(doc_idx, Some(sch));
        }

        more_anchor
    }

    /// "Find in SQL" bar, shown while the editor search is open.
    fn draw_editor_search_bar(&mut self, ui: &mut egui::Ui) {
        ui.add_space(8.0);
        let mut goto_range = None;
        let mut close_search = false;

        if let Some(doc) = self.query_documents.get_mut(self.active_query_document) {
            ui.horizontal(|ui| {
                let prev_search = self.editor_search.clone();
                input(ui, &mut self.editor_search, "Find in SQL…", 240.0, self.theme);
                if self.editor_search != prev_search {
                    doc.search.query = self.editor_search.clone();
                    doc.search.update_matches(doc.buffer.text());
                    if let Some(first_match) = doc.search.matches.first().copied() {
                        doc.search.active_match_index = 0;
                        goto_range = Some(first_match);
                    }
                }

                if !self.editor_search.is_empty() {
                    let total = doc.search.matches.len();
                    let current = if total == 0 {
                        0
                    } else {
                        doc.search.active_match_index + 1
                    };
                    let label_text = if total == 0 {
                        "No matches".to_string()
                    } else {
                        format!("{current} of {total}")
                    };
                    ui.label(RichText::new(label_text).small().color(if total == 0 {
                        self.theme.danger
                    } else {
                        self.theme.text_muted
                    }));

                    if compact_icon_button(ui, Icon::ChevronUp, self.theme)
                        .on_hover_text("Previous match (Shift+Enter)")
                        .clicked()
                    {
                        if let Some(m) = doc.search.prev_match() {
                            goto_range = Some(m);
                        }
                    }
                    if compact_icon_button(ui, Icon::ChevronDown, self.theme)
                        .on_hover_text("Next match (Enter)")
                        .clicked()
                    {
                        if let Some(m) = doc.search.next_match() {
                            goto_range = Some(m);
                        }
                    }
                }

                if compact_icon_button(ui, Icon::X, self.theme)
                    .on_hover_text("Close find bar (Esc)")
                    .clicked()
                {
                    close_search = true;
                }
            });

            if let Some((start, end)) = goto_range {
                doc.cursor.set_offset(&doc.buffer, end);
                doc.selection = crate::editor::SelectionRange::new(start, end);
            }
        }

        if close_search {
            self.editor_search_open = false;
        }
    }

    /// Keyword / table / column completion list.
    fn draw_sql_completion(&mut self, ui: &mut egui::Ui) {
        card_frame(self.theme).show(ui, |ui| {
            ui.label(RichText::new("SQL completion").strong());
            let uses_positional = !self.query_capabilities().allows(|caps| caps.query.numbered_parameters);
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
            if uses_positional {
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
                    self.append_to_active_query(keyword);
                    self.completion_open = false;
                }
            }
        });
    }

    /// Quick SQL snippet inserters.
    fn draw_sql_snippets(&mut self, ui: &mut egui::Ui) {
        card_frame(self.theme).show(ui, |ui| {
            ui.label(RichText::new("SQL snippets").strong());
            for (label, snippet) in Self::builtin_sql_snippets() {
                if compact_button(ui, *label, self.theme).clicked() {
                    self.insert_snippet(snippet);
                    self.snippets_open = false;
                }
            }
        });
    }

    pub(crate) fn builtin_sql_snippets() -> &'static [(&'static str, &'static str)] {
        &[
            ("SELECT table", "SELECT *\nFROM table_name\nLIMIT 100;"),
            (
                "UPDATE by primary key",
                "UPDATE table_name\nSET column_name = value\nWHERE id = 1;",
            ),
            (
                "INSERT row",
                "INSERT INTO table_name (column_a, column_b)\nVALUES ($1, $2);",
            ),
            ("DELETE with WHERE", "DELETE FROM table_name\nWHERE id = $1;"),
            (
                "EXPLAIN ANALYZE",
                "EXPLAIN (ANALYZE, BUFFERS)\nSELECT *\nFROM table_name\nWHERE id = $1;",
            ),
            (
                "CREATE INDEX",
                "CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_table_column\nON table_name (column_name);",
            ),
        ]
    }

    /// Parser diagnostics for the current SQL, when any.
    fn draw_diagnostics(&mut self, ui: &mut egui::Ui) {
        if self.diagnostics.is_empty() {
            return;
        }
        ui.colored_label(self.theme.warning, format!("Diagnostics · {}", self.diagnostics.len()));
        for diagnostic in &self.diagnostics {
            ui.colored_label(self.theme.warning, format!("• {diagnostic}"));
        }
    }

    /// Discovered bind placeholders for the active document (#225 discovery slice).
    fn draw_sql_parameters_panel(&mut self, ui: &mut egui::Ui) {
        let sql = self.active_query_text().to_owned();
        let params = crate::query::discover_sql_parameters(&sql);
        if params.is_empty() {
            return;
        }
        let supports_parameters = self.query_capabilities().allows(|caps| caps.query.parameters);
        let doc_index = self.active_query_document;
        ui.add_space(SPACE_XS);
        ui.horizontal(|ui| {
            ui.colored_label(self.theme.accent, format!("Parameters · {}", params.len()));
            if !supports_parameters {
                ui.label(
                    RichText::new("provider does not advertise bindings yet")
                        .small()
                        .color(self.theme.warning),
                );
            }
        });
        for param in params {
            let mut value = self
                .query_documents
                .get(doc_index)
                .and_then(|doc| doc.parameter_values.get(&param.name).cloned())
                .unwrap_or_default();
            let mut is_secret = self
                .query_documents
                .get(doc_index)
                .is_some_and(|doc| doc.parameter_secrets.contains(&param.name));
            ui.horizontal(|ui| {
                let kind = match param.kind {
                    crate::query::ParameterKind::Numbered => "numbered",
                    crate::query::ParameterKind::Named => "named",
                    crate::query::ParameterKind::Positional => "positional",
                };
                ui.label(
                    RichText::new(format!("{} ({kind})", param.name))
                        .small()
                        .color(self.theme.text_secondary),
                );
                let edit = if is_secret {
                    egui::TextEdit::singleline(&mut value).password(true)
                } else {
                    egui::TextEdit::singleline(&mut value)
                };
                ui.add(edit.desired_width(180.0));
                ui.checkbox(&mut is_secret, "secret");
            });
            if let Some(doc) = self.query_documents.get_mut(doc_index) {
                doc.parameter_values.insert(param.name.clone(), value);
                if is_secret {
                    doc.parameter_secrets.insert(param.name.clone());
                } else {
                    doc.parameter_secrets.remove(&param.name);
                }
            }
        }
        ui.label(
            RichText::new("Values stay in-memory for this document; secret values are never persisted with drafts.")
                .small()
                .color(self.theme.text_muted),
        );
    }

    /// Output tab strip (Results / Messages / Explain / History).
    fn draw_output_tabs(&mut self, ui: &mut egui::Ui) {
        ui.add_space(SPACE_MD);
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing = egui::vec2(4.0, 0.0);
            for (tab, icon, label) in [
                (OutputTab::Results, Icon::Table2, "Results"),
                (OutputTab::Chart, Icon::BarChart3, "Chart"),
                (OutputTab::Messages, Icon::MessageSquareText, "Messages"),
                (OutputTab::Explain, Icon::ChartNoAxesCombined, "Explain"),
                (OutputTab::History, Icon::History, "History"),
            ] {
                let selected = self.active_query_output_tab() == tab;
                let bg_color = if selected {
                    self.theme.surface_active
                } else {
                    egui::Color32::TRANSPARENT
                };
                let text_color = if selected {
                    self.theme.text_primary
                } else {
                    self.theme.text_secondary
                };
                let icon_color = if selected {
                    self.theme.accent
                } else {
                    self.theme.text_muted
                };

                let resp = egui::Frame::none()
                    .fill(bg_color)
                    .rounding(egui::Rounding::same(RADIUS_SM))
                    .inner_margin(egui::Margin::symmetric(SPACE_SM, SPACE_XS))
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            ui.label(
                                RichText::new(char::from(icon).to_string())
                                    .font(egui::FontId::new(12.0, egui::FontFamily::Name("lucide".into())))
                                    .color(icon_color),
                            );
                            ui.add_space(2.0);
                            ui.label(RichText::new(label).font(font_ui_label()).color(text_color));
                            if tab == OutputTab::Results {
                                if let Some(res) = self.active_query_result() {
                                    badge(
                                        ui,
                                        &res.row_count.to_string(),
                                        self.theme.accent_soft,
                                        self.theme.accent,
                                    );
                                }
                            } else if tab == OutputTab::Messages && !self.active_query_messages().is_empty() {
                                badge(
                                    ui,
                                    &self.active_query_messages().len().to_string(),
                                    self.theme.surface_hover,
                                    self.theme.text_muted,
                                );
                            }
                        });
                    });

                if resp.response.interact(egui::Sense::click()).clicked() {
                    self.set_active_query_output_tab(tab);
                }
            }
            if let Some(request_id) = self.active_explain_request() {
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    ui.label(
                        RichText::new(format!("Explain request {}…", request_id.0))
                            .font(font_caption())
                            .color(self.theme.text_muted),
                    );
                });
            }
        });
        ui.add_space(SPACE_XS);
    }

    /// Body of the selected output tab.
    fn draw_output_pane(&mut self, ui: &mut egui::Ui, result: Option<&UiQueryResult>) {
        match self.active_query_output_tab() {
            OutputTab::Results => self.draw_results_pane(ui, result),
            OutputTab::Chart => self.draw_chart_pane(ui, result),
            OutputTab::Messages => self.draw_messages_pane(ui),
            OutputTab::Explain => self.draw_explain_pane(ui),
            OutputTab::History => self.draw_history_pane(ui),
        }
    }

    /// Results grid plus its row-count / export header.
    fn draw_results_pane(&mut self, ui: &mut egui::Ui, result: Option<&UiQueryResult>) {
        let results_width = ui.max_rect().width();
        grid_frame(self.theme).show(ui, |ui| {
            ui.set_min_width((results_width - 24.0).max(0.0));
            let result_count = self.active_query_result_count();
            if result_count > 1 {
                ui.horizontal(|ui| {
                    let active_index = self
                        .query_documents
                        .get(self.active_query_document)
                        .map_or(0, |doc| doc.active_result_index);
                    for index in 0..result_count {
                        if ui
                            .selectable_label(active_index == index, format!("Result {}", index + 1))
                            .clicked()
                        {
                            self.set_active_query_result(index);
                        }
                    }
                });
            }
            ui.horizontal(|ui| {
                if let Some(value) = result {
                    ui.label(
                        RichText::new(format!("{} rows · {} ms", value.row_count, value.duration_ms))
                            .small()
                            .color(self.theme.text_muted),
                    );
                    if compact_button(ui, "Export", self.theme).clicked() {
                        self.export_open = true;
                    }
                }
            });
            if let Some(result) = result {
                self.draw_result_grid(ui, result);
            } else {
                ui.centered_and_justified(|ui| {
                    empty_state(
                        ui,
                        Icon::Table2,
                        "No results yet",
                        "Run a query to populate this result grid.",
                        self.theme,
                    );
                });
            }
        });
        self.draw_export_dialog(ui, result);
        self.draw_destructive_run_dialog(ui);
    }

    /// Query notice log.
    fn draw_messages_pane(&mut self, ui: &mut egui::Ui) {
        let output_width = ui.available_width();
        let messages = self.active_query_messages();
        card_frame(self.theme).show(ui, |ui| {
            ui.set_min_width((output_width - 24.0).max(0.0));
            if messages.is_empty() {
                empty_state(
                    ui,
                    Icon::MessageSquareText,
                    "No messages yet",
                    "Query notices and execution details will appear here.",
                    self.theme,
                );
            } else {
                for message in messages.iter().rev().take(20) {
                    ui.label(RichText::new(message).small().color(self.theme.text_secondary));
                }
            }
        });
    }

    fn draw_chart_pane(&mut self, ui: &mut egui::Ui, result: Option<&UiQueryResult>) {
        use crate::{ChartAggregation, ChartEngine, ChartRenderer, ChartType};

        let output_width = ui.available_width();
        card_frame(self.theme).show(ui, |ui| {
            ui.set_min_width((output_width - 24.0).max(0.0));
            let Some(result) = result else {
                empty_state(
                    ui,
                    Icon::BarChart3,
                    "No result to chart",
                    "Run a query that returns rows, then open Chart.",
                    self.theme,
                );
                return;
            };
            if result.columns.is_empty() || result.rows.is_empty() {
                empty_state(
                    ui,
                    Icon::BarChart3,
                    "No data to chart",
                    "The current result has no rows.",
                    self.theme,
                );
                return;
            }

            let doc_index = self.active_query_document;
            let column_names: Vec<String> = result.columns.iter().map(|c| c.name.clone()).collect();
            let column_types: Vec<String> = result.columns.iter().map(|c| c.data_type.clone()).collect();

            ui.horizontal(|ui| {
                ui.label(RichText::new("Type").small().color(self.theme.text_secondary));
                if let Some(doc) = self.query_documents.get_mut(doc_index) {
                    egui::ComboBox::from_id_salt("chart_type")
                        .selected_text(doc.chart_config.chart_type.to_string())
                        .show_ui(ui, |ui| {
                            for chart_type in [
                                ChartType::Bar,
                                ChartType::Line,
                                ChartType::Area,
                                ChartType::Scatter,
                                ChartType::Pie,
                            ] {
                                ui.selectable_value(
                                    &mut doc.chart_config.chart_type,
                                    chart_type,
                                    chart_type.to_string(),
                                );
                            }
                        });
                }

                ui.label(RichText::new("X").small().color(self.theme.text_secondary));
                if let Some(doc) = self.query_documents.get_mut(doc_index) {
                    let x_label = doc
                        .chart_config
                        .x_column
                        .and_then(|i| column_names.get(i))
                        .cloned()
                        .unwrap_or_else(|| column_names.first().cloned().unwrap_or_default());
                    egui::ComboBox::from_id_salt("chart_x")
                        .selected_text(x_label)
                        .show_ui(ui, |ui| {
                            for (idx, name) in column_names.iter().enumerate() {
                                ui.selectable_value(&mut doc.chart_config.x_column, Some(idx), name);
                            }
                        });
                }

                ui.label(RichText::new("Y").small().color(self.theme.text_secondary));
                if let Some(doc) = self.query_documents.get_mut(doc_index) {
                    let numeric_idxs: Vec<usize> = column_types
                        .iter()
                        .enumerate()
                        .filter(|(_, ty)| ChartEngine::is_numeric_column(ty))
                        .map(|(i, _)| i)
                        .collect();
                    let y_label = doc
                        .chart_config
                        .y_column
                        .and_then(|i| column_names.get(i))
                        .cloned()
                        .unwrap_or_else(|| {
                            numeric_idxs
                                .first()
                                .and_then(|i| column_names.get(*i))
                                .cloned()
                                .unwrap_or_else(|| "—".to_owned())
                        });
                    egui::ComboBox::from_id_salt("chart_y")
                        .selected_text(y_label)
                        .show_ui(ui, |ui| {
                            if numeric_idxs.is_empty() {
                                ui.label("No numeric columns");
                            }
                            for idx in numeric_idxs {
                                ui.selectable_value(&mut doc.chart_config.y_column, Some(idx), &column_names[idx]);
                            }
                        });
                }

                ui.label(RichText::new("Agg").small().color(self.theme.text_secondary));
                if let Some(doc) = self.query_documents.get_mut(doc_index) {
                    egui::ComboBox::from_id_salt("chart_agg")
                        .selected_text(doc.chart_config.aggregation.to_string())
                        .show_ui(ui, |ui| {
                            for agg in [
                                ChartAggregation::None,
                                ChartAggregation::Count,
                                ChartAggregation::Sum,
                                ChartAggregation::Average,
                                ChartAggregation::Min,
                                ChartAggregation::Max,
                            ] {
                                ui.selectable_value(&mut doc.chart_config.aggregation, agg, agg.to_string());
                            }
                        });
                }

                ui.label(RichText::new("Series").small().color(self.theme.text_secondary));
                if let Some(doc) = self.query_documents.get_mut(doc_index) {
                    let series_label = doc
                        .chart_config
                        .series_column
                        .and_then(|i| column_names.get(i))
                        .cloned()
                        .unwrap_or_else(|| "—".to_owned());
                    egui::ComboBox::from_id_salt("chart_series")
                        .selected_text(series_label)
                        .show_ui(ui, |ui| {
                            ui.selectable_value(&mut doc.chart_config.series_column, None, "—");
                            for (idx, name) in column_names.iter().enumerate() {
                                ui.selectable_value(&mut doc.chart_config.series_column, Some(idx), name);
                            }
                        });
                }
            });

            ui.add_space(8.0);

            let config = self
                .query_documents
                .get(doc_index)
                .map(|doc| doc.chart_config.clone())
                .unwrap_or_default();
            let projection = if config.chart_type == ChartType::Pie {
                ChartEngine::project_pie(&result.columns, &result.rows, &config)
            } else {
                ChartEngine::project(&result.columns, &result.rows, &config)
            };
            ui.allocate_ui(egui::vec2(ui.available_width(), 280.0), |ui| {
                ChartRenderer::draw(ui, &projection.points, &config, &self.theme);
            });
            let mut footer = format!("{} points (max {})", projection.points.len(), config.max_points.max(1));
            if projection.skipped_null_y > 0 {
                footer.push_str(&format!(" · skipped {} null/non-numeric Y", projection.skipped_null_y));
            }
            if projection.x_fallback_to_index > 0 {
                let x_note = if config.chart_type == ChartType::Pie {
                    format!(" · {} X nulls labeled NULL", projection.x_fallback_to_index)
                } else {
                    format!(" · {} X nulls mapped to row index", projection.x_fallback_to_index)
                };
                footer.push_str(&x_note);
            }
            ui.label(RichText::new(footer).small().color(self.theme.text_muted));
        });
    }

    /// EXPLAIN output for the last explain request.
    fn draw_explain_pane(&mut self, ui: &mut egui::Ui) {
        let output_width = ui.available_width();
        card_frame(self.theme).show(ui, |ui| {
            ui.set_min_width((output_width - 24.0).max(0.0));
            if let Some(plan) = self.active_explain_plan() {
                egui::ScrollArea::vertical().max_height(300.0).show(ui, |ui| {
                    ui.label(RichText::new(plan).monospace().color(self.theme.text_secondary));
                });
            } else {
                empty_state(
                    ui,
                    Icon::ChartNoAxesCombined,
                    "No query plan yet",
                    "Run Explain to inspect the query plan.",
                    self.theme,
                );
            }
        });
    }

    /// Recently executed queries.
    fn draw_history_pane(&mut self, ui: &mut egui::Ui) {
        let output_width = ui.available_width();
        card_frame(self.theme).show(ui, |ui| {
            ui.set_min_width((output_width - 24.0).max(0.0));
            ui.horizontal(|ui| {
                ui.label(RichText::new("Search").small().color(self.theme.text_muted));
                ui.add_sized(
                    [220.0, 24.0],
                    egui::TextEdit::singleline(&mut self.query_history_search).hint_text("SQL, connection, schema"),
                );
                if compact_button(ui, "Clear History", self.theme).clicked() {
                    self.query_history_entries.clear();
                    self.runtime_message = "Query history cleared".to_owned();
                }
            });
            if self.query_history_entries.is_empty() {
                empty_state(
                    ui,
                    Icon::History,
                    "No query history yet",
                    "Executed queries will appear here.",
                    self.theme,
                );
            } else {
                let search = self.query_history_search.trim().to_lowercase();
                let entries = self
                    .query_history_entries
                    .iter()
                    .rev()
                    .filter(|entry| {
                        search.is_empty()
                            || entry.sql.to_lowercase().contains(&search)
                            || entry
                                .connection_id
                                .as_deref()
                                .is_some_and(|connection| connection.to_lowercase().contains(&search))
                            || entry
                                .schema
                                .as_deref()
                                .is_some_and(|schema| schema.to_lowercase().contains(&search))
                    })
                    .take(20)
                    .cloned()
                    .collect::<Vec<_>>();
                for entry in entries {
                    ui.horizontal_wrapped(|ui| {
                        let status = match entry.status {
                            UiQueryHistoryStatus::Success => "OK",
                            UiQueryHistoryStatus::Failed => "Failed",
                            UiQueryHistoryStatus::Cancelled => "Cancelled",
                        };
                        ui.label(RichText::new(status).small().color(self.theme.text_muted));
                        ui.label(
                            RichText::new(format!("{} ms", entry.duration_ms))
                                .small()
                                .color(self.theme.text_muted),
                        );
                        ui.label(
                            RichText::new(entry.sql.lines().next().unwrap_or("query"))
                                .monospace()
                                .small()
                                .color(self.theme.text_secondary),
                        );
                        if compact_button(ui, "Open", self.theme).clicked() {
                            self.open_history_entry(&entry, false);
                        }
                        if compact_button(ui, "Copy SQL", self.theme).clicked() {
                            ui.output_mut(|output| output.copied_text = entry.sql.clone());
                        }
                        if compact_button(ui, "Run Again", self.theme).clicked() {
                            self.open_history_entry(&entry, true);
                        }
                    });
                }
            }
        });
    }

    fn draw_query_actions_menu(&mut self, ctx: &egui::Context, anchor: egui::Rect) {
        let menu_width = 264.0;
        let menu_position = egui::pos2((anchor.right() - menu_width).max(8.0), anchor.bottom() + 4.0);
        let mut close_menu = false;
        let menu = egui::Area::new(egui::Id::new("query_actions_menu"))
            .order(egui::Order::Foreground)
            .fixed_pos(menu_position)
            .show(ctx, |ui| {
                egui::Frame {
                    fill: self.theme.surface_elevated,
                    inner_margin: egui::Margin::same(8.0),
                    rounding: egui::Rounding::same(8.0),
                    stroke: egui::Stroke::new(1.0, self.theme.border_subtle),
                    ..Default::default()
                }
                .show(ui, |ui| {
                    ui.set_min_width(menu_width);
                    ui.label(
                        RichText::new("Query actions")
                            .small()
                            .strong()
                            .color(self.theme.text_muted),
                    );
                    ui.add_space(4.0);
                    close_menu |= self.draw_query_run_actions(ui, ctx);
                    ui.separator();
                    ui.label(RichText::new("Editor").small().strong().color(self.theme.text_muted));
                    ui.add_space(4.0);
                    close_menu |= self.draw_query_editor_actions(ui);
                    ui.label(
                        RichText::new(format!("Editor font · {} px", self.editor_font_size))
                            .small()
                            .color(self.theme.text_muted),
                    );
                });
            });
        let clicked_outside = ctx.input(|input| {
            input.pointer.any_click()
                && input
                    .pointer
                    .interact_pos()
                    .is_some_and(|position| !menu.response.rect.contains(position) && !anchor.contains(position))
        });
        if clicked_outside || close_menu {
            self.query_tools_open = false;
        }
    }

    /// Run / format / explain / agent / save entries.
    /// Returns true when the menu should close.
    fn draw_query_run_actions(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) -> bool {
        let mut close_menu = false;
        if menu_button_with_icon(
            ui,
            Icon::Play,
            if self.selected_query.is_empty() {
                "Run query"
            } else {
                "Run selection"
            },
            self.theme,
        )
        .clicked()
        {
            self.dispatch_query();
            close_menu = true;
        }
        if menu_button_with_icon(ui, Icon::WandSparkles, "Format SQL", self.theme).clicked() {
            self.format_active_query();
            close_menu = true;
        }
        if menu_button_with_icon(ui, Icon::ChartNoAxesCombined, "Explain query", self.theme).clicked() {
            self.explain_query();
            close_menu = true;
        }
        if menu_button_with_icon(ui, Icon::Bot, "Ask Agent", self.theme).clicked() {
            self.open_agent_prompt(
                if self.selected_query.trim().is_empty() {
                    "Explain the current SQL and suggest improvements"
                } else {
                    "Explain the selected SQL and suggest improvements"
                },
                ctx,
            );
            close_menu = true;
        }
        if menu_button_with_icon(ui, Icon::Save, "Save query", self.theme).clicked() {
            self.save_query_document();
            close_menu = true;
        }
        if menu_button_with_icon(ui, Icon::Save, "Save query as…", self.theme).clicked() {
            self.open_save_as_dialog();
            close_menu = true;
        }
        close_menu
    }

    /// Persists the current editor contents as a named saved query.
    fn save_query_document(&mut self) {
        self.save_query_document_at(self.active_query_document);
    }

    pub(crate) fn save_query_document_at(&mut self, document_index: usize) {
        if self
            .query_documents
            .get(document_index)
            .and_then(|document| document.file_path.as_ref())
            .is_some()
            && document_index == self.active_query_document
            && self.save_active_workspace_file()
        {
            return;
        }
        let Some(connection_id) = self
            .query_documents
            .get(document_index)
            .and_then(|document| document.connection_id.clone())
            .or_else(|| self.active_connection_id.clone())
        else {
            self.runtime_message = "Create or select a connection first".to_owned();
            return;
        };
        let request_id = self.task_bridge.next_request_id();
        let document_id = self
            .query_documents
            .get(document_index)
            .map(|document| document.id.clone());
        let name = self
            .query_documents
            .get(document_index)
            .map(|document| document.title.clone())
            .unwrap_or_else(|| "Saved query".to_owned());
        let saved_query_id = self
            .query_documents
            .get(document_index)
            .and_then(|document| document.saved_query_id.clone());
        let sql = self
            .query_documents
            .get(document_index)
            .map_or_else(String::new, |document| document.text().to_owned());
        self.dispatch_command(UiCommand::SaveQuery {
            request_id,
            connection_id,
            saved_query_id,
            name,
            sql,
            folder: (!self.query_folder.trim().is_empty()).then(|| self.query_folder.trim().to_owned()),
        });
        if let Some(document_id) = document_id {
            self.query_save_requests.insert(request_id, document_id);
        }
        self.runtime_message = "Saving query…".to_owned();
    }

    pub(crate) fn open_save_as_dialog(&mut self) {
        self.save_as_name = self
            .query_documents
            .get(self.active_query_document)
            .map_or_else(|| "Saved query".to_owned(), |document| document.title.clone());
        self.save_as_open = true;
    }

    fn draw_save_as_dialog(&mut self, ctx: &egui::Context) {
        if !self.save_as_open {
            return;
        }
        let mut save = false;
        let mut cancel = false;
        egui::Window::new("Save Query As")
            .collapsible(false)
            .resizable(false)
            .show(ctx, |ui| {
                ui.label("Name");
                ui.text_edit_singleline(&mut self.save_as_name);
                ui.horizontal(|ui| {
                    if primary_button_with_icon(ui, Icon::Save, "Save", self.theme).clicked() {
                        save = true;
                    }
                    if secondary_button_with_icon(ui, Icon::X, "Cancel", self.theme).clicked() {
                        cancel = true;
                    }
                });
            });
        if cancel {
            self.save_as_open = false;
        } else if save {
            let name = self.save_as_name.trim().to_owned();
            if name.is_empty() {
                self.runtime_message = "Enter a name for the saved query".to_owned();
                return;
            }
            let document_index = self.active_query_document;
            if let Some(document) = self.query_documents.get_mut(document_index) {
                document.saved_query_id = None;
                document.title = name;
            }
            self.save_as_open = false;
            self.save_query_document_at(document_index);
        }
    }

    fn draw_dirty_close_dialog(&mut self, ctx: &egui::Context) {
        let Some(document_index) = self.pending_dirty_close else {
            return;
        };
        let Some(title) = self
            .query_documents
            .get(document_index)
            .map(|document| document.title.clone())
        else {
            self.pending_dirty_close = None;
            return;
        };
        let mut save = false;
        let mut discard = false;
        let mut cancel = false;
        egui::Window::new("Unsaved query")
            .collapsible(false)
            .resizable(false)
            .show(ctx, |ui| {
                ui.label(format!("Save changes to {title} before closing?"));
                ui.horizontal(|ui| {
                    if primary_button_with_icon(ui, Icon::Save, "Save", self.theme).clicked() {
                        save = true;
                    }
                    if secondary_button_with_icon(ui, Icon::Trash2, "Don't Save", self.theme).clicked() {
                        discard = true;
                    }
                    if secondary_button_with_icon(ui, Icon::X, "Cancel", self.theme).clicked() {
                        cancel = true;
                    }
                });
            });
        if cancel {
            self.pending_dirty_close = None;
        } else if discard {
            self.pending_dirty_close = None;
            self.close_query_document(document_index);
        } else if save {
            self.pending_dirty_close = None;
            self.pending_close_after_save = Some(document_index);
            self.save_query_document_at(document_index);
        }
    }

    /// Editor entries: find, font size, completion, snippets and folder creation.
    /// Returns true when the menu should close.
    fn draw_query_editor_actions(&mut self, ui: &mut egui::Ui) -> bool {
        let mut close_menu = false;
        if menu_button_with_icon(ui, Icon::Search, "Find in SQL", self.theme).clicked() {
            self.editor_search_open = !self.editor_search_open;
            close_menu = true;
        }
        if menu_button_with_icon(ui, Icon::Minus, "Decrease font size", self.theme).clicked() {
            self.editor_font_size = (self.editor_font_size - 1.0).max(10.0);
        }
        if menu_button_with_icon(ui, Icon::Plus, "Increase font size", self.theme).clicked() {
            self.editor_font_size = (self.editor_font_size + 1.0).min(24.0);
        }
        if menu_button_with_icon(ui, Icon::List, "SQL completion", self.theme).clicked() {
            self.completion_open = !self.completion_open;
            close_menu = true;
        }
        if menu_button_with_icon(ui, Icon::Bot, "Generate SQL Prediction", self.theme).clicked() {
            if self.prediction_mode != PredictionMode::Off {
                if let Some(doc) = self.query_documents.get_mut(self.active_query_document) {
                    doc.schedule_prediction_with_mode(Instant::now(), true);
                }
            }
            close_menu = true;
        }
        ui.horizontal(|ui| {
            ui.label(RichText::new("AI prediction").small().color(self.theme.text_muted));
            for (mode, label) in [
                (PredictionMode::Off, "Off"),
                (PredictionMode::Subtle, "Subtle"),
                (PredictionMode::Eager, "Eager"),
            ] {
                if ui.selectable_label(self.prediction_mode == mode, label).clicked() {
                    self.prediction_mode = mode;
                    if mode == PredictionMode::Off {
                        self.cancel_prediction_for_document(self.active_query_document);
                    }
                }
            }
        });
        ui.label(
            RichText::new(AI_PREDICTION_EGRESS_NOTE)
                .font(font_caption())
                .color(self.theme.text_muted),
        );
        ui.add_space(4.0);
        if menu_button_with_icon(ui, Icon::FileCode2, "SQL snippets", self.theme).clicked() {
            self.snippets_open = !self.snippets_open;
            close_menu = true;
        }
        ui.horizontal(|ui| {
            input(ui, &mut self.query_folder, "folder (optional)", 150.0, self.theme);
            if compact_button(ui, "New folder", self.theme).clicked() {
                self.create_query_folder();
            }
        });
        close_menu
    }

    /// Creates a saved-query folder from the name typed in the actions menu.
    fn create_query_folder(&mut self) {
        let Some(connection) = self.active_connection().cloned() else {
            return;
        };
        if self.query_folder.trim().is_empty() {
            return;
        }
        let request_id = self.task_bridge.next_request_id();
        self.dispatch_command(UiCommand::CreateQueryFolder {
            request_id,
            connection_id: connection.id.clone(),
            name: self.query_folder.trim().to_owned(),
        });
        self.runtime_message = "Creating query folder…".to_owned();
    }

    fn draw_query_editor(&mut self, ui: &mut egui::Ui) {
        let editor_width = ui.max_rect().width();
        let editor_height = if self.active_query_result().is_some() {
            ui.available_height().clamp(260.0, 360.0)
        } else {
            ui.available_height().clamp(320.0, 480.0)
        };

        if self.active_query_document >= self.query_documents.len() {
            return;
        }

        // Prefer numbered-parameter dialect heuristics when the capability set advertises
        // them; otherwise use the positional/SQLite editor dialect (covers SQLite + MySQL).
        let dialect = if self.query_capabilities().allows(|caps| caps.query.numbered_parameters) {
            SqlDialect::Postgres
        } else {
            SqlDialect::SQLite
        };
        let uses_positional_editor = !matches!(dialect, SqlDialect::Postgres);
        let active_schema = self.active_query_schema().to_owned();

        let theme = self.theme;
        let font_size = self.editor_font_size;
        let mut dispatch_statement = false;
        let mut dispatch_all = false;
        let mut trigger_completion = false;
        let mut manual_completion = false;
        let mut completion_pos = egui::Pos2::ZERO;

        let available_size = egui::vec2((editor_width - 24.0).max(280.0), (editor_height - 20.0).max(240.0));

        let doc_index = self.active_query_document;
        let doc = &mut self.query_documents[doc_index];

        let search_query = self.editor_search.clone();
        let is_completion_open = doc.completion.is_open;
        let execution_range = doc.executing_range;
        let previous_cursor = doc.cursor.offset;
        let previous_selection = doc.selection;
        let mut editor = SqlEditor::new(
            &mut doc.buffer,
            &mut doc.cursor,
            &mut doc.selection,
            dialect,
            &theme,
            &doc.diagnostics,
            doc.prediction.as_ref(),
            "active_sql_editor",
        )
        .with_cached_tokens(&mut doc.cached_tokens)
        .with_search(&search_query, doc.search.active_match_index)
        .with_completion_open(is_completion_open)
        .with_execution_range(execution_range)
        .with_prediction_visible(self.prediction_mode == PredictionMode::Eager || doc.prediction_reveal);
        editor.font_size = font_size;

        let response = editor.show(ui, available_size);

        let cursor_context_changed = previous_cursor != doc.cursor.offset || previous_selection != doc.selection;
        if cursor_context_changed {
            if let Some(request_id) = doc.pending_prediction_request {
                let _ = self.task_bridge.send(UiCommand::CancelSqlPrediction { request_id });
            }
            doc.invalidate_prediction();
        }

        self.query_editor_focused = response.focused;
        self.query_cursor_line = doc.cursor.line + 1;
        self.query_cursor_column = doc.cursor.col + 1;

        if let Some(accepted_len) = response.accepted_prediction_len {
            if let Some(pred) = doc.prediction.as_mut() {
                let was_partial = accepted_len < pred.text.len();
                pred.consume(accepted_len);
                doc.prediction_accepted = doc.prediction_accepted.saturating_add(1);
                if was_partial {
                    doc.prediction_partially_accepted = doc.prediction_partially_accepted.saturating_add(1);
                }
                if pred.is_empty() {
                    doc.prediction = None;
                }
            }
        }
        if response.wants_dismiss_prediction {
            doc.prediction = None;
        }

        if let Some(pred) = &doc.prediction {
            if pred.anchor != doc.cursor.offset {
                doc.prediction = None;
            }
        }

        doc.cached_tokens.get_or_recompute(&doc.buffer, dialect);
        let cursor_in_string_or_comment = doc
            .cached_tokens
            .is_in_string_or_comment(doc.cursor.offset.saturating_sub(1));

        if response.wants_format {
            if let Some(request_id) = doc.pending_prediction_request {
                // Cancellation is best effort; the document version guard remains authoritative.
                let _ = self.task_bridge.send(UiCommand::CancelSqlPrediction { request_id });
            }
            format_query_document(doc, dialect);
        }

        if response.changed {
            doc.reanalyze(dialect);
            doc.dirty = true;
            doc.execution_diagnostic = None;
            if !doc.selection.is_empty() {
                let (start, end) = doc.selection.normalized();
                self.selected_query = doc.buffer.slice(start, end).to_owned();
            } else {
                self.selected_query.clear();
            }
        }
        if response.wants_format {
            if !doc.selection.is_empty() {
                let (start, end) = doc.selection.normalized();
                self.selected_query = doc.buffer.slice(start, end).to_owned();
            } else {
                self.selected_query.clear();
            }
        }

        if response.wants_manual_prediction {
            doc.schedule_prediction_with_mode(Instant::now(), true);
        } else if (response.changed || cursor_context_changed)
            && !response.wants_dismiss_prediction
            && self.prediction_mode != PredictionMode::Off
            && doc.selection.is_empty()
            && !cursor_in_string_or_comment
        {
            doc.schedule_prediction(Instant::now());
        }

        if self.prediction_mode != PredictionMode::Off
            && doc.prediction_is_due(Instant::now())
            && doc.pending_prediction_request.is_none()
            && !doc.completion.is_open
            && doc.selection.is_empty()
            && !cursor_in_string_or_comment
        {
            doc.take_prediction_schedule();
            let manual = doc.take_prediction_manual();
            if let Some(scheduled_at) = doc.prediction_scheduled_at.take() {
                doc.prediction_last_debounce_ms = Some(scheduled_at.elapsed().as_millis() as u64);
            }
            let (before_cursor, after_cursor) = doc.buffer.split_at(doc.cursor.offset);
            let ai_context = SchemaCompletionProvider::build_ai_sql_context(
                before_cursor,
                after_cursor,
                doc.cursor.offset,
                &active_schema,
                &self.schema,
                uses_positional_editor,
            );
            let document_version = doc.buffer.version();
            let anchor = doc.cursor.offset;
            let fingerprint = ai_context.fingerprint(document_version, anchor);
            if !manual {
                if let Some(cached) = doc.take_cached_prediction(fingerprint, Instant::now()) {
                    doc.prediction_context_fingerprint = Some(fingerprint);
                    doc.prediction = Some(cached);
                }
            }
            if doc.prediction.is_none() && !manual && doc.should_dedupe_prediction(fingerprint, Instant::now()) {
                doc.prediction_requests_deduped = doc.prediction_requests_deduped.saturating_add(1);
            } else if doc.prediction.is_none() {
                let req_id = self.task_bridge.next_request_id();
                doc.prediction_context_fingerprint = Some(fingerprint);
                doc.prediction_last_request_fingerprint = Some(fingerprint);
                doc.prediction_last_request_at = Some(Instant::now());
                doc.pending_prediction_request = Some(req_id);
                doc.prediction_request_started_at = Some(Instant::now());
                doc.prediction_requests_sent = doc.prediction_requests_sent.saturating_add(1);
                let replacement_range = prediction_replacement_range(&doc.buffer, anchor, manual);
                let _ = self.task_bridge.send(UiCommand::RequestSqlPrediction {
                    request_id: req_id,
                    document_id: doc.id.clone(),
                    document_version,
                    anchor,
                    replacement_range,
                    context: ai_context,
                });
            }
        }

        if response.wants_execute_statement {
            dispatch_statement = true;
        } else if response.wants_execute_all {
            dispatch_all = true;
        }

        if response.wants_completion {
            trigger_completion = true;
            manual_completion = response.wants_manual_completion;
            completion_pos = response.cursor_screen_pos;
        }

        if trigger_completion {
            let (before_cursor, after_cursor) = doc.buffer.split_at(doc.cursor.offset);
            let ctx = CompletionContext {
                text_before_cursor: before_cursor,
                text_after_cursor: after_cursor,
                cursor_offset: doc.cursor.offset,
                active_schema: &active_schema,
                schema_summary: &self.schema,
                cached_tokens: Some(&doc.cached_tokens),
                is_sqlite: uses_positional_editor,
                is_manual_trigger: manual_completion,
            };
            let (prefix, items) = SchemaCompletionProvider::provide(&ctx);
            if !items.is_empty() {
                doc.completion.open(
                    doc.cursor.offset,
                    completion_pos,
                    prefix,
                    items,
                    CompletionTriggerKind::Automatic,
                );
            }
        }

        if dispatch_statement {
            self.dispatch_query();
        } else if dispatch_all {
            self.dispatch_query_all();
        }
    }

    fn draw_floating_completion_popup(&mut self, ctx: &egui::Context) {
        let dialect = if self.query_capabilities().allows(|caps| caps.query.numbered_parameters) {
            SqlDialect::Postgres
        } else {
            SqlDialect::SQLite
        };
        let theme = self.theme;

        let Some(doc) = self.query_documents.get_mut(self.active_query_document) else {
            return;
        };

        if !doc.completion.is_open || doc.completion.items.is_empty() {
            return;
        }

        let mut apply_item = None;
        let mut close_popup = false;

        ctx.input(|i| {
            for event in &i.events {
                if let egui::Event::Key { key, pressed: true, .. } = event {
                    match key {
                        egui::Key::ArrowUp => {
                            doc.completion.select_prev();
                        }
                        egui::Key::ArrowDown => {
                            doc.completion.select_next();
                        }
                        egui::Key::PageUp => {
                            doc.completion.select_page_up(5);
                        }
                        egui::Key::PageDown => {
                            doc.completion.select_page_down(5);
                        }
                        egui::Key::Enter | egui::Key::Tab => {
                            if let Some(item) = doc.completion.current_item() {
                                apply_item = Some(item.clone());
                            }
                        }
                        egui::Key::Escape => {
                            close_popup = true;
                        }
                        _ => {}
                    }
                }
            }
        });

        if let Some(item) = apply_item {
            let (start, end) = item.replacement_range;
            doc.buffer.replace(start, end, &item.insert_text);
            let new_offset = start + item.insert_text.len();
            doc.cursor.set_offset(&doc.buffer, new_offset);
            doc.selection.collapse_to_active();
            doc.reanalyze(dialect);
            doc.dirty = true;
            doc.prediction = None;
            doc.completion.close();
            return;
        }

        if close_popup {
            doc.completion.close();
            return;
        }

        let screen_rect = ctx.screen_rect();
        let mut popup_pos = doc.completion.popup_position;
        let popup_height = 220.0;
        let popup_width = 340.0;

        // Auto-flip popup above cursor if near bottom of screen
        if popup_pos.y + popup_height > screen_rect.max.y - 30.0 {
            popup_pos.y = (popup_pos.y - popup_height - 24.0).max(screen_rect.min.y + 10.0);
        }
        popup_pos.x = popup_pos.x.clamp(
            screen_rect.min.x + 10.0,
            (screen_rect.max.x - popup_width - 20.0).max(screen_rect.min.x + 10.0),
        );

        let mut clicked_item = None;

        let area_resp = egui::Area::new(egui::Id::new("floating_completion_popup"))
            .order(egui::Order::Foreground)
            .fixed_pos(popup_pos)
            .show(ctx, |ui| {
                egui::Frame {
                    fill: theme.surface_floating,
                    rounding: egui::Rounding::same(6.0),
                    stroke: egui::Stroke::new(1.0, theme.border_default),
                    shadow: theme.floating_shadow(),
                    inner_margin: egui::Margin::same(6.0),
                    ..Default::default()
                }
                .show(ui, |ui| {
                    ui.set_max_width(popup_width);
                    ui.set_max_height(popup_height);

                    egui::ScrollArea::vertical().max_height(200.0).show(ui, |ui| {
                        let items = doc.completion.items.clone();
                        let sel_idx = doc.completion.selected_index;

                        for (idx, item) in items.iter().enumerate() {
                            let is_selected = idx == sel_idx;
                            let bg = if is_selected {
                                theme.surface_active
                            } else {
                                egui::Color32::TRANSPARENT
                            };

                            let item_frame = egui::Frame::none()
                                .fill(bg)
                                .rounding(egui::Rounding::same(4.0))
                                .inner_margin(egui::Margin::symmetric(6.0, 3.0))
                                .show(ui, |ui| {
                                    ui.horizontal(|ui| {
                                        let (badge_text, badge_color) = match item.kind {
                                            CompletionItemKind::Keyword => ("KEY", theme.code_keyword),
                                            CompletionItemKind::Table => ("TBL", theme.accent),
                                            CompletionItemKind::View => ("VIEW", theme.info),
                                            CompletionItemKind::Column => ("COL", theme.code_variable),
                                            CompletionItemKind::Function => ("FN", theme.code_function),
                                            CompletionItemKind::Schema => ("SCH", theme.warning),
                                            CompletionItemKind::Snippet => ("SNP", theme.success),
                                            CompletionItemKind::Cte => ("CTE", theme.code_type),
                                        };

                                        ui.label(
                                            RichText::new(badge_text)
                                                .font(FontId::monospace(9.5))
                                                .color(badge_color),
                                        );
                                        ui.add_space(4.0);

                                        ui.label(
                                            RichText::new(&item.label)
                                                .font(FontId::monospace(12.5))
                                                .strong()
                                                .color(theme.text_primary),
                                        );

                                        if let Some(detail) = &item.detail {
                                            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                                ui.label(
                                                    RichText::new(detail).font(font_caption()).color(theme.text_muted),
                                                );
                                            });
                                        }
                                    });
                                });

                            if is_selected {
                                item_frame.response.scroll_to_me(Some(egui::Align::Center));
                            }
                            if item_frame.response.hovered() {
                                doc.completion.selected_index = idx;
                            }
                            if item_frame.response.interact(egui::Sense::click()).clicked() {
                                clicked_item = Some(item.clone());
                            }
                        }
                    });
                });
            });

        if let Some(item) = clicked_item {
            let (start, end) = item.replacement_range;
            doc.buffer.replace(start, end, &item.insert_text);
            let new_offset = start + item.insert_text.len();
            doc.cursor.set_offset(&doc.buffer, new_offset);
            doc.selection.collapse_to_active();
            doc.reanalyze(dialect);
            doc.dirty = true;
            doc.prediction = None;
            doc.completion.close();
        }

        // Close on click outside
        let clicked_outside = ctx.input(|i| {
            i.pointer.any_click()
                && i.pointer
                    .interact_pos()
                    .is_some_and(|pos| !area_resp.response.rect.contains(pos))
        });
        if clicked_outside {
            doc.completion.close();
        }
    }

    /// Confirmation gate for statements the safety classifier rates `Destructive`.
    ///
    /// The editor deliberately allows arbitrary SQL; what it must not do is send a
    /// statement that can drop or truncate data without the user seeing the exact text
    /// that is about to run. The statement is held (not refused) and dispatched by
    /// `confirm_pending_destructive_run`.
    fn draw_destructive_run_dialog(&mut self, ui: &mut egui::Ui) {
        let Some(pending) = self.pending_destructive_run.clone() else {
            return;
        };
        const PREVIEW_CHARS: usize = 600;
        let mut open = true;
        let mut confirmed = false;
        let mut cancelled = false;
        Dialog::new(&mut open, "Run Destructive Statement?", self.theme)
            .id_salt("destructive_run_dialog")
            .width(560.0)
            .show(ui, |ui| {
                ui.label(
                    RichText::new(if pending.all_statements {
                        "The script you are about to run contains a statement that can drop or truncate data. Nothing has been sent yet."
                    } else {
                        "This statement can drop or truncate data. Nothing has been sent yet."
                    })
                    .color(self.theme.text_primary),
                );
                ui.add_space(SPACE_SM);
                let mut preview = pending.sql.clone();
                if preview.chars().count() > PREVIEW_CHARS {
                    preview = preview.chars().take(PREVIEW_CHARS).collect::<String>() + "…";
                }
                editor_frame(self.theme).show(ui, |ui| {
                    ui.label(RichText::new(preview).font(font_mono_sm()).color(self.theme.text_secondary));
                });
                ui.add_space(SPACE_SM);
                ui.colored_label(
                    self.theme.warning,
                    "It is sent to the server exactly as written; the app cannot undo it.",
                );
                ui.add_space(SPACE_MD);
                ui.horizontal(|ui| {
                    if danger_button(ui, "Run Destructive Statement", self.theme).clicked() {
                        confirmed = true;
                    }
                    if compact_button(ui, "Cancel", self.theme).clicked() {
                        cancelled = true;
                    }
                });
            });
        if confirmed {
            self.confirm_pending_destructive_run();
        } else if cancelled || !open {
            self.cancel_pending_destructive_run();
        }
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
                    self.export_overwrite_pending = false;
                }
            });
            if self.export_overwrite_pending {
                ui.add_space(SPACE_XS);
                ui.colored_label(
                    self.theme.warning,
                    format!("{} already exists. Overwrite it?", self.export_path.trim()),
                );
                ui.horizontal(|ui| {
                    if danger_button(ui, "Overwrite", self.theme).clicked() {
                        if let Some(result) = result {
                            self.export_result_confirming_overwrite(result);
                        }
                    }
                    if compact_button(ui, "Keep existing file", self.theme).clicked() {
                        self.export_overwrite_pending = false;
                    }
                });
            }
        });
    }

    pub(crate) fn explain_query(&mut self) {
        if self.active_explain_request().is_some() {
            return;
        }
        let lookup = self.query_capabilities();
        if let Some(reason) = lookup.feature_limitation(db_pro_core::domain::capabilities::CapabilityFeature::Explain) {
            self.runtime_message = format!("Explain is unavailable: {reason}");
            return;
        }
        let Some(connection_id) = self.active_query_connection_id().map(str::to_owned) else {
            self.runtime_message = "Connect to a database before explaining a query".to_owned();
            return;
        };
        let sql = if self.selected_query.trim().is_empty() {
            self.active_query_text().trim().to_owned()
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
            let doc_index = self.active_query_document;
            if let Some(doc) = self.query_documents.get_mut(doc_index) {
                doc.explain_request = Some(request_id);
                doc.explain_plan = None;
            }
            self.set_active_query_output_tab(OutputTab::Explain);
            self.runtime_message = "Explaining query…".to_owned();
        }
    }

    pub(crate) fn export_result(&mut self, result: &UiQueryResult) {
        self.export_result_to_disk(result, false);
    }

    /// Write the export after the user confirmed overwriting an existing file (#244, E-1).
    pub(crate) fn export_result_confirming_overwrite(&mut self, result: &UiQueryResult) {
        self.export_result_to_disk(result, true);
    }

    /// Export the visible result to `export_path`.
    ///
    /// #244 (E-1): the file is published atomically and an existing path is never replaced without
    /// asking, so a failed export cannot leave a truncated file that looks complete and a typo in the
    /// path cannot destroy an unrelated file.
    fn export_result_to_disk(&mut self, result: &UiQueryResult, overwrite: bool) {
        let path_text = self.export_path.trim().to_owned();
        if path_text.is_empty() {
            self.runtime_message = "Choose an export path first".to_owned();
            return;
        }
        let path = PathBuf::from(&path_text);
        if path.exists() && !overwrite {
            self.export_overwrite_pending = true;
            return;
        }

        let delimiter = if self.export_format == "CSV" { "," } else { "\t" };
        let output = DbProApp::format_result_delimited(result, delimiter);
        let exported_rows = result.rows.len();
        match write_file_atomically(&path, output.as_bytes()) {
            Ok(()) => {
                // Truthful row count: a capped result exports the rows it holds, not the rows the
                // query matched, and the message has to say which one it is.
                self.runtime_message = if result.row_count > exported_rows as u64 {
                    format!(
                        "Exported {exported_rows} rows to {path_text} ({} rows matched; the result holds the first {exported_rows})",
                        result.row_count
                    )
                } else {
                    format!("Exported {exported_rows} rows to {path_text}")
                };
            }
            Err(error) => {
                self.runtime_message = format!("Export failed: {error} (no file was written)");
            }
        }
        self.export_overwrite_pending = false;
        self.export_open = false;
    }

    pub(crate) fn format_active_query(&mut self) {
        let doc_index = self.active_query_document;
        self.cancel_prediction_for_document(doc_index);
        let dialect = if self.query_capabilities().allows(|caps| caps.query.numbered_parameters) {
            SqlDialect::Postgres
        } else {
            SqlDialect::SQLite
        };
        if let Some(doc) = self.query_documents.get_mut(doc_index) {
            format_query_document(doc, dialect);
        }
    }

    pub(crate) fn analyze_sql_diagnostics(sql: &str, driver: &str) -> (Vec<String>, Vec<Diagnostic>) {
        let mut string_diagnostics = Vec::new();
        let mut structured_diagnostics = Vec::new();
        let capabilities = match CapabilityLookup::for_driver_label(driver) {
            CapabilityLookup::Supported(caps) => Some(caps),
            CapabilityLookup::NoActiveConnection | CapabilityLookup::UnsupportedDriver { .. } => None,
        };
        let parse_result = if capabilities.as_ref().is_some_and(|caps| caps.query.numbered_parameters) {
            Parser::parse_sql(&PostgreSqlDialect {}, sql)
        } else if driver.eq_ignore_ascii_case("mysql") {
            // MySQL shares the positional editor dialect; GenericDialect is the closest
            // sqlparser stand-in until a dedicated MySQL dialect is wired.
            Parser::parse_sql(&GenericDialect {}, sql)
        } else {
            Parser::parse_sql(&SQLiteDialect {}, sql)
        };
        if let Err(error) = parse_result {
            let msg = format!("SQL parser: {error}");
            string_diagnostics.push(msg.clone());
            structured_diagnostics.push(Diagnostic::error((0, sql.len().clamp(1, 4)), msg));
        }
        if sql.trim().is_empty() {
            string_diagnostics.push("Query is empty".to_owned());
            return (string_diagnostics, structured_diagnostics);
        }
        for issue in crate::editor::brackets::structural_delimiter_issues(sql) {
            let end = issue.offset + sql[issue.offset..].chars().next().map_or(1, char::len_utf8);
            let message = if let Some(expected) = issue.expected {
                format!("Mismatched delimiter {}: expected {}", issue.character, expected)
            } else {
                format!("Unmatched delimiter {}", issue.character)
            };
            string_diagnostics.push(message.clone());
            structured_diagnostics.push(Diagnostic::delimiter((issue.offset, end), message));
        }
        Self::append_sql_lint_diagnostics(sql, &mut string_diagnostics, &mut structured_diagnostics);
        let mut tokens = Vec::new();
        let mut current = String::new();
        let mut in_string = false;
        let mut string_start_byte = 0;

        for (byte_offset, ch) in sql.char_indices() {
            if ch == '\'' {
                if !in_string {
                    in_string = true;
                    string_start_byte = byte_offset;
                } else {
                    in_string = false;
                }
                current.push(ch);
            } else if in_string {
                current.push(ch);
            } else if matches!(ch, '(' | ')') {
                tokens.push((current.to_lowercase(), byte_offset));
                current.clear();
            } else if ch.is_whitespace() || ch == ';' || ch == ',' {
                if !current.is_empty() {
                    tokens.push((current.to_lowercase(), byte_offset - current.len()));
                    current.clear();
                }
            } else {
                current.push(ch);
            }
        }
        if !current.is_empty() {
            tokens.push((current.to_lowercase(), sql.len() - current.len()));
        }
        if in_string {
            let msg = "Unclosed string literal".to_owned();
            string_diagnostics.push(msg.clone());
            structured_diagnostics.push(Diagnostic::error((string_start_byte, sql.len()), msg));
        }
        if tokens.first().map(|(t, _)| t.as_str()) == Some("update") && !tokens.iter().any(|(t, _)| t == "where") {
            let msg = "UPDATE without WHERE will affect every row".to_owned();
            string_diagnostics.push(msg.clone());
            structured_diagnostics.push(Diagnostic::warning((0, sql.len()), msg));
        }
        if !capabilities.as_ref().is_some_and(|caps| caps.query.ilike) {
            if let Some((_, offset)) = tokens.iter().find(|(t, _)| t == "ilike") {
                let msg = "ILIKE is not supported for this provider; use LIKE or lower()".to_owned();
                string_diagnostics.push(msg.clone());
                structured_diagnostics.push(Diagnostic::error((*offset, offset + 5), msg));
            }
        }
        if !capabilities.as_ref().is_some_and(|caps| caps.query.glob) {
            if let Some((_, offset)) = tokens.iter().find(|(t, _)| t == "glob") {
                let msg = "GLOB is not supported for this provider; use LIKE instead".to_owned();
                string_diagnostics.push(msg.clone());
                structured_diagnostics.push(Diagnostic::error((*offset, offset + 4), msg));
            }
        }
        let lower = sql.to_lowercase();
        if lower.contains("select * from") && lower.contains("select * from select") {
            let msg = "Subquery must be enclosed in parentheses".to_owned();
            string_diagnostics.push(msg.clone());
            structured_diagnostics.push(Diagnostic::error((0, sql.len()), msg));
        }

        (
            deduplicate_messages(string_diagnostics),
            deduplicate_diagnostics(structured_diagnostics),
        )
    }

    /// Local, explainable lint rules (issue #257). Never requires schema or network.
    fn append_sql_lint_diagnostics(
        sql: &str,
        string_diagnostics: &mut Vec<String>,
        structured_diagnostics: &mut Vec<Diagnostic>,
    ) {
        let lower = sql.to_lowercase();
        // SELECT * — warn on the star token when it is a projection wildcard.
        if let Some(star_at) = lower.find("select") {
            let after = &lower[star_at..];
            if let Some(rel) = after.find('*') {
                let abs = star_at + rel;
                let before_ok = after[..rel].chars().rev().find(|c| !c.is_whitespace()).is_some();
                let from_follows = after[rel..].contains("from");
                if before_ok && from_follows {
                    let msg = "SELECT * makes column contracts brittle; prefer an explicit column list".to_owned();
                    string_diagnostics.push(msg.clone());
                    structured_diagnostics.push(Diagnostic::lint((abs, abs + 1), msg, "lint.select-star"));
                }
            }
        }
        // = NULL / != NULL / <> NULL — always unknown in SQL; suggest IS [NOT] NULL.
        for (needle, suggestion) in [
            ("= null", "IS NULL"),
            ("!= null", "IS NOT NULL"),
            ("<> null", "IS NOT NULL"),
            ("=null", "IS NULL"),
            ("!=null", "IS NOT NULL"),
            ("<>null", "IS NOT NULL"),
        ] {
            if let Some(at) = lower.find(needle) {
                let msg = format!("Comparing with NULL using {needle} is always unknown; use {suggestion}");
                string_diagnostics.push(msg.clone());
                structured_diagnostics.push(Diagnostic::lint_with_fix(
                    (at, at + needle.len()),
                    msg,
                    "lint.null-compare",
                    suggestion,
                ));
            }
        }
        // DELETE / UPDATE without WHERE already warned above for UPDATE; cover DELETE.
        let trimmed = lower.trim_start();
        if trimmed.starts_with("delete") && !lower.contains("where") {
            let msg = "DELETE without WHERE will remove every row".to_owned();
            string_diagnostics.push(msg.clone());
            structured_diagnostics.push(Diagnostic::lint((0, sql.len().min(6)), msg, "lint.delete-no-where"));
        }
        // ORDER BY n — positional ordinals are brittle across projection changes.
        if let Some(order_at) = lower.find("order by") {
            let after = &lower[order_at + "order by".len()..];
            let trimmed_after = after.trim_start();
            let skip = after.len() - trimmed_after.len();
            if let Some(first) = trimmed_after.chars().next() {
                if first.is_ascii_digit() {
                    let abs = order_at + "order by".len() + skip;
                    let end = abs
                        + trimmed_after
                            .chars()
                            .take_while(|c| c.is_ascii_digit() || *c == ',' || c.is_whitespace())
                            .map(char::len_utf8)
                            .sum::<usize>();
                    let msg = "ORDER BY ordinal is brittle; prefer an explicit column or expression".to_owned();
                    string_diagnostics.push(msg.clone());
                    structured_diagnostics.push(Diagnostic::lint(
                        (abs, end.max(abs + 1)),
                        msg,
                        "lint.order-by-ordinal",
                    ));
                }
            }
        }
        // FROM a, b — classic comma join / cartesian-product pattern when JOIN is absent.
        if let Some(from_at) = lower.find("from") {
            let after_from = &lower[from_at + 4..];
            let has_join = after_from.contains(" join ")
                || after_from.contains(" join\n")
                || after_from.contains("\njoin ")
                || after_from.starts_with("join ")
                || after_from.contains(" join(");
            if !has_join {
                if let Some(comma_rel) = after_from.find(',') {
                    let between = after_from[..comma_rel].trim();
                    let after_comma = after_from[comma_rel + 1..].trim_start();
                    let looks_like_table = !between.is_empty()
                        && after_comma
                            .chars()
                            .next()
                            .is_some_and(|c| c.is_alphabetic() || c == '"');
                    if looks_like_table {
                        let abs = from_at + 4 + comma_rel;
                        let msg = "Comma join may produce a cartesian product; prefer explicit JOIN … ON".to_owned();
                        string_diagnostics.push(msg.clone());
                        structured_diagnostics.push(Diagnostic::lint((abs, abs + 1), msg, "lint.comma-join"));
                    }
                }
            }
        }
        // Duplicate projection aliases: `SELECT a AS x, b AS x`.
        if let Some(select_at) = lower.find("select") {
            let after_select = &lower[select_at + "select".len()..];
            let projection = after_select.split(" from ").next().unwrap_or(after_select);
            let mut seen: Vec<(String, usize)> = Vec::new();
            let mut search_from = 0usize;
            while let Some(rel) = projection[search_from..].find(" as ") {
                let abs_in_proj = search_from + rel + " as ".len();
                let alias_slice = projection[abs_in_proj..].trim_start();
                let skip = projection[abs_in_proj..].len() - alias_slice.len();
                let alias: String = alias_slice
                    .chars()
                    .take_while(|c| c.is_alphanumeric() || *c == '_' || *c == '"')
                    .collect();
                if !alias.is_empty() {
                    let alias_key = alias.trim_matches('"').to_ascii_lowercase();
                    let abs = select_at + "select".len() + abs_in_proj + skip;
                    if let Some((_, first_at)) = seen.iter().find(|(name, _)| name == &alias_key) {
                        let msg = format!("Duplicate projection alias `{alias_key}`");
                        string_diagnostics.push(msg.clone());
                        structured_diagnostics.push(Diagnostic::lint(
                            (abs, abs + alias.len()),
                            msg,
                            "lint.duplicate-alias",
                        ));
                        let _ = first_at;
                    } else {
                        seen.push((alias_key, abs));
                    }
                }
                search_from = abs_in_proj + alias.len().max(1);
            }
        }
    }

    pub(crate) fn parse_sql_diagnostics(sql: &str, driver: &str) -> Vec<String> {
        Self::analyze_sql_diagnostics(sql, driver).0
    }

    pub(crate) fn refresh_diagnostics(&mut self) {
        let driver = self.active_driver().to_owned();
        let doc_index = self.active_query_document;
        if let Some(doc) = self.query_documents.get_mut(doc_index) {
            let (raw_diags, structured) = Self::analyze_sql_diagnostics(doc.text(), &driver);
            self.diagnostics = raw_diags;
            doc.diagnostics =
                deduplicate_diagnostics(structured.into_iter().chain(doc.execution_diagnostic.clone()).collect());
        } else {
            self.diagnostics = Self::parse_sql_diagnostics(self.active_query_text(), &driver);
        }
    }

    pub(crate) fn insert_snippet(&mut self, snippet: &str) {
        self.cancel_prediction_for_document(self.active_query_document);
        if let Some(doc) = self.query_documents.get_mut(self.active_query_document) {
            let offset = doc.cursor.offset.min(doc.buffer.len_bytes());
            let insertion = if offset > 0 && !doc.buffer.text()[..offset].ends_with('\n') {
                format!("\n{snippet}")
            } else {
                snippet.to_owned()
            };
            doc.buffer.insert(offset, &insertion);
            let new_offset = offset + insertion.len();
            doc.cursor = crate::editor::CursorPosition::from_offset(&doc.buffer, new_offset);
            doc.selection = crate::editor::SelectionRange::point(new_offset);
            doc.dirty = true;
            self.query_cursor_line = doc.cursor.line + 1;
            self.query_cursor_column = doc.cursor.col + 1;
        }
        self.active_tab = WorkspaceTab::Query;
        self.refresh_diagnostics();
        self.runtime_message = "Snippet inserted".to_owned();
    }
}

fn deduplicate_messages(messages: Vec<String>) -> Vec<String> {
    let mut unique = Vec::with_capacity(messages.len());
    for message in messages {
        if !unique.iter().any(|existing| existing == &message) {
            unique.push(message);
        }
    }
    unique
}

fn deduplicate_diagnostics(diagnostics: Vec<Diagnostic>) -> Vec<Diagnostic> {
    let mut unique = Vec::with_capacity(diagnostics.len());
    for diagnostic in diagnostics {
        let duplicate = unique.iter().any(|existing: &Diagnostic| {
            existing.source == diagnostic.source
                && existing.message == diagnostic.message
                && ranges_overlap(existing.range, diagnostic.range)
        });
        if !duplicate {
            unique.push(diagnostic);
        }
    }
    unique
}

pub(crate) fn database_error_diagnostic(
    message: &str,
    executed_sql: &str,
    executed_range: (usize, usize),
    position: Option<usize>,
    code: Option<&str>,
) -> Option<Diagnostic> {
    if executed_sql.is_empty() || executed_range.0 >= executed_range.1 {
        return None;
    }
    let (start, end) = position
        .filter(|position| *position > 0)
        .map(|position| char_position_to_byte_offset(executed_sql, position))
        .filter(|start| *start < executed_sql.len())
        .and_then(|start| {
            executed_sql[start..]
                .chars()
                .next()
                .map(|character| (start, start + character.len_utf8()))
        })
        .unwrap_or((0, executed_sql.len()));
    let local_limit = executed_range.1 - executed_range.0;
    let document_start = executed_range.0 + start.min(local_limit);
    let document_end = (executed_range.0 + end)
        .min(executed_range.1)
        .max((document_start + 1).min(executed_range.1));
    Some(match code {
        Some(code) => Diagnostic::database_with_code((document_start, document_end), message, code),
        None => Diagnostic::database((document_start, document_end), message),
    })
}

fn char_position_to_byte_offset(sql: &str, position: usize) -> usize {
    let index = position.saturating_sub(1);
    sql.char_indices()
        .nth(index)
        .map(|(offset, _)| offset)
        .unwrap_or(sql.len())
}

fn ranges_overlap(left: (usize, usize), right: (usize, usize)) -> bool {
    left.0 < right.1 && right.0 < left.1
}

fn format_query_document(doc: &mut QueryDocument, dialect: SqlDialect) {
    let original_selection = doc.selection;
    let had_selection = !original_selection.is_empty();
    let (start, end) = original_selection.normalized();
    let original_text = doc.buffer.slice(start, end).to_owned();
    let formatted_text = crate::query::sql_format::format_sql_for_dialect(&original_text, dialect);
    if formatted_text == original_text {
        return;
    }

    let cursor_after = if had_selection {
        if original_selection.active >= original_selection.anchor {
            start + formatted_text.len()
        } else {
            start
        }
    } else {
        start + doc.cursor.offset.saturating_sub(start).min(formatted_text.len())
    };
    let anchor_after = if had_selection {
        if original_selection.anchor <= original_selection.active {
            start
        } else {
            start + formatted_text.len()
        }
    } else {
        cursor_after
    };

    doc.buffer.replace_with_snapshot(
        start,
        end,
        &formatted_text,
        EditorSnapshot {
            cursor_offset: doc.cursor.offset,
            anchor_offset: original_selection.anchor,
        },
        EditorSnapshot {
            cursor_offset: cursor_after,
            anchor_offset: anchor_after,
        },
    );
    doc.cursor.set_offset(&doc.buffer, cursor_after);
    doc.selection = SelectionRange::new(anchor_after, cursor_after);
    doc.completion.clear();
    doc.invalidate_prediction();
    doc.reanalyze(dialect);
    doc.search.update_matches(doc.buffer.text());
    doc.dirty = true;
}

fn prediction_replacement_range(
    buffer: &crate::editor::buffer::TextBuffer,
    anchor: usize,
    manual: bool,
) -> (usize, usize) {
    if !manual {
        return (anchor, anchor);
    }
    let before = buffer.slice(0, anchor);
    let start = before
        .char_indices()
        .rev()
        .find(|(_, ch)| !ch.is_ascii_alphanumeric() && *ch != '_')
        .map_or(0, |(offset, ch)| offset + ch.len_utf8());
    (start, anchor)
}

#[cfg(test)]
mod egress_tests {
    use super::*;

    /// Every text run the frame actually painted, with the editor actions menu open.
    fn rendered_editor_actions_texts(app: &mut DbProApp) -> Vec<String> {
        fn collect(shape: &egui::Shape, texts: &mut Vec<String>) {
            match shape {
                egui::Shape::Text(text) => texts.push(text.galley.text().to_owned()),
                egui::Shape::Vec(shapes) => {
                    for shape in shapes {
                        collect(shape, texts);
                    }
                }
                _ => {}
            }
        }

        let ctx = egui::Context::default();
        DbProTheme::install_fonts(&ctx);
        let output = ctx.run(Default::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                let _ = app.draw_query_editor_actions(ui);
            });
        });

        let mut texts = Vec::new();
        for clipped in &output.shapes {
            collect(&clipped.shape, &mut texts);
        }
        texts
    }

    #[test]
    fn ai_prediction_control_discloses_its_egress() {
        let mut app = DbProApp::default();

        let texts = rendered_editor_actions_texts(&mut app);

        assert!(
            texts.iter().any(|text| text == AI_PREDICTION_EGRESS_NOTE),
            "the AI prediction control must state that it sends SQL and schema context (#242); painted texts: {texts:?}"
        );
        assert!(
            texts.iter().any(|text| text == "AI prediction"),
            "the prediction mode control itself is still painted; painted texts: {texts:?}"
        );
    }

    #[test]
    fn prediction_default_stays_eager_with_the_note_visible() {
        // #242 item 3: the decision is "keep Eager", so the note is the disclosure that makes the
        // default an informed one. If the default ever changes, this test fails on purpose.
        let app = DbProApp::default();

        assert_eq!(app.prediction_mode, PredictionMode::Eager);
        assert!(AI_PREDICTION_EGRESS_NOTE.contains("configured AI provider"));
    }
}

#[cfg(test)]
mod export_tests {
    use super::*;
    #[cfg(unix)]
    use std::os::unix::fs::PermissionsExt;

    fn temp_dir(label: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("db-pro-export-{label}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).expect("create temp dir");
        dir
    }

    fn dir_entries(dir: &std::path::Path) -> Vec<String> {
        let mut names: Vec<String> = std::fs::read_dir(dir)
            .expect("read temp dir")
            .map(|entry| entry.expect("entry").file_name().to_string_lossy().into_owned())
            .collect();
        names.sort();
        names
    }

    #[test]
    fn atomic_write_publishes_the_whole_file_and_leaves_no_scratch_file() {
        let dir = temp_dir("publish");
        let path = dir.join("out.csv");

        write_file_atomically(&path, b"id,name\n1,alpha\n").expect("write");

        assert_eq!(std::fs::read_to_string(&path).expect("read"), "id,name\n1,alpha\n");
        assert_eq!(
            dir_entries(&dir),
            vec!["out.csv".to_owned()],
            "the temp file must not survive a successful write"
        );
    }

    /// The regression #244 (E-1) describes: a write that cannot complete must leave **no** truncated
    /// file at the destination. The directory is made read-only so the temp file cannot be created —
    /// the destination is then untouched, where `std::fs::write` would have failed mid-stream.
    #[cfg(unix)]
    #[test]
    fn failed_atomic_write_leaves_the_destination_untouched() {
        let dir = temp_dir("failure");
        let path = dir.join("out.csv");
        std::fs::write(&path, "previous export\n").expect("seed existing file");

        std::fs::set_permissions(&dir, std::fs::Permissions::from_mode(0o500)).expect("chmod read-only");
        let error = write_file_atomically(&path, b"id,name\n1,alpha\n")
            .expect_err("a write into a read-only directory must fail");
        std::fs::set_permissions(&dir, std::fs::Permissions::from_mode(0o700)).expect("chmod back");

        assert_eq!(error.kind(), std::io::ErrorKind::PermissionDenied);
        assert_eq!(
            std::fs::read_to_string(&path).expect("read"),
            "previous export\n",
            "the destination must keep its previous content, never a truncated prefix"
        );
        assert_eq!(
            dir_entries(&dir),
            vec!["out.csv".to_owned()],
            "no scratch file may survive"
        );
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn failed_atomic_write_into_a_missing_directory_creates_nothing() {
        let dir = temp_dir("missing");
        let path = dir.join("no-such-dir").join("out.csv");

        write_file_atomically(&path, b"id,name\n").expect_err("the parent does not exist");

        assert!(!path.exists());
        assert_eq!(dir_entries(&dir), Vec::<String>::new(), "nothing may be created");
        let _ = std::fs::remove_dir_all(&dir);
    }

    /// Pins that the **export path** publishes rather than rewrites in place — the property the
    /// acceptance cares about ("no partial file survives"), which a test of the helper alone does not
    /// establish: reverting `export_result_to_disk` to `std::fs::write` leaves a helper-only suite
    /// green. An in-place rewrite truncates the destination before writing into it; a publish
    /// replaces it, so the destination is a different file afterwards even though the path is the
    /// same.
    #[cfg(unix)]
    #[test]
    fn export_replaces_the_destination_instead_of_truncating_it() {
        use std::os::unix::fs::MetadataExt as _;

        let dir = temp_dir("publish-semantics");
        let path = dir.join("out.csv");
        std::fs::write(&path, "previous export\n").expect("seed existing file");
        let inode_before = std::fs::metadata(&path).expect("metadata").ino();

        let mut app = DbProApp {
            export_open: true,
            export_path: path.to_string_lossy().into_owned(),
            export_format: "CSV".to_owned(),
            ..Default::default()
        };
        let result = UiQueryResult {
            columns: vec![crate::UiColumn {
                name: "id".to_owned(),
                data_type: "int".to_owned(),
                nullable: false,
            }],
            rows: vec![vec![UiCell::Number("1".to_owned())]],
            row_count: 1,
            duration_ms: 0,
        };

        app.export_result(&result);
        app.export_result_confirming_overwrite(&result);

        assert_eq!(std::fs::read_to_string(&path).expect("read"), "id\n1\n");
        assert_ne!(
            std::fs::metadata(&path).expect("metadata").ino(),
            inode_before,
            "the destination must be replaced by a rename, not rewritten in place: an in-place write \
             is what leaves a truncated file when it fails part-way"
        );
        assert_eq!(dir_entries(&dir), vec!["out.csv".to_owned()]);
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn export_refuses_to_replace_an_existing_file_without_confirmation() {
        let dir = temp_dir("confirm");
        let path = dir.join("out.csv");
        std::fs::write(&path, "previous export\n").expect("seed existing file");
        let mut app = DbProApp {
            export_open: true,
            export_path: path.to_string_lossy().into_owned(),
            export_format: "CSV".to_owned(),
            ..Default::default()
        };
        let result = UiQueryResult {
            columns: vec![crate::UiColumn {
                name: "id".to_owned(),
                data_type: "int".to_owned(),
                nullable: false,
            }],
            rows: vec![vec![UiCell::Number("1".to_owned())]],
            row_count: 1,
            duration_ms: 0,
        };

        app.export_result(&result);

        assert!(app.export_overwrite_pending, "the first click must ask, not overwrite");
        assert!(app.export_open, "the dialog stays open for the confirmation");
        assert_eq!(
            std::fs::read_to_string(&path).expect("read"),
            "previous export\n",
            "nothing may be written before the user confirms"
        );

        app.export_result_confirming_overwrite(&result);

        assert!(!app.export_overwrite_pending);
        assert!(!app.export_open);
        assert_eq!(std::fs::read_to_string(&path).expect("read"), "id\n1\n");
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn export_message_names_a_capped_result_as_capped() {
        let dir = temp_dir("capped");
        let path = dir.join("out.csv");
        let mut app = DbProApp {
            export_open: true,
            export_path: path.to_string_lossy().into_owned(),
            export_format: "CSV".to_owned(),
            ..Default::default()
        };
        let result = UiQueryResult {
            columns: vec![crate::UiColumn {
                name: "id".to_owned(),
                data_type: "int".to_owned(),
                nullable: false,
            }],
            rows: vec![vec![UiCell::Number("1".to_owned())]],
            row_count: 500,
            duration_ms: 0,
        };

        app.export_result(&result);

        assert!(
            app.runtime_message.contains("Exported 1 rows") && app.runtime_message.contains("500 rows matched"),
            "a capped export must not read as a complete one: {}",
            app.runtime_message
        );
        let _ = std::fs::remove_dir_all(&dir);
    }
}
