//! Query SQL editor surface and floating completion popup.
use super::query_view::{format_query_document, prediction_replacement_range};
use super::*;
use crate::editor::{
    CompletionIntent, CompletionItem, CompletionItemKind, CompletionTriggerKind, EditorInteractionPolicy, SqlDialect,
    SqlEditor,
};
use crate::query::{
    CompletionContext, HoverColumn, RichHoverHelp, SchemaCompletionProvider, SqlSignatureHelp, SqlSymbolHelp,
};
use egui::RichText;
use std::time::Instant;

fn draw_symbol_help(ui: &mut egui::Ui, help: &SqlSymbolHelp, theme: &DbProTheme) {
    ui.label(
        RichText::new(&help.title)
            .font(FontId::monospace(12.5))
            .strong()
            .color(theme.text_primary),
    );
    ui.horizontal(|ui| {
        ui.label(RichText::new(&help.kind).small().strong().color(theme.accent));
        ui.label(RichText::new(&help.detail).small().color(theme.text_secondary));
    });
    ui.add(egui::Label::new(RichText::new(&help.documentation).small().color(theme.text_muted)).wrap());
}

/// Render a column summary row inside a table hover card.
fn draw_hover_column_row(ui: &mut egui::Ui, col: &HoverColumn, theme: &DbProTheme) {
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing = egui::vec2(4.0, 0.0);
        // PK badge
        if col.is_primary_key {
            egui::Frame::none()
                .fill(theme.soft_tint(theme.warning))
                .rounding(egui::Rounding::same(3.0))
                .inner_margin(egui::Margin::symmetric(3.0, 1.0))
                .show(ui, |ui| {
                    ui.label(RichText::new("PK").font(FontId::monospace(8.5)).color(theme.warning));
                });
        } else if col.is_foreign_key {
            egui::Frame::none()
                .fill(theme.soft_tint(theme.info))
                .rounding(egui::Rounding::same(3.0))
                .inner_margin(egui::Margin::symmetric(3.0, 1.0))
                .show(ui, |ui| {
                    ui.label(RichText::new("FK").font(FontId::monospace(8.5)).color(theme.info));
                });
        } else {
            ui.add_space(24.0);
        }
        // Column name
        ui.label(
            RichText::new(&col.name)
                .font(FontId::monospace(12.0))
                .color(theme.text_primary),
        );
        // Type
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            let null_text = if col.nullable { "null" } else { "not null" };
            ui.label(
                RichText::new(null_text)
                    .font(FontId::monospace(10.0))
                    .color(theme.text_muted),
            );
            ui.add_space(4.0);
            ui.label(
                RichText::new(&col.data_type)
                    .font(FontId::monospace(10.5))
                    .color(theme.code_type),
            );
        });
    });
}

/// Draw the rich hover popup for any `RichHoverHelp` variant.
fn draw_rich_hover_popup(
    ctx: &egui::Context,
    anchor: egui::Rect,
    token_range: (usize, usize),
    help: &RichHoverHelp,
    theme: &DbProTheme,
) {
    let popup_width: f32 = match help {
        RichHoverHelp::Table { columns, .. } => {
            if columns.len() > 8 {
                520.0
            } else {
                420.0
            }
        }
        RichHoverHelp::Keyword { example: Some(_), .. } => 500.0,
        _ => 380.0,
    };
    let max_height: f32 = 400.0;

    let position = crate::components::clamp_popup_to_screen(
        anchor.left_bottom() + egui::vec2(0.0, 6.0),
        egui::vec2(popup_width, max_height),
        ctx.screen_rect(),
        10.0,
    );

    egui::Area::new(egui::Id::new(("sql_rich_hover", token_range)))
        .order(egui::Order::Tooltip)
        .fixed_pos(position)
        .interactable(true) // Allow text selection / copy
        .show(ctx, |ui| {
            egui::Frame::none()
                .fill(theme.surface_panel)
                .stroke(egui::Stroke::new(1.0, theme.border_default))
                .rounding(egui::Rounding::same(8.0))
                .shadow(theme.floating_shadow())
                .inner_margin(egui::Margin::same(10.0))
                .show(ui, |ui| {
                    ui.set_max_width(popup_width);
                    egui::ScrollArea::vertical()
                        .max_height(max_height - 20.0)
                        .show(ui, |ui| {
                            draw_rich_hover_content(ui, help, theme);
                        });
                });
        });
}

/// Inner content renderer (called inside the scrollable area).
fn draw_rich_hover_content(ui: &mut egui::Ui, help: &RichHoverHelp, theme: &DbProTheme) {
    match help {
        RichHoverHelp::Table {
            qualified_name,
            kind,
            row_count,
            columns,
            foreign_keys,
        } => {
            // Header
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing = egui::vec2(4.0, 0.0);
                let (badge_text, badge_color) = if kind == "View" {
                    ("VIEW", theme.info)
                } else {
                    ("TABLE", theme.accent)
                };
                egui::Frame::none()
                    .fill(theme.soft_tint(badge_color))
                    .rounding(egui::Rounding::same(4.0))
                    .inner_margin(egui::Margin::symmetric(5.0, 2.0))
                    .show(ui, |ui| {
                        ui.label(
                            RichText::new(badge_text)
                                .font(FontId::monospace(9.0))
                                .color(badge_color),
                        );
                    });
                ui.label(
                    RichText::new(qualified_name)
                        .font(FontId::monospace(13.0))
                        .strong()
                        .color(theme.text_primary),
                );
            });

            if let Some(rows) = row_count {
                ui.label(
                    RichText::new(format!("~{rows} rows"))
                        .font(FontId::proportional(11.0))
                        .color(theme.text_muted),
                );
            }

            if !columns.is_empty() {
                ui.add_space(6.0);
                ui.label(
                    RichText::new(format!("{} columns", columns.len()))
                        .small()
                        .strong()
                        .color(theme.text_muted),
                );
                ui.add_space(2.0);
                ui.separator();
                for col in columns.iter().take(20) {
                    draw_hover_column_row(ui, col, theme);
                }
                if columns.len() > 20 {
                    ui.label(
                        RichText::new(format!("… and {} more columns", columns.len() - 20))
                            .small()
                            .color(theme.text_muted),
                    );
                }
            }

            if !foreign_keys.is_empty() {
                ui.add_space(6.0);
                ui.label(RichText::new("Foreign keys").small().strong().color(theme.text_muted));
                ui.add_space(2.0);
                for fk in foreign_keys {
                    ui.label(
                        RichText::new(format!(
                            "({}) → {}({})",
                            fk.from_columns.join(", "),
                            fk.to_table,
                            fk.to_columns.join(", "),
                        ))
                        .font(FontId::monospace(11.0))
                        .color(theme.text_secondary),
                    );
                }
            }
        }

        RichHoverHelp::Column {
            qualified_name,
            data_type,
            nullable,
            is_primary_key,
            parent_table,
            foreign_key_target,
        } => {
            // Type badge + name
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing = egui::vec2(4.0, 0.0);
                egui::Frame::none()
                    .fill(theme.soft_tint(theme.code_type))
                    .rounding(egui::Rounding::same(4.0))
                    .inner_margin(egui::Margin::symmetric(5.0, 2.0))
                    .show(ui, |ui| {
                        ui.label(RichText::new("COL").font(FontId::monospace(9.0)).color(theme.code_type));
                    });
                ui.label(
                    RichText::new(qualified_name)
                        .font(FontId::monospace(13.0))
                        .strong()
                        .color(theme.text_primary),
                );
            });

            ui.add_space(4.0);
            ui.horizontal_wrapped(|ui| {
                ui.spacing_mut().item_spacing = egui::vec2(4.0, 4.0);
                // Type chip
                egui::Frame::none()
                    .fill(theme.soft_tint(theme.code_type))
                    .rounding(egui::Rounding::same(4.0))
                    .inner_margin(egui::Margin::symmetric(5.0, 2.0))
                    .show(ui, |ui| {
                        ui.label(
                            RichText::new(data_type)
                                .font(FontId::monospace(10.5))
                                .color(theme.code_type),
                        );
                    });
                // Null chip
                let null_color = if *nullable { theme.text_muted } else { theme.warning };
                let null_text = if *nullable { "nullable" } else { "not null" };
                egui::Frame::none()
                    .fill(theme.soft_tint(null_color))
                    .rounding(egui::Rounding::same(4.0))
                    .inner_margin(egui::Margin::symmetric(5.0, 2.0))
                    .show(ui, |ui| {
                        ui.label(RichText::new(null_text).font(FontId::monospace(10.0)).color(null_color));
                    });
                // PK chip
                if *is_primary_key {
                    egui::Frame::none()
                        .fill(theme.soft_tint(theme.warning))
                        .rounding(egui::Rounding::same(4.0))
                        .inner_margin(egui::Margin::symmetric(5.0, 2.0))
                        .show(ui, |ui| {
                            ui.label(
                                RichText::new("primary key")
                                    .font(FontId::monospace(10.0))
                                    .color(theme.warning),
                            );
                        });
                }
            });

            ui.add_space(4.0);
            ui.label(
                RichText::new(format!("Column of {parent_table}"))
                    .small()
                    .color(theme.text_muted),
            );

            if let Some(target) = foreign_key_target {
                ui.label(
                    RichText::new(format!("References {target}"))
                        .font(FontId::monospace(11.0))
                        .color(theme.info),
                );
            }
        }

        RichHoverHelp::Function {
            label,
            parameters,
            active_parameter,
            return_type,
            documentation,
        } => {
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing = egui::vec2(4.0, 0.0);
                egui::Frame::none()
                    .fill(theme.soft_tint(theme.code_function))
                    .rounding(egui::Rounding::same(4.0))
                    .inner_margin(egui::Margin::symmetric(5.0, 2.0))
                    .show(ui, |ui| {
                        ui.label(
                            RichText::new("FN")
                                .font(FontId::monospace(9.0))
                                .color(theme.code_function),
                        );
                    });
                ui.label(
                    RichText::new(label)
                        .font(FontId::monospace(12.5))
                        .strong()
                        .color(theme.text_primary),
                );
            });
            if !parameters.is_empty() {
                ui.add_space(4.0);
                ui.horizontal_wrapped(|ui| {
                    for (idx, param) in parameters.iter().enumerate() {
                        let color = if idx == *active_parameter {
                            theme.accent
                        } else {
                            theme.text_muted
                        };
                        ui.label(RichText::new(param).font(FontId::monospace(11.0)).color(color));
                        if idx + 1 < parameters.len() {
                            ui.label(RichText::new(",").font(FontId::monospace(11.0)).color(theme.text_muted));
                        }
                    }
                });
            }
            if !return_type.is_empty() {
                ui.label(
                    RichText::new(format!("→ {return_type}"))
                        .font(FontId::monospace(11.0))
                        .color(theme.code_type),
                );
            }
            if !documentation.is_empty() {
                ui.add_space(4.0);
                ui.add(egui::Label::new(RichText::new(documentation).small().color(theme.text_muted)).wrap());
            }
        }

        RichHoverHelp::Keyword {
            keyword,
            dialect_note,
            documentation,
            example,
        } => {
            // Header row
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing = egui::vec2(4.0, 0.0);
                egui::Frame::none()
                    .fill(theme.soft_tint(theme.code_keyword))
                    .rounding(egui::Rounding::same(4.0))
                    .inner_margin(egui::Margin::symmetric(5.0, 2.0))
                    .show(ui, |ui| {
                        ui.label(
                            RichText::new("SQL")
                                .font(FontId::monospace(9.0))
                                .color(theme.code_keyword),
                        );
                    });
                ui.label(
                    RichText::new(keyword.as_str())
                        .font(FontId::monospace(13.0))
                        .strong()
                        .color(theme.code_keyword),
                );
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(RichText::new(dialect_note.as_str()).small().color(theme.text_muted));
                });
            });

            ui.add_space(4.0);
            ui.add(egui::Label::new(RichText::new(documentation.as_str()).small().color(theme.text_primary)).wrap());

            if let Some(ex) = example {
                ui.add_space(6.0);
                ui.label(RichText::new("Example").small().strong().color(theme.text_muted));
                ui.add_space(2.0);
                egui::Frame::none()
                    .fill(theme.editor_gutter_fill())
                    .rounding(egui::Rounding::same(5.0))
                    .inner_margin(egui::Margin::same(6.0))
                    .show(ui, |ui| {
                        ui.set_width(ui.available_width());
                        ui.add(
                            egui::Label::new(
                                RichText::new(ex.as_str())
                                    .font(FontId::monospace(11.0))
                                    .color(theme.text_secondary),
                            )
                            .wrap(),
                        );
                    });
            }
        }

        RichHoverHelp::Symbol(help) => {
            draw_symbol_help(ui, help, theme);
        }
    }
}

fn draw_signature_help(ctx: &egui::Context, anchor: egui::Pos2, signature: &SqlSignatureHelp, theme: &DbProTheme) {
    const POPUP_WIDTH: f32 = 440.0;
    const POPUP_HEIGHT: f32 = 112.0;
    let position = crate::components::clamp_popup_to_screen(
        anchor + egui::vec2(8.0, 22.0),
        egui::vec2(POPUP_WIDTH, POPUP_HEIGHT),
        ctx.screen_rect(),
        10.0,
    );
    egui::Area::new(egui::Id::new("sql_signature_help"))
        .order(egui::Order::Foreground)
        .fixed_pos(position)
        .interactable(false)
        .show(ctx, |ui| {
            egui::Frame::none()
                .fill(theme.surface_panel)
                .stroke(egui::Stroke::new(1.0, theme.border_default))
                .rounding(egui::Rounding::same(7.0))
                .shadow(theme.floating_shadow())
                .inner_margin(egui::Margin::same(8.0))
                .show(ui, |ui| {
                    ui.set_width(POPUP_WIDTH);
                    ui.label(
                        RichText::new(&signature.label)
                            .font(FontId::monospace(12.0))
                            .color(theme.text_primary),
                    );
                    if !signature.parameters.is_empty() {
                        ui.horizontal_wrapped(|ui| {
                            for (index, parameter) in signature.parameters.iter().enumerate() {
                                let color = if index == signature.active_parameter {
                                    theme.accent
                                } else {
                                    theme.text_muted
                                };
                                ui.label(RichText::new(parameter).font(FontId::monospace(11.0)).color(color));
                            }
                        });
                    }
                    ui.label(
                        RichText::new(&signature.documentation)
                            .small()
                            .color(theme.text_secondary),
                    );
                });
        });
}

fn draw_completion_explanation(ui: &mut egui::Ui, item: &CompletionItem, theme: &DbProTheme) {
    ui.label(
        RichText::new(&item.label)
            .font(FontId::monospace(12.5))
            .strong()
            .color(theme.text_primary),
    );
    if let Some(detail) = item.detail.as_deref() {
        ui.label(RichText::new(detail).small().color(theme.text_secondary));
    }
    if let Some(documentation) = item.documentation.as_deref() {
        ui.add(egui::Label::new(RichText::new(documentation).small().color(theme.text_muted)).wrap());
    }
}

fn apply_completion_item(doc: &mut QueryDocument, item: &CompletionItem, dialect: SqlDialect) -> bool {
    if !doc.completion.can_apply_to_version(doc.buffer.version()) {
        doc.completion.close();
        return false;
    }

    let (start, end) = item.replacement_range;
    let source = doc.buffer.text();
    if start > end || end > source.len() || !source.is_char_boundary(start) || !source.is_char_boundary(end) {
        doc.completion.close();
        return false;
    }

    doc.buffer.replace(start, end, &item.insert_text);
    doc.cursor.set_offset(&doc.buffer, start + item.insert_text.len());
    doc.selection.collapse_to_active();
    doc.reanalyze(dialect);
    doc.dirty = true;
    doc.prediction = None;
    doc.completion.close();
    true
}

impl DbProApp {
    pub(super) fn draw_query_editor(&mut self, ui: &mut egui::Ui) {
        let editor_width = ui.max_rect().width();
        // Editor owns the allocated region from draw_query — no permanent output reserve.
        let editor_height = ui.available_height().max(120.0);

        if self.query.session.active_document_index >= self.query.session.documents.len() {
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
        let font_size = self.query.editor.editor_font_size;
        let auto_focus = self.query.editor.query_focus_editor_on_open;
        let mut dispatch_statement = false;
        let mut dispatch_all = false;
        let mut save_query = false;

        let available_size = egui::vec2(editor_width.max(280.0), editor_height);

        let doc_index = self.query.session.active_document_index;
        let doc = &mut self.query.session.documents[doc_index];

        let search_query = self.query.editor.editor_search.clone();
        let is_completion_open = doc.completion.is_open;
        let previous_completion_trigger = doc.completion.trigger_kind;
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
        .with_auto_focus(auto_focus)
        .with_cached_tokens(&mut doc.cached_tokens)
        .with_search(&search_query, doc.search.active_match_index)
        .with_completion_open(is_completion_open)
        .with_execution_range(execution_range)
        .with_prediction_visible(self.preferences.prediction_mode == PredictionMode::Eager || doc.prediction_reveal);
        editor.font_size = font_size;

        let response = editor.show(ui, available_size);
        self.query.editor.query_focus_editor_on_open = false;
        self.query.editor.query_editor_rect = response.rect;

        let cursor_context_changed = previous_cursor != doc.cursor.offset || previous_selection != doc.selection;
        let completion_intent =
            EditorInteractionPolicy::completion_intent(&response, is_completion_open, cursor_context_changed);
        if completion_intent == CompletionIntent::Close {
            doc.completion.close();
        }
        if cursor_context_changed {
            if let Some(request_id) = doc.pending_prediction_request {
                self.task_bridge
                    .send_best_effort(UiCommand::CancelSqlPrediction { request_id });
            }
            doc.invalidate_prediction();
        }

        self.query.editor.query_editor_focused = response.focused;
        self.query.editor.query_cursor_line = doc.cursor.line + 1;
        self.query.editor.query_cursor_column = doc.cursor.col + 1;

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
                self.task_bridge
                    .send_best_effort(UiCommand::CancelSqlPrediction { request_id });
            }
            format_query_document(doc, dialect);
        }

        if response.changed {
            doc.reanalyze(dialect);
            doc.dirty = true;
            doc.execution_diagnostic = None;
            if !doc.selection.is_empty() {
                let (start, end) = doc.selection.normalized();
                self.query.session.selected_text = doc.buffer.slice(start, end).to_owned();
            } else {
                self.query.session.selected_text.clear();
            }
        }
        if response.wants_format {
            if !doc.selection.is_empty() {
                let (start, end) = doc.selection.normalized();
                self.query.session.selected_text = doc.buffer.slice(start, end).to_owned();
            } else {
                self.query.session.selected_text.clear();
            }
        }

        // Prediction is an explicit action, never a side effect of typing or moving the
        // caret. This keeps the editor quiet even when an older persisted setting opted into
        // a prediction mode; the user must deliberately request an AI prediction.
        if EditorInteractionPolicy::should_schedule_prediction(
            &response,
            self.preferences.prediction_mode,
            doc.selection.is_empty(),
            cursor_in_string_or_comment,
        ) {
            doc.schedule_prediction_with_mode(Instant::now(), true);
        }

        if self.preferences.prediction_mode != PredictionMode::Off
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
                &self.schema_explorer.schema,
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
                self.task_bridge.send_best_effort(UiCommand::RequestSqlPrediction {
                    request_id: req_id,
                    document_id: doc.id.clone(),
                    document_version,
                    anchor,
                    replacement_range,
                    context: ai_context,
                });
            }
        }

        if response.wants_save {
            save_query = true;
        }
        if response.wants_execute_statement {
            dispatch_statement = true;
        } else if response.wants_execute_all {
            dispatch_all = true;
        }

        let completion_request = match completion_intent {
            CompletionIntent::Open(trigger) => Some((trigger, trigger == CompletionTriggerKind::Manual)),
            CompletionIntent::Refresh => {
                let trigger = previous_completion_trigger.unwrap_or(CompletionTriggerKind::Automatic);
                Some((trigger, trigger == CompletionTriggerKind::Manual))
            }
            CompletionIntent::None | CompletionIntent::Close => None,
        };
        if let Some((trigger_kind, is_manual_trigger)) = completion_request {
            let (before_cursor, after_cursor) = doc.buffer.split_at(doc.cursor.offset);
            let context = CompletionContext {
                text_before_cursor: before_cursor,
                text_after_cursor: after_cursor,
                cursor_offset: doc.cursor.offset,
                active_schema: &active_schema,
                schema_summary: &self.schema_explorer.schema,
                cached_tokens: Some(&doc.cached_tokens),
                is_sqlite: uses_positional_editor,
                is_manual_trigger,
            };
            let (prefix, items) = SchemaCompletionProvider::provide(&context);
            if items.is_empty() {
                doc.completion.close();
            } else {
                let document_version = doc.buffer.version();
                doc.completion.open(
                    doc.cursor.offset,
                    document_version,
                    response.cursor_screen_pos,
                    prefix,
                    items,
                    trigger_kind,
                );
            }
        }

        // ── Signature help ─────────────────────────────────────────────────────
        // Show signature help only while actively typing inside a function call.
        let signature = if response.focused && !doc.completion.is_open {
            self.schema_explorer.schema_symbol_index.signature_help(
                doc.buffer.text(),
                doc.cursor.offset,
                &active_schema,
                dialect,
                &doc.cached_tokens,
            )
        } else {
            None
        };
        if let Some(sig) = &signature {
            draw_signature_help(ui.ctx(), response.cursor_screen_pos, sig, &theme);
        }

        // ── Hover state machine ────────────────────────────────────────────────
        // The hover popup fires whether the editor is focused or not, after a
        // 500 ms delay. Completion popup and signature help suppress hover.
        let now = Instant::now();
        let raw_hover = if !doc.completion.is_open && signature.is_none() {
            response.hovered_token
        } else {
            None
        };
        let state_changed = doc.hover_state.update(raw_hover, now);
        if state_changed {
            ui.ctx().request_repaint();
        }
        // Schedule a repaint at the exact time the delay elapses so the popup
        // appears without the user having to move the mouse.
        if let Some(remaining) = doc.hover_state.pending_repaint_after(now) {
            ui.ctx().request_repaint_after(remaining);
        }

        if let Some(confirmed) = doc.hover_state.confirmed_token() {
            let sql = doc.buffer.text();
            if let Some(rich) = self.schema_explorer.schema_symbol_index.rich_hover(
                sql,
                confirmed.range,
                &active_schema,
                dialect,
                &self.schema_explorer.schema,
            ) {
                draw_rich_hover_popup(ui.ctx(), confirmed.anchor_rect, confirmed.range, &rich, &theme);
            }
        }

        if dispatch_statement {
            self.dispatch_query();
        } else if dispatch_all {
            self.dispatch_query_all();
        }
        if save_query {
            self.save_query_document();
        }
    }

    pub(super) fn draw_floating_completion_popup(&mut self, ctx: &egui::Context) {
        let dialect = if self.query_capabilities().allows(|caps| caps.query.numbered_parameters) {
            SqlDialect::Postgres
        } else {
            SqlDialect::SQLite
        };
        let theme = self.theme;

        let Some(doc) = self
            .query
            .session
            .documents
            .get_mut(self.query.session.active_document_index)
        else {
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
            apply_completion_item(doc, &item, dialect);
            return;
        }

        if close_popup {
            doc.completion.close();
            return;
        }

        let popup_height = 300.0;
        let popup_width = 420.0;
        let popup_pos = crate::components::clamp_popup_to_screen(
            doc.completion.popup_position,
            egui::vec2(popup_width, popup_height),
            ctx.screen_rect(),
            10.0,
        );

        let mut clicked_item = None;

        let area_resp = egui::Area::new(egui::Id::new("floating_completion_popup"))
            .order(egui::Order::Foreground)
            .fixed_pos(popup_pos)
            .show(ctx, |ui| {
                egui::Frame {
                    // Panel tone contrasts against the flush editor buffer in both themes.
                    fill: theme.surface_panel,
                    rounding: egui::Rounding::same(8.0),
                    stroke: egui::Stroke::new(1.0, theme.border_default),
                    shadow: theme.floating_shadow(),
                    inner_margin: egui::Margin::same(4.0),
                    ..Default::default()
                }
                .show(ui, |ui| {
                    ui.set_max_width(popup_width);
                    ui.set_max_height(popup_height);
                    ui.horizontal(|ui| {
                        ui.label(RichText::new("Suggestions").small().strong().color(theme.text_primary));
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            ui.label(
                                RichText::new("↑↓ navigate · Enter accept · Esc close")
                                    .small()
                                    .color(theme.text_muted),
                            );
                        });
                    });
                    ui.separator();

                    egui::ScrollArea::vertical().max_height(170.0).show(ui, |ui| {
                        let sel_idx = doc.completion.selected_index;
                        let mut hover_idx = None;
                        let mut click_idx = None;

                        for idx in 0..doc.completion.items.len() {
                            let item = &doc.completion.items[idx];
                            let is_selected = idx == sel_idx;
                            let bg = if is_selected {
                                theme.accent_soft
                            } else {
                                egui::Color32::TRANSPARENT
                            };

                            let item_frame = egui::Frame::none()
                                .fill(bg)
                                .rounding(egui::Rounding::same(5.0))
                                .inner_margin(egui::Margin::symmetric(8.0, 4.0))
                                .show(ui, |ui| {
                                    // Fixed row width so label/detail never paint on top of each other.
                                    let row_w = (popup_width - 20.0).max(200.0);
                                    ui.set_width(row_w);
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

                                        egui::Frame::none()
                                            .fill(theme.soft_tint(badge_color))
                                            .rounding(egui::Rounding::same(3.0))
                                            .inner_margin(egui::Margin::symmetric(4.0, 1.0))
                                            .show(ui, |ui| {
                                                ui.label(
                                                    RichText::new(badge_text)
                                                        .font(FontId::monospace(9.0))
                                                        .color(badge_color),
                                                );
                                            });
                                        ui.add_space(6.0);

                                        let detail = item.detail.as_deref().unwrap_or("");
                                        let detail_budget = if detail.is_empty() {
                                            0.0
                                        } else {
                                            (ui.available_width() * 0.42).clamp(72.0, 150.0)
                                        };
                                        let label_budget = (ui.available_width() - detail_budget - 4.0).max(48.0);
                                        let row_h = ui.spacing().interact_size.y.max(16.0);

                                        ui.add_sized(
                                            [label_budget, row_h],
                                            egui::Label::new(
                                                RichText::new(&item.label)
                                                    .font(FontId::monospace(12.5))
                                                    .strong()
                                                    .color(theme.text_primary),
                                            )
                                            .truncate(),
                                        )
                                        .on_hover_text(&item.label);

                                        if !detail.is_empty() {
                                            ui.add_sized(
                                                [detail_budget, row_h],
                                                egui::Label::new(
                                                    RichText::new(detail).font(font_caption()).color(theme.text_muted),
                                                )
                                                .truncate(),
                                            )
                                            .on_hover_text(detail);
                                        }
                                    });
                                });

                            let item_response = item_frame.response.interact(egui::Sense::click());
                            if is_selected {
                                item_response.scroll_to_me(Some(egui::Align::Center));
                            }
                            if item_response.hovered() {
                                hover_idx = Some(idx);
                                item_response.clone().on_hover_ui(|ui| {
                                    ui.set_max_width(320.0);
                                    draw_completion_explanation(ui, item, &theme);
                                });
                            }
                            if item_response.clicked() {
                                click_idx = Some(idx);
                            }
                        }

                        if let Some(idx) = hover_idx {
                            doc.completion.selected_index = idx;
                        }
                        if let Some(idx) = click_idx {
                            clicked_item = doc.completion.items.get(idx).cloned();
                        }
                    });
                    if let Some(item) = doc.completion.current_item() {
                        ui.separator();
                        ui.allocate_ui_with_layout(
                            egui::vec2(ui.available_width(), 58.0),
                            egui::Layout::top_down(egui::Align::Min),
                            |ui| draw_completion_explanation(ui, item, &theme),
                        );
                    }
                    ui.separator();
                    ui.label(
                        RichText::new("Ctrl/Cmd+Space open · ↑↓ navigate · Enter accept · Esc close")
                            .small()
                            .color(theme.text_muted),
                    );
                });
            });

        if let Some(item) = clicked_item {
            apply_completion_item(doc, &item, dialect);
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
}

#[cfg(test)]
mod tests {
    use super::*;

    fn table_completion(replacement_range: (usize, usize)) -> CompletionItem {
        CompletionItem {
            label: "users".to_owned(),
            insert_text: "users".to_owned(),
            kind: CompletionItemKind::Table,
            detail: Some("Table · public.users".to_owned()),
            documentation: Some("Table with 3 columns".to_owned()),
            replacement_range,
            sort_score: 900,
        }
    }

    #[test]
    fn completion_applies_only_to_the_document_version_that_produced_it() {
        let mut document = QueryDocument::new("query-1", "Query 1", "SELECT * FROM us");
        let version = document.buffer.version();
        let item = table_completion((14, 16));
        document.completion.open(
            16,
            version,
            egui::Pos2::ZERO,
            "us".to_owned(),
            vec![item.clone()],
            crate::editor::CompletionTriggerKind::Manual,
        );

        document.buffer.insert(16, "x");
        assert!(!apply_completion_item(&mut document, &item, SqlDialect::Postgres));
        assert_eq!(document.buffer.text(), "SELECT * FROM usx");
        assert!(!document.completion.is_open);
    }

    #[test]
    fn current_completion_replaces_only_its_declared_prefix() {
        let mut document = QueryDocument::new("query-1", "Query 1", "SELECT * FROM us");
        let version = document.buffer.version();
        let item = table_completion((14, 16));
        document.completion.open(
            16,
            version,
            egui::Pos2::ZERO,
            "us".to_owned(),
            vec![item.clone()],
            crate::editor::CompletionTriggerKind::Manual,
        );

        assert!(apply_completion_item(&mut document, &item, SqlDialect::Postgres));
        assert_eq!(document.buffer.text(), "SELECT * FROM users");
        assert_eq!(document.cursor.offset, document.buffer.len_bytes());
        assert!(!document.completion.is_open);
    }
}
