use super::query_editor_support::{draw_rich_hover_popup, draw_signature_help};
use super::query_view::{format_query_document, prediction_replacement_range};
use super::*;
use crate::editor::{CompletionIntent, CompletionTriggerKind, EditorInteractionPolicy, SqlDialect, SqlEditor};
use crate::query::{CompletionContext, SchemaCompletionProvider};
use std::time::Instant;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub(super) struct QueryEditorEffects {
    pub(super) dispatch_statement: bool,
    pub(super) dispatch_all: bool,
    pub(super) save_query: bool,
}

pub(super) struct QueryEditorSurfaceContext<'a> {
    pub(super) theme: DbProTheme,
    pub(super) query_editor: &'a mut QueryEditorState,
    pub(super) query_session: &'a mut QuerySessionState,
    pub(super) preferences: &'a PreferencesState,
    pub(super) schema: &'a SchemaExplorerState,
    pub(super) task_bridge: &'a mut TaskBridge,
    pub(super) active_schema: &'a str,
    pub(super) dialect: SqlDialect,
}

impl<'a> QueryEditorSurfaceContext<'a> {
    pub(super) fn draw_query_editor(&mut self, ui: &mut egui::Ui) -> QueryEditorEffects {
        let editor_width = ui.max_rect().width();
        // Editor owns the allocated region from draw_query — no permanent output reserve.
        let editor_height = ui.available_height().max(120.0);

        let dialect = self.dialect;
        let uses_positional_editor = !matches!(dialect, SqlDialect::Postgres);
        let active_schema = self.active_schema.to_owned();

        let theme = self.theme;
        let font_size = self.query_editor.editor_font_size;
        let auto_focus = self.query_editor.query_focus_editor_on_open;
        let mut dispatch_statement = false;
        let mut dispatch_all = false;
        let mut save_query = false;

        let available_size = egui::vec2(editor_width.max(280.0), editor_height);

        let doc_index = self.query_session.active_document_index;
        let doc = &mut self.query_session.documents[doc_index];

        let search_query = self.query_editor.editor_search.clone();
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
        self.query_editor.query_focus_editor_on_open = false;
        self.query_editor.query_editor_rect = response.rect;

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

        self.query_editor.query_editor_focused = response.focused;
        self.query_editor.query_cursor_line = doc.cursor.line + 1;
        self.query_editor.query_cursor_column = doc.cursor.col + 1;

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
                self.query_session.selected_text = doc.buffer.slice(start, end).to_owned();
            } else {
                self.query_session.selected_text.clear();
            }
        }
        if response.wants_format {
            if !doc.selection.is_empty() {
                let (start, end) = doc.selection.normalized();
                self.query_session.selected_text = doc.buffer.slice(start, end).to_owned();
            } else {
                self.query_session.selected_text.clear();
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
                &self.schema.schema,
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
                let replacement_range = prediction_replacement_range(&doc.buffer, anchor, manual);
                let command = UiCommand::RequestSqlPrediction {
                    request_id: req_id,
                    document_id: doc.id.clone(),
                    document_version,
                    anchor,
                    replacement_range,
                    context: ai_context,
                };
                if self.task_bridge.send_best_effort(command) {
                    doc.prediction_context_fingerprint = Some(fingerprint);
                    doc.prediction_last_request_fingerprint = Some(fingerprint);
                    doc.prediction_last_request_at = Some(Instant::now());
                    doc.pending_prediction_request = Some(req_id);
                    doc.prediction_request_started_at = Some(Instant::now());
                    doc.prediction_requests_sent = doc.prediction_requests_sent.saturating_add(1);
                }
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
                schema_summary: &self.schema.schema,
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
            self.schema.schema_symbol_index.signature_help(
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
            if let Some(rich) = self.schema.schema_symbol_index.rich_hover(
                sql,
                confirmed.range,
                &active_schema,
                dialect,
                &self.schema.schema,
            ) {
                draw_rich_hover_popup(ui.ctx(), confirmed.anchor_rect, confirmed.range, &rich, &theme);
            }
        }

        QueryEditorEffects {
            dispatch_statement,
            dispatch_all,
            save_query,
        }
    }
}
