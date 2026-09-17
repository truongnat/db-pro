//! Query SQL editor surface and floating completion popup.
use super::query_view::{format_query_document, prediction_replacement_range};
use super::*;
use crate::editor::{CompletionItemKind, CompletionTriggerKind, SqlDialect, SqlEditor};
use crate::query::{CompletionContext, SchemaCompletionProvider};
use egui::RichText;

impl DbProApp {
    pub(super) fn draw_query_editor(&mut self, ui: &mut egui::Ui) {
        let editor_width = ui.max_rect().width();
        // File-editor feel: grow into remaining space instead of capping at ~480px.
        let reserved_for_output = if self.active_query_result().is_some() {
            240.0
        } else {
            140.0
        };
        let editor_height = (ui.available_height() - reserved_for_output).max(220.0);

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

        let available_size = egui::vec2(editor_width.max(280.0), editor_height);

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

    pub(super) fn draw_floating_completion_popup(&mut self, ctx: &egui::Context) {
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

        let popup_height = 220.0;
        let popup_width = 340.0;
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
}
