use super::*;
use std::time::Instant;

/// Egress note shown with the AI prediction control (#242).
///
/// Inline prediction is the one AI feature that runs without any user action — `PredictionMode`
/// defaults to `Eager` (`crates/ui/src/editor/prediction.rs:5-11`) and a scheduled request follows
/// 300 ms after an edit or cursor move (`query_document.rs:13`) — so the control that chooses the
/// mode is where its data flow has to be stated. The registry entry recording the same facts is
/// `docs/release/known-limitations.md` LIM-019; with no key configured the runtime answers
/// "AI provider is not configured" and nothing leaves the machine (`crates/runtime/src/worker.rs:1118`).
pub(super) const AI_PREDICTION_EGRESS_NOTE: &str =
    "Sends the SQL around your cursor and its schema context to your configured AI provider.";

#[path = "query_helpers.rs"]
mod query_helpers;
pub(crate) use query_helpers::{
    database_error_diagnostic, deduplicate_diagnostics, deduplicate_messages, format_query_document,
    prediction_replacement_range, write_file_atomically,
};

impl DbProApp {
    pub(super) fn draw_query(&mut self, ui: &mut egui::Ui) {
        ui.add_space(SPACE_SM);
        egui::Frame::none()
            .inner_margin(egui::Margin::symmetric(SPACE_MD, 0.0))
            .show(ui, |ui| {
                self.refresh_diagnostics();
                let more_anchor = self.draw_query_header(ui);
                if self.query_txn_bar_open || self.query_in_transaction {
                    ui.add_space(6.0);
                    if let Some(action) =
                        TransactionBar::new(self.query_in_transaction, self.query_txn_pending, self.theme)
                            .auto_commit(self.query_auto_commit)
                            .show(ui)
                    {
                        self.handle_transaction_action(action);
                    }
                }
                if self.disconnect_txn_guard {
                    ui.colored_label(
                        self.theme.warning,
                        "Open transaction blocks disconnect — Commit or Rollback first.",
                    );
                    if compact_button(ui, "Dismiss", self.theme).clicked() {
                        self.disconnect_txn_guard = false;
                    }
                }
                ui.add_space(4.0);
                if self.query_tools_open {
                    if let Some(anchor) = more_anchor {
                        self.draw_query_actions_menu(ui.ctx(), anchor);
                    }
                }
                if self.editor_search_open {
                    self.draw_editor_search_bar(ui);
                }
                if self.visual_query_builder_open {
                    egui::CollapsingHeader::new("Visual query builder")
                        .default_open(true)
                        .show(ui, |ui| {
                            self.draw_visual_query_builder(ui);
                        });
                    ui.add_space(SPACE_SM);
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

    /// Query title / file path, connection breadcrumb, run/stop and the overflow button.
    /// Returns the overflow button rect so the actions menu can anchor to it.
    fn draw_query_header(&mut self, ui: &mut egui::Ui) -> Option<egui::Rect> {
        let modifier = Self::primary_modifier_label();
        let mut more_anchor = None;
        let doc_idx = self.active_query_document;
        let file_path = self
            .query_documents
            .get(doc_idx)
            .and_then(|document| document.file_path.clone());
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
        let connected = self.active_query_connection_id().is_some() && self.connected;

        let mut next_conn_id = None;
        let mut next_schema = None;

        // Zed-like path strip for file-backed SQL; plain title for untitled buffers.
        if let Some(path) = file_path.as_deref() {
            ui.horizontal(|ui| {
                ui.label(icon_text(Icon::FileCode2, "", self.theme.text_muted));
                ui.label(
                    RichText::new(path)
                        .font(font_caption())
                        .monospace()
                        .color(self.theme.text_secondary),
                );
            });
            ui.add_space(4.0);
        }

        ui.horizontal(|ui| {
            if file_path.is_none() {
                ui.label(
                    RichText::new(query_title)
                        .font(font_subheading())
                        .strong()
                        .color(self.theme.text_primary),
                );
                ui.label(icon_text(Icon::ChevronRight, "", self.theme.text_muted));
            }

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
                    let tip = if !connected {
                        "Connect to a database before running".to_owned()
                    } else {
                        format!("Run query ({modifier}↵)")
                    };
                    primary_button_with_icon(ui, Icon::Play, "Run", self.theme).on_hover_text(tip)
                };
                if run_button.clicked() {
                    if let Some(request_id) = active_doc_running {
                        if cancel_supported {
                            self.cancel_query(request_id);
                        } else {
                            self.runtime_message = cancel_reason
                                .unwrap_or_else(|| "Query cancellation is not supported for this provider".to_owned());
                        }
                    } else if !connected {
                        self.runtime_message = "Connect to a database before running a query".to_owned();
                    } else {
                        self.dispatch_query();
                    }
                }
                let builder_label = if self.visual_query_builder_open {
                    "Builder ✓"
                } else {
                    "Builder"
                };
                if secondary_button(ui, builder_label, self.theme)
                    .on_hover_text("Toggle visual SELECT builder (#247)")
                    .clicked()
                {
                    self.visual_query_builder_open = !self.visual_query_builder_open;
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
    fn draw_query_editor_actions(&mut self, ui: &mut egui::Ui) -> bool {
        let mut close_menu = false;
        if menu_button_with_icon(ui, Icon::Search, "Find in SQL", self.theme).clicked() {
            self.editor_search_open = !self.editor_search_open;
            close_menu = true;
        }
        let txn_label = if self.query_txn_bar_open {
            "Hide transaction controls"
        } else {
            "Show transaction controls"
        };
        if menu_button_with_icon(ui, Icon::GitBranch, txn_label, self.theme).clicked() {
            self.query_txn_bar_open = !self.query_txn_bar_open;
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

#[cfg(test)]
#[path = "query_view_tests.rs"]
mod query_view_tests;
