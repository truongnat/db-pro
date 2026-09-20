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
    database_error_diagnostic, deduplicate_diagnostics, deduplicate_messages, elide_chars, format_query_document,
    prediction_replacement_range, write_file_atomically,
};

/// Caption-style budget for context chips (full names stay on hover tooltips).
const CONTEXT_CHIP_MAX_CHARS: usize = 28;
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

                let more_anchor = self.draw_query_context_strip(ui);
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

    /// Compact context strip: optional path breadcrumb + connection/schema chip + More.
    /// Returns anchors for the context picker and overflow menu.
    fn draw_query_context_strip(&mut self, ui: &mut egui::Ui) -> QueryChromeAnchors {
        let mut anchors = QueryChromeAnchors::default();
        let doc_idx = self.query.session.active_document_index;
        let file_path = self
            .query
            .session
            .documents
            .get(doc_idx)
            .and_then(|document| document.file_path.clone());

        if let Some(path) = file_path.as_deref() {
            self.draw_file_path_breadcrumb(ui, path);
        }

        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing = egui::vec2(4.0, 0.0);
            let chip_resp = self.draw_query_context_chip(ui);
            if chip_resp.clicked() {
                self.query.editor.query_context_picker_open = !self.query.editor.query_context_picker_open;
            }
            anchors.context_anchor = Some(chip_resp.rect);

            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                let more_response = Button::new(self.theme)
                    .icon(Icon::MoreHorizontal)
                    .variant(ButtonVariant::Ghost)
                    .size(ButtonSize::IconSm)
                    .tooltip("More query actions")
                    .show(ui);
                if more_response.clicked() {
                    self.query.editor.query_tools_open = !self.query.editor.query_tools_open;
                }
                anchors.more_anchor = Some(more_response.rect);
            });
        });

        anchors
    }

    fn draw_file_path_breadcrumb(&self, ui: &mut egui::Ui, path: &str) {
        let segments: Vec<&str> = path.split(['/', '\\']).filter(|segment| !segment.is_empty()).collect();
        let start = segments.len().saturating_sub(3);
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing = egui::vec2(2.0, 0.0);
            ui.label(icon_text(Icon::FileCode2, "", self.theme.text_muted));
            if start > 0 {
                ui.label(RichText::new("…").font(font_caption()).color(self.theme.text_muted));
                ui.label(RichText::new("/").font(font_caption()).color(self.theme.text_muted));
            }
            for (index, segment) in segments[start..].iter().enumerate() {
                if index > 0 {
                    ui.label(RichText::new("/").font(font_caption()).color(self.theme.text_muted));
                }
                let is_leaf = start + index + 1 == segments.len();
                ui.label(RichText::new(*segment).font(font_caption()).color(if is_leaf {
                    self.theme.text_secondary
                } else {
                    self.theme.text_muted
                }));
            }
        })
        .response
        .on_hover_text(path);
    }

    fn draw_query_context_chip(&self, ui: &mut egui::Ui) -> egui::Response {
        let connected = self.active_query_connection_id().is_some() && self.connection.lifecycle.is_connected();
        let conn_label = if connected {
            self.active_query_connection_name().to_owned()
        } else {
            "No connection".to_owned()
        };
        let schema = self.active_query_schema().to_owned();
        let environment = self
            .active_query_connection()
            .map(|c| c.environment.as_str())
            .unwrap_or("");
        let is_production = environment.eq_ignore_ascii_case("production") || environment.eq_ignore_ascii_case("prod");

        let chip_text = if connected {
            format!(
                "{} / {}",
                elide_chars(&conn_label, CONTEXT_CHIP_MAX_CHARS),
                elide_chars(&schema, CONTEXT_CHIP_MAX_CHARS)
            )
        } else {
            conn_label.clone()
        };
        let tooltip = if connected {
            format!("{conn_label} · {schema}")
        } else {
            "Choose a connection to run SQL".to_owned()
        };
        let color = if !connected {
            self.theme.warning
        } else if is_production {
            self.theme.danger
        } else {
            self.theme.text_secondary
        };

        let response = egui::Frame::none()
            .fill(egui::Color32::TRANSPARENT)
            .rounding(egui::Rounding::same(RADIUS_SM))
            .inner_margin(egui::Margin::symmetric(SPACE_XS + 2.0, 2.0))
            .stroke(egui::Stroke::new(1.0, self.theme.border_subtle))
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing = egui::vec2(4.0, 0.0);
                    if connected {
                        let dot = if is_production {
                            self.theme.danger
                        } else {
                            self.theme.success
                        };
                        ui.label(RichText::new("●").font(font_caption()).color(dot));
                    } else {
                        ui.label(icon_text(Icon::AlertTriangle, "", self.theme.warning));
                    }
                    ui.label(RichText::new(chip_text).font(font_caption()).color(color));
                    ui.label(icon_text(Icon::ChevronDown, "", self.theme.text_muted));
                });
            })
            .response
            .on_hover_text(tooltip)
            .interact(egui::Sense::click());
        response
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

        let mut next_conn_id = None;
        let mut next_schema = None;
        let mut close = false;
        let menu_width = 280.0;
        let menu_position = egui::pos2(anchor.left(), anchor.bottom() + 4.0);
        let menu = egui::Area::new(egui::Id::new("query_context_picker"))
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
                        RichText::new("Connection")
                            .small()
                            .strong()
                            .color(self.theme.text_muted),
                    );
                    ui.add_space(4.0);
                    egui::ScrollArea::vertical().max_height(180.0).show(ui, |ui| {
                        for (id, name, environment) in &connections {
                            let selected = current_conn_id.as_deref() == Some(id.as_str());
                            let label = if environment.is_empty() {
                                name.clone()
                            } else {
                                format!("{name} · {environment}")
                            };
                            if ui.selectable_label(selected, label).clicked() {
                                next_conn_id = Some(id.clone());
                                close = true;
                            }
                        }
                    });
                    ui.separator();
                    ui.label(RichText::new("Schema").small().strong().color(self.theme.text_muted));
                    ui.add_space(4.0);
                    for sch in &available_schemas {
                        let selected = &current_schema == sch;
                        if ui.selectable_label(selected, sch).clicked() {
                            next_schema = Some(sch.clone());
                            close = true;
                        }
                    }
                });
            });

        if let Some(cid) = next_conn_id {
            self.set_document_connection(doc_idx, Some(cid));
        }
        if let Some(sch) = next_schema {
            self.set_document_schema(doc_idx, Some(sch));
        }

        let clicked_outside = ctx.input(|input| {
            input.pointer.any_click()
                && input
                    .pointer
                    .interact_pos()
                    .is_some_and(|position| !menu.response.rect.contains(position) && !anchor.contains(position))
        });
        if clicked_outside || close {
            self.query.editor.query_context_picker_open = false;
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
                self.draw_output_tabs(ui, true);
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
                self.draw_query_run_stop_button(ui, connected, modifier);
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

    fn draw_query_run_stop_button(&mut self, ui: &mut egui::Ui, connected: bool, modifier: &str) {
        let active_doc_running = self
            .query
            .session
            .documents
            .get(self.query.session.active_document_index)
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
                Button::new(self.theme)
                    .text("Stop")
                    .icon(Icon::Square)
                    .variant(ButtonVariant::Secondary)
                    .size(ButtonSize::Sm)
                    .tooltip("Stop query (Esc)")
                    .show(ui)
            } else {
                let tip = cancel_reason
                    .as_deref()
                    .unwrap_or("Query running (cancellation is unsupported by this provider)");
                Button::new(self.theme)
                    .text("Running…")
                    .icon(Icon::Loader)
                    .variant(ButtonVariant::Secondary)
                    .size(ButtonSize::Sm)
                    .tooltip(tip)
                    .show(ui)
            }
        } else {
            let tip = if !connected {
                "Connect to a database before running".to_owned()
            } else {
                format!("Run query ({modifier}↵)")
            };
            Button::new(self.theme)
                .text("Run")
                .icon(Icon::Play)
                .variant(ButtonVariant::Default)
                .size(ButtonSize::Sm)
                .tooltip(tip)
                .show(ui)
        };
        if run_button.clicked() {
            if let Some(request_id) = active_doc_running {
                if cancel_supported {
                    self.cancel_query(request_id);
                } else {
                    self.feedback.runtime_message = cancel_reason
                        .unwrap_or_else(|| "Query cancellation is not supported for this provider".to_owned());
                }
            } else if !connected {
                self.feedback.runtime_message = "Connect to a database before running a query".to_owned();
            } else {
                self.dispatch_query();
            }
        }
    }
}

#[derive(Default)]
struct QueryChromeAnchors {
    context_anchor: Option<egui::Rect>,
    more_anchor: Option<egui::Rect>,
}

impl DbProApp {
    /// Legacy name kept for call-site clarity during the shell rewrite.
    #[allow(dead_code)]
    fn draw_query_header(&mut self, ui: &mut egui::Ui) -> Option<egui::Rect> {
        self.draw_query_context_strip(ui).more_anchor
    }

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
        let sql = self.query.session.active_text().to_owned();
        let params = crate::query::discover_sql_parameters(&sql);
        if params.is_empty() {
            return;
        }
        let supports_parameters = self.query_capabilities().allows(|caps| caps.query.parameters);
        let doc_index = self.query.session.active_document_index;
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
                .query
                .session
                .documents
                .get(doc_index)
                .and_then(|doc| doc.parameter_values.get(&param.name).cloned())
                .unwrap_or_default();
            let mut is_secret = self
                .query
                .session
                .documents
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
            if let Some(doc) = self.query.session.documents.get_mut(doc_index) {
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
}

#[cfg(test)]
#[path = "query_view_tests.rs"]
mod query_view_tests;
