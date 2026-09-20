use super::*;
use std::time::Instant;

/// Egress note shown with the AI prediction control (#242).
///
/// Inline prediction can still schedule without a click — default is `Subtle`
/// (`crates/ui/src/editor/prediction.rs`) so ghost text stays quieter until reveal,
/// and a request follows 300 ms after an edit or cursor move (`query_document.rs`).
/// The control that chooses the mode is where its data flow has to be stated. The
/// registry entry recording the same facts is `docs/release/known-limitations.md`
/// LIM-019; with no key configured the runtime answers "AI provider is not configured"
/// and nothing leaves the machine (`crates/runtime/src/worker.rs:1118`).
pub(super) const AI_PREDICTION_EGRESS_NOTE: &str =
    "Sends the SQL around your cursor and its schema context to your configured AI provider.";

#[path = "query_helpers.rs"]
mod query_helpers;
pub(crate) use query_helpers::{
    database_error_diagnostic, deduplicate_diagnostics, deduplicate_messages, format_query_document,
    prediction_replacement_range, write_file_atomically,
};

/// Thin query status strip under the editor / dock.
const QUERY_STATUS_HEIGHT: f32 = 24.0;

impl DbProApp {
    pub(super) fn draw_query(&mut self, ui: &mut egui::Ui) {
        // Shell owns horizontal inset (`SHELL_SPLIT_INSET`); keep the query surface flush.
        egui::Frame::none()
            .inner_margin(egui::Margin {
                left: SPACE_XS,
                right: SPACE_XS,
                top: SPACE_XS,
                bottom: 0.0,
            })
            .show(ui, |ui| {
                let driver = self.active_driver().to_owned();
                let lint = self.preferences.settings.editor.lint.clone();
                query_diagnostics_view::refresh_diagnostics(&mut self.query, &driver, &lint);
                if let Some(deadline) = self.query.editor.diagnostics_debounce_at {
                    let remaining = deadline.saturating_duration_since(Instant::now());
                    if !remaining.is_zero() {
                        ui.ctx().request_repaint_after(remaining);
                    }
                }

                let doc_idx = self.query.session.active_document_index;
                let file_path = self
                    .query
                    .session
                    .documents
                    .get(doc_idx)
                    .and_then(|document| document.file_path.clone());
                let connected = self.active_query_connection_id().is_some() && self.connection.lifecycle.is_connected();
                let connection_name = if connected {
                    self.active_query_connection_name().to_owned()
                } else {
                    "No connection".to_owned()
                };
                let schema = self.active_query_schema().to_owned();
                let environment = self
                    .active_query_connection()
                    .map(|connection| connection.environment.clone())
                    .unwrap_or_default();
                let more_anchor = {
                    let mut context = query_context_view::QueryContextViewContext {
                        theme: self.theme,
                        editor: &mut self.query.editor,
                        file_path: file_path.as_deref(),
                        connected,
                        connection_name: &connection_name,
                        schema: &schema,
                        environment: &environment,
                    };
                    query_context_view::draw_context_strip(&mut context, ui)
                };
                if self.query.editor.query_context_picker_open {
                    if let Some(anchor) = more_anchor.context_anchor {
                        self.draw_query_context_picker(ui.ctx(), anchor);
                    }
                }
                if self.query.editor.query_tools_open {
                    if let Some(anchor) = more_anchor.more_anchor {
                        self.draw_query_actions_menu(ui.ctx(), anchor);
                    }
                }

                // Transaction chrome only when relevant — never a permanent form row.
                if self.query.execution.query_txn_bar_open || self.query.execution.query_in_transaction {
                    ui.add_space(4.0);
                    if let Some(action) = TransactionBar::new(
                        self.query.execution.query_in_transaction,
                        self.query.execution.query_txn_pending,
                        self.theme,
                    )
                    .auto_commit(self.query.execution.query_auto_commit)
                    .show(ui)
                    {
                        self.handle_transaction_action(action);
                    }
                }
                if self.query.execution.disconnect_txn_guard {
                    ui.colored_label(
                        self.theme.warning,
                        "Open transaction blocks disconnect — Commit or Rollback first.",
                    );
                    if Button::new(self.theme)
                        .text("Dismiss")
                        .variant(ButtonVariant::Ghost)
                        .size(ButtonSize::Sm)
                        .show(ui)
                        .clicked()
                    {
                        self.query.execution.disconnect_txn_guard = false;
                    }
                }

                // Builder stays secondary: prefer a compact side/bottom split later;
                // for now keep it out of the default vertical stack unless opened.
                if self.query.editor.visual_builder.open {
                    ui.add_space(SPACE_XS);
                    egui::CollapsingHeader::new("Visual query builder")
                        .default_open(true)
                        .show(ui, |ui| {
                            self.draw_visual_query_builder(ui);
                        });
                    ui.add_space(SPACE_XS);
                }

                let status_h = QUERY_STATUS_HEIGHT;
                let dock_open = self.workspace.bottom_panel_open;
                let available = ui.available_height();
                let dock_h = if !dock_open {
                    0.0
                } else if self.query.editor.query_output_dock_maximized {
                    (available - status_h - 80.0).max(OUTPUT_MIN_HEIGHT)
                } else {
                    self.workspace
                        .bottom_panel_height
                        .clamp(OUTPUT_MIN_HEIGHT, OUTPUT_MAX_HEIGHT)
                };
                let editor_h = if self.query.editor.query_output_dock_maximized && dock_open {
                    80.0
                } else {
                    (available - dock_h - status_h).max(120.0)
                };

                ui.allocate_ui_with_layout(
                    egui::vec2(ui.available_width(), editor_h),
                    Layout::top_down(Align::Min),
                    |ui| {
                        self.draw_query_editor(ui);
                    },
                );
                self.draw_floating_completion_popup(ui.ctx());
                {
                    let mut context = query_search_view::QuerySearchContext {
                        theme: self.theme,
                        editor: &mut self.query.editor,
                        session: &mut self.query.session,
                    };
                    query_search_view::draw_editor_search_overlay(&mut context, ui.ctx());
                }

                // Snippets remain opt-in via More; keep them out of the default stack
                // unless the user opened them (floating-ish card is acceptable for now).
                if self.query.editor.snippets_open {
                    self.draw_sql_snippets(ui);
                }
                if self.query.editor.query_params_panel_open {
                    self.draw_sql_parameters_panel(ui);
                }

                if dock_open {
                    self.draw_query_output_dock(ui, dock_h);
                }

                self.draw_query_status_bar(ui);
                self.draw_dirty_close_dialog(ui.ctx());
                self.draw_save_as_dialog(ui.ctx());
            });
    }

    fn draw_query_context_picker(&mut self, ctx: &egui::Context, anchor: egui::Rect) {
        let doc_idx = self.query.session.active_document_index;
        let current_conn_id = self
            .query
            .session
            .documents
            .get(doc_idx)
            .and_then(|d| d.connection_id.clone())
            .or_else(|| self.connection.lifecycle.active_connection_id().map(str::to_owned));
        let current_schema = self.active_query_schema().to_owned();
        let available_schemas = if !self.schema.explorer.schema.schemas.is_empty() {
            self.schema.explorer.schema.schemas.clone()
        } else if !self.query_capabilities().allows(|caps| caps.schema.schemas) {
            vec!["main".to_string()]
        } else {
            vec!["public".to_string()]
        };
        let connections: Vec<(String, String, String)> = self
            .connection
            .catalog
            .iter()
            .map(|c| (c.id.clone(), c.name.clone(), c.environment.clone()))
            .collect();

        let context = query_context_picker_view::QueryContextPickerContext {
            theme: self.theme,
            current_connection_id: current_conn_id.as_deref(),
            current_schema: &current_schema,
            available_schemas: &available_schemas,
            connections: &connections,
        };
        if let Some(action) = query_context_picker_view::draw_picker(&context, ctx, anchor) {
            match action {
                query_context_picker_view::QueryContextPickerAction::SelectConnection(id) => {
                    self.set_document_connection(doc_idx, Some(id));
                    self.query.editor.query_context_picker_open = false;
                }
                query_context_picker_view::QueryContextPickerAction::SelectSchema(schema) => {
                    self.set_document_schema(doc_idx, Some(schema));
                    self.query.editor.query_context_picker_open = false;
                }
                query_context_picker_view::QueryContextPickerAction::Close => {
                    self.query.editor.query_context_picker_open = false;
                }
            }
        }
    }

    fn draw_query_output_dock(&mut self, ui: &mut egui::Ui, dock_height: f32) {
        // Resize grip above the dock.
        let grip_height = 4.0;
        let (grip_rect, grip_resp) =
            ui.allocate_exact_size(egui::vec2(ui.available_width(), grip_height), egui::Sense::drag());
        ui.painter().rect_filled(
            grip_rect,
            0.0,
            if grip_resp.hovered() || grip_resp.dragged() {
                self.theme.border_strong
            } else {
                self.theme.border_subtle
            },
        );
        if grip_resp.dragged() {
            let next_height = self.workspace.bottom_panel_height - grip_resp.drag_delta().y;
            self.workspace.set_bottom_panel_height(next_height);
            self.query.editor.query_output_dock_maximized = false;
        }
        grip_resp.on_hover_cursor(egui::CursorIcon::ResizeVertical);

        let body_h = (dock_height - grip_height).max(OUTPUT_MIN_HEIGHT - grip_height);
        ui.allocate_ui_with_layout(
            egui::vec2(ui.available_width(), body_h),
            Layout::top_down(Align::Min),
            |ui| {
                let mut tabs_context = query_output_tabs_view::QueryOutputTabsContext {
                    theme: self.theme,
                    output: &mut self.query.output,
                    session: &self.query.session,
                    editor: &mut self.query.editor,
                    bottom_panel_open: &mut self.workspace.bottom_panel_open,
                };
                query_output_tabs_view::draw_output_tabs(&mut tabs_context, ui, true);
                let result = self.query.session.active_result().cloned();
                self.draw_output_pane(ui, result.as_ref());
            },
        );
    }

    fn draw_query_status_bar(&mut self, ui: &mut egui::Ui) {
        let modifier = Self::primary_modifier_label();
        let connected = self.active_query_connection_id().is_some() && self.connection.lifecycle.is_connected();
        let driver = self.active_query_driver().to_owned();
        let schema = self.active_query_schema().to_owned();
        let param_key = (
            self.query.session.active_document_index,
            self.query.session.active_buffer_version(),
        );
        if self.query.editor.param_count_cache_key != Some(param_key) {
            self.query.editor.param_count_cache_key = Some(param_key);
            self.query.editor.param_count_cache =
                crate::query::discover_sql_parameters(self.query.session.active_text()).len();
        }
        let param_count = self.query.editor.param_count_cache;
        let diagnostic_count = self.query.editor.diagnostics.len();
        let txn_label = if self.query.execution.query_in_transaction {
            format!("Transaction · {} pending", self.query.execution.query_txn_pending)
        } else if self.query.execution.query_auto_commit {
            "Auto-commit".to_owned()
        } else {
            "Manual".to_owned()
        };

        let (rect, _) = ui.allocate_exact_size(
            egui::vec2(ui.available_width(), QUERY_STATUS_HEIGHT),
            egui::Sense::hover(),
        );
        ui.painter().hline(
            rect.x_range(),
            rect.top(),
            egui::Stroke::new(1.0, self.theme.border_subtle),
        );

        ui.scope_builder(egui::UiBuilder::new().max_rect(rect), |ui| {
            // Outer right_to_left keeps Run pinned; left metadata fills the remainder.
            // Nested right_to_left inside an already-started horizontal was pushing Run off-screen.
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                ui.set_min_height(QUERY_STATUS_HEIGHT);
                ui.spacing_mut().item_spacing = egui::vec2(8.0, 0.0);
                ui.add_space(SPACE_XS);
                self.draw_query_run_control(ui, connected, modifier);
                if !self.workspace.bottom_panel_open
                    && Button::new(self.theme)
                        .icon(Icon::PanelBottom)
                        .variant(ButtonVariant::Ghost)
                        .size(ButtonSize::IconSm)
                        .tooltip("Show output")
                        .show(ui)
                        .clicked()
                {
                    self.workspace.bottom_panel_open = true;
                }

                ui.with_layout(Layout::left_to_right(Align::Center), |ui| {
                    ui.spacing_mut().item_spacing = egui::vec2(8.0, 0.0);
                    ui.add_space(SPACE_XS);
                    ui.label(
                        RichText::new(format!(
                            "Ln {}, Col {}",
                            self.query.editor.query_cursor_line, self.query.editor.query_cursor_column
                        ))
                        .font(font_mono_sm())
                        .color(self.theme.text_muted),
                    );
                    ui.label(RichText::new(driver).font(font_caption()).color(self.theme.text_muted));
                    ui.label(RichText::new(schema).font(font_caption()).color(self.theme.text_muted));

                    let txn_resp = ui.add(
                        egui::Label::new(RichText::new(&txn_label).font(font_caption()).color(
                            if self.query.execution.query_in_transaction {
                                self.theme.warning
                            } else {
                                self.theme.text_muted
                            },
                        ))
                        .sense(egui::Sense::click()),
                    );
                    if txn_resp.clicked() {
                        self.query.execution.query_txn_bar_open = !self.query.execution.query_txn_bar_open;
                    }
                    txn_resp.on_hover_text("Toggle transaction controls");

                    if param_count > 0 {
                        let label = if param_count == 1 {
                            "1 parameter".to_owned()
                        } else {
                            format!("{param_count} parameters")
                        };
                        let resp = ui.add(
                            egui::Label::new(RichText::new(label).font(font_caption()).color(self.theme.accent))
                                .sense(egui::Sense::click()),
                        );
                        if resp.clicked() {
                            self.query.editor.query_params_panel_open = !self.query.editor.query_params_panel_open;
                        }
                        resp.on_hover_text("Edit bind parameters");
                    }

                    if diagnostic_count > 0 {
                        let resp = ui.add(
                            egui::Label::new(
                                RichText::new(format!("{diagnostic_count} diagnostics"))
                                    .font(font_caption())
                                    .color(self.theme.warning),
                            )
                            .sense(egui::Sense::click()),
                        );
                        if resp.clicked() {
                            self.workspace.bottom_panel_open = true;
                            if let Some(doc_id) = self
                                .query
                                .session
                                .documents
                                .get(self.query.session.active_document_index)
                                .map(|d| d.id.clone())
                            {
                                self.query.output.set_for_document_and_activate_if_active(
                                    &doc_id,
                                    self.query
                                        .session
                                        .active_document()
                                        .map(|document| document.id.as_str()),
                                    OutputTab::Messages,
                                );
                            }
                        }
                    }
                });
            });
        });
    }

    fn draw_query_run_control(&mut self, ui: &mut egui::Ui, connected: bool, modifier: &str) {
        let active_request_id = self
            .query
            .session
            .documents
            .get(self.query.session.active_document_index)
            .and_then(|document| match document.execution_state {
                QueryExecutionState::Running(request_id) => Some(request_id),
                _ => None,
            });
        let capabilities = self.query_capabilities();
        let cancel_reason =
            capabilities.feature_limitation(db_pro_core::domain::capabilities::CapabilityFeature::Cancel);
        let context = query_run_control_view::QueryRunControlContext {
            theme: self.theme,
            connected,
            active_request_id,
            cancel_supported: capabilities.allows(|value| value.query.cancel),
            cancel_reason: cancel_reason.as_deref(),
            modifier,
        };
        let Some(action) = query_run_control_view::draw_run_control(&context, ui) else {
            return;
        };
        match action {
            query_run_control_view::QueryRunControlAction::Run => self.dispatch_query(),
            query_run_control_view::QueryRunControlAction::Cancel(request_id) => self.cancel_query(request_id),
            query_run_control_view::QueryRunControlAction::ReportUnsupportedCancel(reason) => {
                self.feedback.runtime_message = reason;
            }
            query_run_control_view::QueryRunControlAction::ReportDisconnected => {
                self.feedback.runtime_message = "Connect to a database before running a query".to_owned();
            }
        }
    }
}

impl DbProApp {
    /// Keyword / table / column completion list (legacy inline card — floating popup is canonical).
    #[allow(dead_code)]
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
            candidates.extend(self.schema.explorer.schema.views.iter().map(|view| view.name.clone()));
            candidates.extend(
                self.schema
                    .explorer
                    .schema
                    .functions
                    .iter()
                    .map(|function| function.name.clone()),
            );
            for keyword in candidates.iter() {
                if ui
                    .selectable_label(false, keyword)
                    .on_hover_text("Insert SQL keyword or expression")
                    .clicked()
                {
                    self.append_to_active_query(keyword);
                    self.query.editor.completion_open = false;
                }
            }
        });
    }

    /// Quick SQL snippet inserters.
    fn draw_sql_snippets(&mut self, ui: &mut egui::Ui) {
        card_frame(self.theme).show(ui, |ui| {
            ui.label(RichText::new("SQL snippets").strong());
            for (label, snippet) in query_snippets::builtin_sql_snippets() {
                if Button::new(self.theme)
                    .text(*label)
                    .variant(ButtonVariant::Secondary)
                    .size(ButtonSize::Sm)
                    .show(ui)
                    .clicked()
                {
                    self.insert_snippet(snippet);
                    self.query.editor.snippets_open = false;
                }
            }
        });
    }

    /// Parser diagnostics for the current SQL (legacy list — gutter + status count are canonical).
    #[allow(dead_code)]
    fn draw_diagnostics(&mut self, ui: &mut egui::Ui) {
        if self.query.editor.diagnostics.is_empty() {
            return;
        }
        ui.colored_label(
            self.theme.warning,
            format!("Diagnostics · {}", self.query.editor.diagnostics.len()),
        );
        for diagnostic in &self.query.editor.diagnostics {
            ui.colored_label(self.theme.warning, format!("• {diagnostic}"));
        }
    }

    /// Discovered bind placeholders for the active document (#225 discovery slice).
    fn draw_sql_parameters_panel(&mut self, ui: &mut egui::Ui) {
        let supports_parameters = self.query_capabilities().allows(|caps| caps.query.parameters);
        let mut context = query_parameters_view::QueryParametersContext {
            theme: self.theme,
            session: &mut self.query.session,
            supports_parameters,
        };
        query_parameters_view::draw_parameters_panel(&mut context, ui);
    }
}

#[cfg(test)]
#[path = "query_view_tests.rs"]
mod query_view_tests;
