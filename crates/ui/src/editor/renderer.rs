use super::brackets;
use super::buffer::{EditorSnapshot, TextBuffer};
use super::cursor::CursorPosition;
use super::decorations::DiagnosticSeverity;
use super::diagnostics::Diagnostic;
use super::hover::HoveredSqlToken;
use super::prediction::EditPrediction;
use super::selection::SelectionRange;
use super::syntax::{CachedSqlTokens, SqlDialect, SqlHighlighter, SyntaxToken, SyntaxTokenKind};
use crate::components::interact::text_input_info;
use crate::DbProTheme;
use egui::{
    text::{LayoutJob, TextFormat},
    Event, FontId, Key, Pos2, Rect, Rounding, Sense, Stroke, Ui, Vec2,
};
use std::time::Duration;

const DEFAULT_LINE_HEIGHT: f32 = 20.0;
const LINE_HEIGHT_MULT: f32 = 1.48;
const FONT_SIZE: f32 = 13.5;
const PADDING_LEFT: f32 = 10.0;
const PADDING_TOP: f32 = 6.0;
const PADDING_BOTTOM: f32 = 64.0;
const CARET_WIDTH: f32 = 1.5;
const CARET_BLINK_PERIOD_SECS: f64 = 1.05;
const CARET_SOLID_AFTER_INPUT_SECS: f64 = 0.45;
/// Flush with the query workspace — no card chrome around the buffer.
const EDITOR_ROUNDING: f32 = 0.0;

#[derive(Debug, Clone)]
pub struct SqlEditorResponse {
    pub changed: bool,
    pub cursor_screen_pos: Pos2,
    /// Interactive viewport rect — used to anchor find/completion overlays.
    pub rect: Rect,
    pub wants_completion: bool,
    pub wants_manual_completion: bool,
    pub wants_manual_prediction: bool,
    pub wants_execute_statement: bool,
    pub wants_execute_all: bool,
    pub wants_format: bool,
    pub wants_save: bool,
    pub wants_dismiss_prediction: bool,
    pub accepted_prediction_len: Option<usize>,
    pub hovered_token: Option<HoveredSqlToken>,
    pub focused: bool,
}

impl Default for SqlEditorResponse {
    fn default() -> Self {
        Self {
            changed: false,
            cursor_screen_pos: Pos2::ZERO,
            rect: Rect::NOTHING,
            wants_completion: false,
            wants_manual_completion: false,
            wants_manual_prediction: false,
            wants_execute_statement: false,
            wants_execute_all: false,
            wants_format: false,
            wants_save: false,
            wants_dismiss_prediction: false,
            accepted_prediction_len: None,
            hovered_token: None,
            focused: false,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PairTextResult {
    NotHandled,
    Inserted,
    SkippedExistingClosing,
}

pub struct SqlEditor<'a> {
    pub buffer: &'a mut TextBuffer,
    pub cursor: &'a mut CursorPosition,
    pub selection: &'a mut SelectionRange,
    pub dialect: SqlDialect,
    pub theme: &'a DbProTheme,
    pub diagnostics: &'a [Diagnostic],
    pub prediction: Option<&'a EditPrediction>,
    pub prediction_visible: bool,
    pub cached_tokens: Option<&'a mut CachedSqlTokens>,
    pub search_query: &'a str,
    pub active_search_match_index: usize,
    pub completion_open: bool,
    pub auto_focus: bool,
    pub execution_range: Option<(usize, usize)>,
    pub font_size: f32,
    pub id_salt: &'a str,
}

impl<'a> SqlEditor<'a> {
    // allow: constructor borrows buffer/cursor/selection simultaneously (&mut for each
    // distinct field) from egui's paint pass — an options struct would not reduce borrows, only move them.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        buffer: &'a mut TextBuffer,
        cursor: &'a mut CursorPosition,
        selection: &'a mut SelectionRange,
        dialect: SqlDialect,
        theme: &'a DbProTheme,
        diagnostics: &'a [Diagnostic],
        prediction: Option<&'a EditPrediction>,
        id_salt: &'a str,
    ) -> Self {
        Self {
            buffer,
            cursor,
            selection,
            dialect,
            theme,
            diagnostics,
            prediction,
            prediction_visible: true,
            cached_tokens: None,
            search_query: "",
            active_search_match_index: 0,
            completion_open: false,
            auto_focus: false,
            execution_range: None,
            font_size: FONT_SIZE,
            id_salt,
        }
    }

    pub fn with_cached_tokens(mut self, cached_tokens: &'a mut CachedSqlTokens) -> Self {
        self.cached_tokens = Some(cached_tokens);
        self
    }

    pub fn with_search(mut self, query: &'a str, active_match: usize) -> Self {
        self.search_query = query;
        self.active_search_match_index = active_match;
        self
    }

    pub fn with_completion_open(mut self, completion_open: bool) -> Self {
        self.completion_open = completion_open;
        self
    }

    pub fn with_prediction_visible(mut self, visible: bool) -> Self {
        self.prediction_visible = visible;
        self
    }

    pub fn with_execution_range(mut self, range: Option<(usize, usize)>) -> Self {
        self.execution_range = range;
        self
    }

    pub fn with_auto_focus(mut self, auto_focus: bool) -> Self {
        self.auto_focus = auto_focus;
        self
    }

    pub fn gutter_width(&self, char_width: f32) -> f32 {
        let lines = self.buffer.line_count().max(1);
        let digits = lines.to_string().len().max(2);
        (digits as f32) * char_width + 24.0
    }

    pub fn show(mut self, ui: &mut Ui, available_size: Vec2) -> SqlEditorResponse {
        let mut response = SqlEditorResponse::default();
        // Stable id tied to the interactive rect so focus survives across frames.
        // Requesting focus on a bare `make_persistent_id` that never registers as a
        // widget makes egui clear focus on the next pass (click → one-frame focus → dead).
        let editor_id = ui.make_persistent_id(self.id_salt);

        let font_id = FontId::monospace(self.font_size);
        let glyph_row = ui.fonts(|f| f.row_height(&font_id));
        let line_height = (self.font_size * LINE_HEIGHT_MULT)
            .max(glyph_row)
            .max(DEFAULT_LINE_HEIGHT);
        let char_width = ui.fonts(|f| f.glyph_width(&font_id, 'M')).max(self.font_size * 0.55);
        let gutter_w = self.gutter_width(char_width);
        let line_count = self.buffer.line_count().max(1);

        let max_line_chars = self.buffer.max_line_len_chars().max(40);
        let content_width =
            (gutter_w + PADDING_LEFT + (max_line_chars as f32) * char_width + 96.0).max(available_size.x);
        let content_height = (line_count as f32) * line_height + PADDING_TOP + PADDING_BOTTOM;

        let (viewport, _) = ui.allocate_exact_size(available_size, Sense::hover());
        response.rect = viewport;
        let resp = ui.interact(viewport, editor_id, Sense::click_and_drag());
        if self.auto_focus || resp.clicked() {
            resp.request_focus();
        }
        resp.widget_info(|| text_input_info(true, "SQL query editor"));
        let focused = resp.has_focus();
        response.focused = focused;

        let max_scroll = Vec2::new(
            (content_width - viewport.width()).max(0.0),
            (content_height - viewport.height()).max(0.0),
        );
        let mut scroll: Vec2 = ui
            .ctx()
            .data(|d| d.get_temp::<Vec2>(editor_id.with("scroll")))
            .unwrap_or(Vec2::ZERO);
        if resp.hovered() || focused {
            let delta = ui.input(|i| i.smooth_scroll_delta);
            scroll -= delta;
        }
        scroll = Vec2::new(scroll.x.clamp(0.0, max_scroll.x), scroll.y.clamp(0.0, max_scroll.y));

        let cursor_before = self.cursor.offset;
        let line_before = self.cursor.line;

        // Flush buffer plane: no rounded card; hairline only while focused.
        ui.painter()
            .rect_filled(viewport, Rounding::same(EDITOR_ROUNDING), self.theme.surface_editor);
        if focused {
            ui.painter().rect_stroke(
                viewport,
                Rounding::same(EDITOR_ROUNDING),
                Stroke::new(1.0, self.theme.border_subtle),
            );
        }

        ui.set_clip_rect(ui.clip_rect().intersect(viewport));

        // Content coordinate space (scroll by translating origin).
        let origin = viewport.min - scroll;
        let rect = Rect::from_min_size(origin, Vec2::new(content_width.max(viewport.width()), content_height));

        // Gutter strip (follows scroll so line numbers stay aligned with rows).
        let gutter_rect = Rect::from_min_size(rect.min, Vec2::new(gutter_w, rect.height()));
        ui.painter()
            .rect_filled(gutter_rect, Rounding::ZERO, self.theme.editor_gutter_fill());
        ui.painter().vline(
            rect.min.x + gutter_w,
            viewport.y_range(),
            Stroke::new(
                1.0,
                self.theme
                    .border_subtle
                    .linear_multiply(if self.theme.dark_mode { 0.55 } else { 0.7 }),
            ),
        );

        // Handle Keyboard Events when focused
        if focused {
            let events = ui.input(|i| i.events.clone());
            let modifiers = ui.input(|i| i.modifiers);
            let cmd_or_ctrl = modifiers.command || modifiers.ctrl || modifiers.mac_cmd;

            if let Some(cache) = self.cached_tokens.as_mut() {
                cache.get_or_recompute(self.buffer, self.dialect);
            }

            for event in events {
                match event {
                    Event::Key {
                        key,
                        pressed: true,
                        modifiers: event_mods,
                        ..
                    } => {
                        let is_cmd = event_mods.command || event_mods.ctrl || event_mods.mac_cmd;
                        let shift = event_mods.shift;

                        match key {
                            Key::Enter => {
                                // Completion owns Enter/Tab while open — do not insert a newline.
                                if self.completion_open {
                                    continue;
                                }
                                self.buffer.break_typing_group();
                                if is_cmd && shift {
                                    response.wants_execute_all = true;
                                } else if is_cmd {
                                    response.wants_execute_statement = true;
                                } else {
                                    // Auto-indent matching current line
                                    let current_line_text = self.buffer.line_at(self.cursor.line).unwrap_or("");
                                    let leading_spaces: String = current_line_text
                                        .chars()
                                        .take_while(|c| *c == ' ' || *c == '\t')
                                        .collect();
                                    let insert_text = format!("\n{leading_spaces}");
                                    self.insert_text(&insert_text);
                                    response.changed = true;
                                }
                            }
                            Key::Backspace => {
                                self.buffer.break_typing_group();
                                if !self.selection.is_empty() {
                                    self.delete_selection();
                                    response.changed = true;
                                } else if self.cursor.offset > 0 {
                                    let prev_off = self.buffer.prev_char_boundary(self.cursor.offset);
                                    let delete_end = brackets::auto_pair_range(self.buffer.text(), self.cursor.offset)
                                        .map_or(self.cursor.offset, |(_, end)| end);
                                    self.buffer.delete(prev_off, delete_end);
                                    self.cursor.set_offset(self.buffer, prev_off);
                                    self.selection.collapse_to_active();
                                    response.changed = true;
                                }
                            }
                            Key::Delete => {
                                self.buffer.break_typing_group();
                                if !self.selection.is_empty() {
                                    self.delete_selection();
                                    response.changed = true;
                                } else if self.cursor.offset < self.buffer.len_bytes() {
                                    let next_off = self.buffer.next_char_boundary(self.cursor.offset);
                                    self.buffer.delete(self.cursor.offset, next_off);
                                    self.selection.collapse_to_active();
                                    response.changed = true;
                                }
                            }
                            Key::Tab => {
                                self.buffer.break_typing_group();
                                // If completion popup is open, do not consume Tab; let completion accept it
                                if self.completion_open {
                                    continue;
                                }

                                // Accept AI prediction on Tab if prediction is active and no popup
                                if let Some(pred) = self.prediction {
                                    if !pred.is_empty() && pred.anchor == self.cursor.offset && !shift {
                                        let text_to_insert = pred.accept_full().to_owned();
                                        let len = text_to_insert.len();
                                        self.insert_prediction_text(pred.replacement_range, &text_to_insert);
                                        response.accepted_prediction_len = Some(len);
                                        response.changed = true;
                                        continue;
                                    }
                                }

                                if !self.selection.is_empty() {
                                    if shift {
                                        self.unindent_selection();
                                    } else {
                                        self.indent_selection();
                                    }
                                    response.changed = true;
                                } else if shift {
                                    self.unindent_line();
                                    response.changed = true;
                                } else {
                                    self.insert_text("  ");
                                    response.changed = true;
                                }
                            }
                            Key::ArrowLeft => {
                                self.buffer.break_typing_group();
                                if is_cmd {
                                    self.cursor.move_word_left(self.buffer);
                                } else {
                                    self.cursor.move_left(self.buffer);
                                }
                                self.update_selection(shift);
                            }
                            Key::ArrowRight => {
                                self.buffer.break_typing_group();
                                if event_mods.alt && !self.completion_open {
                                    if let Some(pred) = self.prediction {
                                        if !pred.is_empty() && pred.anchor == self.cursor.offset {
                                            let word = pred.accept_next_word().to_owned();
                                            if !word.is_empty() {
                                                let len = word.len();
                                                self.insert_prediction_text(pred.replacement_range, &word);
                                                response.accepted_prediction_len = Some(len);
                                                response.changed = true;
                                                continue;
                                            }
                                        }
                                    }
                                }
                                if is_cmd {
                                    self.cursor.move_word_right(self.buffer);
                                } else {
                                    self.cursor.move_right(self.buffer);
                                }
                                self.update_selection(shift);
                            }
                            Key::ArrowUp => {
                                if self.completion_open {
                                    continue;
                                }
                                self.buffer.break_typing_group();
                                if is_cmd {
                                    self.cursor.move_doc_start(self.buffer);
                                } else {
                                    self.cursor.move_up(self.buffer);
                                }
                                self.update_selection(shift);
                            }
                            Key::ArrowDown => {
                                if self.completion_open {
                                    continue;
                                }
                                self.buffer.break_typing_group();
                                if event_mods.alt && !self.completion_open {
                                    if let Some(pred) = self.prediction {
                                        if !pred.is_empty() && pred.anchor == self.cursor.offset {
                                            let line = pred.accept_next_line().to_owned();
                                            if !line.is_empty() {
                                                let len = line.len();
                                                self.insert_prediction_text(pred.replacement_range, &line);
                                                response.accepted_prediction_len = Some(len);
                                                response.changed = true;
                                                continue;
                                            }
                                        }
                                    }
                                }
                                if is_cmd {
                                    self.cursor.move_doc_end(self.buffer);
                                } else {
                                    self.cursor.move_down(self.buffer);
                                }
                                self.update_selection(shift);
                            }
                            Key::Home => {
                                self.buffer.break_typing_group();
                                self.cursor.move_home(self.buffer);
                                self.update_selection(shift);
                            }
                            Key::End => {
                                self.buffer.break_typing_group();
                                self.cursor.move_end(self.buffer);
                                self.update_selection(shift);
                            }
                            Key::PageUp => {
                                if self.completion_open {
                                    continue;
                                }
                                self.buffer.break_typing_group();
                                self.cursor.move_page_up(self.buffer, 15);
                                self.update_selection(shift);
                            }
                            Key::PageDown => {
                                if self.completion_open {
                                    continue;
                                }
                                self.buffer.break_typing_group();
                                self.cursor.move_page_down(self.buffer, 15);
                                self.update_selection(shift);
                            }
                            Key::A if is_cmd => {
                                self.buffer.break_typing_group();
                                self.selection.select_all(self.buffer.len_bytes());
                                self.cursor.set_offset(self.buffer, self.buffer.len_bytes());
                            }
                            Key::Z if is_cmd => {
                                self.buffer.break_typing_group();
                                if shift {
                                    if let Some((cur, anchor)) = self.buffer.redo() {
                                        self.cursor.set_offset(self.buffer, cur);
                                        *self.selection = SelectionRange::new(anchor, cur);
                                        response.changed = true;
                                    }
                                } else if let Some((cur, anchor)) = self.buffer.undo() {
                                    self.cursor.set_offset(self.buffer, cur);
                                    *self.selection = SelectionRange::new(anchor, cur);
                                    response.changed = true;
                                }
                            }
                            Key::Space if is_cmd && event_mods.alt => {
                                response.wants_manual_prediction = true;
                            }
                            Key::F if is_cmd && shift => {
                                response.wants_format = true;
                            }
                            Key::S if is_cmd => {
                                response.wants_save = true;
                            }
                            Key::Space if is_cmd => {
                                response.wants_completion = true;
                                response.wants_manual_completion = true;
                            }
                            Key::Escape => {
                                self.selection.collapse_to_active();
                                response.wants_dismiss_prediction = true;
                            }
                            _ => {}
                        }
                    }
                    Event::Text(text) if !cmd_or_ctrl && !text.is_empty() => {
                        match self.handle_pair_text(&text) {
                            PairTextResult::Inserted => {
                                response.changed = true;
                                continue;
                            }
                            PairTextResult::SkippedExistingClosing => continue,
                            PairTextResult::NotHandled => {}
                        }
                        self.type_text(&text);
                        response.changed = true;
                        if text == "." {
                            response.wants_completion = true;
                        }
                    }
                    Event::Ime(egui::ImeEvent::Commit(text)) if !text.is_empty() => {
                        self.type_text(&text);
                        response.changed = true;
                        if text == "." {
                            response.wants_completion = true;
                        }
                    }
                    Event::Ime(egui::ImeEvent::Preedit(_) | egui::ImeEvent::Enabled | egui::ImeEvent::Disabled) => {}
                    Event::Cut if !self.selection.is_empty() => {
                        self.buffer.break_typing_group();
                        let (start, end) = self.selection.normalized();
                        let text = self.buffer.slice(start, end).to_owned();
                        ui.output_mut(|o| o.copied_text = text);
                        self.delete_selection();
                        response.changed = true;
                    }
                    Event::Copy if !self.selection.is_empty() => {
                        let (start, end) = self.selection.normalized();
                        let text = self.buffer.slice(start, end).to_owned();
                        ui.output_mut(|o| o.copied_text = text);
                    }
                    Event::Paste(text) if !text.is_empty() => {
                        self.buffer.break_typing_group();
                        self.insert_text(&text);
                        response.changed = true;
                    }
                    _ => {}
                }
            }
        }

        // Mouse click & drag positioning
        let shift_pressed = ui.input(|i| i.modifiers.shift);
        if resp.double_clicked() {
            if let Some(mouse_pos) = resp.interact_pointer_pos() {
                let offset =
                    self.screen_pos_to_offset(self.buffer, mouse_pos, rect.min, gutter_w, line_height, char_width);
                self.selection.select_word_at(self.buffer, offset);
                self.cursor.set_offset(self.buffer, self.selection.active);
            }
        } else if resp.triple_clicked() {
            if let Some(mouse_pos) = resp.interact_pointer_pos() {
                let rel_y = mouse_pos.y - (rect.min.y + PADDING_TOP);
                let target_line = ((rel_y / line_height).floor() as usize).min(line_count.saturating_sub(1));
                self.selection.select_line_at(self.buffer, target_line);
                self.cursor.set_offset(self.buffer, self.selection.active);
            }
        } else if resp.drag_started() {
            if let Some(mouse_pos) = resp.interact_pointer_pos() {
                let offset =
                    self.screen_pos_to_offset(self.buffer, mouse_pos, rect.min, gutter_w, line_height, char_width);
                self.cursor.set_offset(self.buffer, offset);
                if !shift_pressed {
                    *self.selection = SelectionRange::point(offset);
                } else {
                    self.selection.grow_to(offset);
                }
            }
        } else if resp.dragged() {
            if let Some(mouse_pos) = resp.interact_pointer_pos() {
                let offset =
                    self.screen_pos_to_offset(self.buffer, mouse_pos, rect.min, gutter_w, line_height, char_width);
                self.cursor.set_offset(self.buffer, offset);
                self.selection.grow_to(offset);
            }
        } else if resp.clicked() {
            if let Some(mouse_pos) = resp.interact_pointer_pos() {
                // Check if click was on gutter
                if mouse_pos.x < rect.min.x + gutter_w {
                    let rel_y = mouse_pos.y - (rect.min.y + PADDING_TOP);
                    let target_line = ((rel_y / line_height).floor() as usize).min(line_count.saturating_sub(1));
                    self.selection.select_line_at(self.buffer, target_line);
                    self.cursor.set_offset(self.buffer, self.selection.active);
                } else {
                    let offset =
                        self.screen_pos_to_offset(self.buffer, mouse_pos, rect.min, gutter_w, line_height, char_width);
                    self.cursor.set_offset(self.buffer, offset);
                    if shift_pressed {
                        self.selection.grow_to(offset);
                    } else {
                        self.selection.collapse_to_active();
                    }
                }
            }
        }

        // Current Line Highlight — soft full-row wash (Zed-like).
        let current_line_top = rect.min.y + PADDING_TOP + (self.cursor.line as f32) * line_height;
        let current_line_rect = Rect::from_min_size(
            Pos2::new(rect.min.x, current_line_top),
            Vec2::new(rect.width().max(viewport.width()), line_height),
        );
        ui.painter()
            .rect_filled(current_line_rect, Rounding::ZERO, self.theme.editor_current_line_fill());

        // Search Matches Highlights
        if !self.search_query.trim().is_empty() {
            let query = self.search_query.to_lowercase();
            let text_lower = self.buffer.text().to_lowercase();
            for (idx, (match_start, _)) in text_lower.match_indices(&query).enumerate() {
                let match_end = match_start + query.len();
                let is_active_match = idx == self.active_search_match_index;
                let rects = self.range_to_screen_rects(
                    self.buffer,
                    match_start,
                    match_end,
                    rect.min,
                    gutter_w,
                    line_height,
                    char_width,
                );
                for m_rect in rects {
                    if is_active_match {
                        ui.painter()
                            .rect_filled(m_rect, Rounding::same(2.0), self.theme.accent.linear_multiply(0.4));
                        ui.painter()
                            .rect_stroke(m_rect, Rounding::same(2.0), Stroke::new(1.5, self.theme.accent));
                    } else {
                        ui.painter()
                            .rect_filled(m_rect, Rounding::same(2.0), self.theme.warning.linear_multiply(0.28));
                    }
                }
            }
        }

        if let Some((execution_start, execution_end)) = self.execution_range {
            for execution_rect in self.range_to_screen_rects(
                self.buffer,
                execution_start,
                execution_end,
                rect.min,
                gutter_w,
                line_height,
                char_width,
            ) {
                ui.painter().rect_filled(
                    execution_rect,
                    Rounding::same(1.0),
                    self.theme.accent.linear_multiply(0.08),
                );
            }
        }

        // Selection Highlight
        if !self.selection.is_empty() {
            let (sel_start, sel_end) = self.selection.normalized();
            let rects = self.range_to_screen_rects(
                self.buffer,
                sel_start,
                sel_end,
                rect.min,
                gutter_w,
                line_height,
                char_width,
            );
            for s_rect in rects {
                ui.painter()
                    .rect_filled(s_rect, Rounding::same(2.0), self.theme.editor_selection_fill());
            }
        }

        // Matching delimiter highlight and a subtle warning for an unmatched
        // structural delimiter near the caret.
        if let Some(delimiter) = brackets::delimiter_near_cursor(self.buffer.text(), self.cursor.offset) {
            let delimiter_len = self.buffer.char_at(delimiter.delimiter).map_or(0, char::len_utf8);
            if delimiter_len > 0 {
                let color = if delimiter.matching.is_some() {
                    self.theme.accent
                } else {
                    self.theme.warning
                };
                for highlighted in self.range_to_screen_rects(
                    self.buffer,
                    delimiter.delimiter,
                    delimiter.delimiter + delimiter_len,
                    rect.min,
                    gutter_w,
                    line_height,
                    char_width,
                ) {
                    ui.painter().rect_stroke(
                        highlighted,
                        Rounding::same(2.0),
                        Stroke::new(1.2, color.linear_multiply(0.85)),
                    );
                }
                if let Some(matching) = delimiter.matching {
                    let matching_len = self.buffer.char_at(matching).map_or(0, char::len_utf8);
                    for highlighted in self.range_to_screen_rects(
                        self.buffer,
                        matching,
                        matching + matching_len,
                        rect.min,
                        gutter_w,
                        line_height,
                        char_width,
                    ) {
                        ui.painter().rect_stroke(
                            highlighted,
                            Rounding::same(2.0),
                            Stroke::new(1.2, self.theme.accent.linear_multiply(0.85)),
                        );
                    }
                }
            }
        }

        // Ensure token cache is recomputed if present before borrowing
        if let Some(cache) = self.cached_tokens.as_mut() {
            cache.get_or_recompute(self.buffer, self.dialect);
        }

        // Syntax Highlighting Tokens (zero-copy borrowed slice)
        let highlighter = SqlHighlighter::new(self.dialect);
        let temp_tokens;
        let tokens: &[SyntaxToken] = if let Some(cache) = self.cached_tokens.as_deref() {
            cache.tokens()
        } else {
            temp_tokens = highlighter.tokenize(self.buffer.text());
            &temp_tokens
        };

        if resp.hovered() && !self.completion_open {
            if let Some(pointer) = ui.input(|input| input.pointer.hover_pos()) {
                if pointer.x >= rect.min.x + gutter_w {
                    let offset =
                        self.screen_pos_to_offset(self.buffer, pointer, rect.min, gutter_w, line_height, char_width);
                    let token_idx = tokens.partition_point(|token| token.range.1 <= offset);
                    if let Some(token) = tokens.get(token_idx).filter(|token| {
                        offset >= token.range.0
                            && offset < token.range.1
                            && matches!(
                                token.kind,
                                SyntaxTokenKind::Identifier
                                    | SyntaxTokenKind::Function
                                    | SyntaxTokenKind::Type
                                    | SyntaxTokenKind::Keyword
                            )
                    }) {
                        let anchor_rect = self
                            .range_to_screen_rects(
                                self.buffer,
                                token.range.0,
                                token.range.1,
                                rect.min,
                                gutter_w,
                                line_height,
                                char_width,
                            )
                            .into_iter()
                            .next();
                        if let Some(anchor_rect) = anchor_rect {
                            response.hovered_token = Some(HoveredSqlToken {
                                range: token.range,
                                anchor_rect,
                            });
                        }
                    }
                }
            }
        }

        // Virtualized visible lines (viewport ∩ scrolled content).
        let first_visible_line = (((scroll.y - PADDING_TOP) / line_height).floor() as isize).max(0) as usize;
        let first_visible_line = first_visible_line.saturating_sub(2);
        let last_visible_line =
            ((((scroll.y + viewport.height() - PADDING_TOP) / line_height).ceil() as usize) + 2).min(line_count);

        // Render Visible Text Lines & Gutter Numbers
        for line_idx in first_visible_line..last_visible_line {
            let line_y = rect.min.y + PADDING_TOP + (line_idx as f32) * line_height;

            // Gutter Line Number
            let line_num_str = format!("{}", line_idx + 1);
            let is_curr = line_idx == self.cursor.line;
            ui.painter().text(
                Pos2::new(rect.min.x + gutter_w - 8.0, line_y + line_height * 0.5),
                egui::Align2::RIGHT_CENTER,
                line_num_str,
                FontId::monospace(11.0),
                self.theme.editor_line_number(is_curr),
            );

            // Line Text Layout & Paint (sub-linear binary search token slicing)
            let line_text = self.buffer.line_at(line_idx).unwrap_or("");
            if !line_text.is_empty() {
                let line_start_off = self.buffer.line_start_offset(line_idx);
                let line_end_off = self.buffer.line_end_offset(line_idx);

                let mut job = LayoutJob::default();
                let start_token_idx = tokens.partition_point(|token| token.range.1 <= line_start_off);
                for token in &tokens[start_token_idx..] {
                    if token.range.0 >= line_end_off {
                        break;
                    }
                    let seg_start = token.range.0.max(line_start_off);
                    let seg_end = token.range.1.min(line_end_off);
                    if seg_start < seg_end {
                        let seg_text = &self.buffer.text()[seg_start..seg_end];
                        let color = highlighter.token_color(token.kind, self.theme);
                        job.append(
                            seg_text,
                            0.0,
                            TextFormat {
                                font_id: font_id.clone(),
                                color,
                                ..Default::default()
                            },
                        );
                    }
                }
                let galley = ui.painter().layout_job(job);
                ui.painter().galley(
                    Pos2::new(rect.min.x + gutter_w + PADDING_LEFT, line_y),
                    galley,
                    self.theme.text_primary,
                );
            }
        }

        // Diagnostics Underlines (Squiggles) & Tooltip Hover
        let pointer_pos = ui.input(|i| i.pointer.hover_pos());
        for diag in self.diagnostics {
            let rects = self.range_to_screen_rects(
                self.buffer,
                diag.range.0,
                diag.range.1,
                rect.min,
                gutter_w,
                line_height,
                char_width,
            );
            let diag_color = match diag.severity {
                DiagnosticSeverity::Error => self.theme.danger,
                DiagnosticSeverity::Warning => self.theme.warning,
                _ => self.theme.accent,
            };

            for d_rect in &rects {
                let line_y = d_rect.bottom() - 2.0;
                ui.painter().line_segment(
                    [
                        Pos2::new(d_rect.left(), line_y),
                        Pos2::new(d_rect.right().max(d_rect.left() + 4.0), line_y),
                    ],
                    Stroke::new(1.5, diag_color),
                );

                if let Some(pos) = pointer_pos {
                    if d_rect.contains(pos) {
                        egui::show_tooltip(ui.ctx(), ui.layer_id(), egui::Id::new("diag_hover"), |ui| {
                            ui.horizontal(|ui| {
                                let badge = match diag.severity {
                                    DiagnosticSeverity::Error => "Error",
                                    DiagnosticSeverity::Warning => "Warning",
                                    _ => "Info",
                                };
                                ui.colored_label(diag_color, badge);
                                if let Some(code) = &diag.code {
                                    ui.monospace(code);
                                }
                                ui.label(&diag.message);
                            });
                        });
                    }
                }
            }
        }

        // Caret Screen Position
        let cursor_screen = self.offset_to_screen_pos(
            self.buffer,
            self.cursor.offset,
            rect.min,
            gutter_w,
            line_height,
            char_width,
        );
        response.cursor_screen_pos = Pos2::new(cursor_screen.x, cursor_screen.y + line_height);

        // Inline AI Prediction Ghost Text (suppressed while completion popup is open)
        if !self.completion_open && self.prediction_visible {
            if let Some(pred) = self.prediction {
                if !pred.is_empty() && pred.anchor == self.cursor.offset {
                    let replacement_start = pred.replacement_range.0;
                    let replacement_end = pred.replacement_range.1;
                    let has_replacement = replacement_start < replacement_end
                        && replacement_end <= self.buffer.len_bytes()
                        && self.buffer.is_char_boundary(replacement_start)
                        && self.buffer.is_char_boundary(replacement_end);
                    let ghost_origin = if has_replacement {
                        self.offset_to_screen_pos(
                            self.buffer,
                            replacement_start,
                            rect.min,
                            gutter_w,
                            line_height,
                            char_width,
                        )
                    } else {
                        cursor_screen
                    };
                    if has_replacement {
                        for replacement_rect in self.range_to_screen_rects(
                            self.buffer,
                            replacement_start,
                            replacement_end,
                            rect.min,
                            gutter_w,
                            line_height,
                            char_width,
                        ) {
                            ui.painter().rect_stroke(
                                replacement_rect,
                                Rounding::same(2.0),
                                Stroke::new(1.0, self.theme.accent.linear_multiply(0.35)),
                            );
                        }
                    }
                    let lines: Vec<&str> = pred.text.split('\n').collect();
                    for (idx, line_str) in lines.iter().enumerate() {
                        let ghost_x = if idx == 0 {
                            ghost_origin.x
                        } else {
                            rect.min.x + gutter_w + PADDING_LEFT
                        };
                        let ghost_y = cursor_screen.y + (idx as f32) * line_height;
                        ui.painter().text(
                            Pos2::new(ghost_x, ghost_y),
                            egui::Align2::LEFT_TOP,
                            *line_str,
                            font_id.clone(),
                            self.theme.text_muted.linear_multiply(0.65),
                        );
                    }
                }
            }
        }

        // Caret — thin bar, solid after input, then soft blink (Zed cadence).
        if focused {
            let now = ui.input(|i| i.time);
            let cursor_moved = self.cursor.offset != cursor_before || self.cursor.line != line_before;
            if response.changed || cursor_moved {
                ui.ctx().data_mut(|d| d.insert_temp(editor_id.with("last_input"), now));
            }
            let last_input = ui
                .ctx()
                .data(|d| d.get_temp::<f64>(editor_id.with("last_input")))
                .unwrap_or(now);
            let since = now - last_input;
            let blink_on =
                since < CARET_SOLID_AFTER_INPUT_SECS || ((now / CARET_BLINK_PERIOD_SECS).fract() as f32) < 0.58;
            if blink_on {
                let caret_h = (line_height - 2.0).max(line_height * 0.85);
                let cursor_rect = Rect::from_min_size(
                    Pos2::new(cursor_screen.x, cursor_screen.y + 1.0),
                    Vec2::new(CARET_WIDTH, caret_h),
                );
                ui.painter()
                    .rect_filled(cursor_rect, Rounding::same(0.5), self.theme.accent);
                ui.output_mut(|o| {
                    o.ime = Some(egui::output::IMEOutput {
                        rect: cursor_rect,
                        cursor_rect,
                    });
                });
            }
            // Schedule the next blink toggle only — continuous request_repaint() kept the
            // whole query shell at ~display refresh while the editor was focused.
            const ON_FRAC: f64 = 0.58;
            let next_secs = if since < CARET_SOLID_AFTER_INPUT_SECS {
                CARET_SOLID_AFTER_INPUT_SECS - since
            } else {
                let phase = (now / CARET_BLINK_PERIOD_SECS).fract();
                if phase < ON_FRAC {
                    (ON_FRAC - phase) * CARET_BLINK_PERIOD_SECS
                } else {
                    (1.0 - phase) * CARET_BLINK_PERIOD_SECS
                }
            };
            ui.ctx()
                .request_repaint_after(Duration::from_secs_f64(next_secs.clamp(0.016, 0.55)));
        }

        // Keep caret inside the viewport after edits / moves.
        let caret_local = Pos2::new(
            gutter_w + PADDING_LEFT + (self.cursor.col as f32) * char_width,
            PADDING_TOP + (self.cursor.line as f32) * line_height,
        );
        let margin = 8.0;
        if caret_local.y < scroll.y + margin {
            scroll.y = (caret_local.y - margin).max(0.0);
        } else if caret_local.y + line_height > scroll.y + viewport.height() - margin {
            scroll.y = (caret_local.y + line_height - viewport.height() + margin).max(0.0);
        }
        if caret_local.x < scroll.x + gutter_w + margin {
            scroll.x = (caret_local.x - gutter_w - margin).max(0.0);
        } else if caret_local.x > scroll.x + viewport.width() - margin {
            scroll.x = (caret_local.x - viewport.width() + margin).max(0.0);
        }
        scroll = Vec2::new(scroll.x.clamp(0.0, max_scroll.x), scroll.y.clamp(0.0, max_scroll.y));
        ui.ctx().data_mut(|d| d.insert_temp(editor_id.with("scroll"), scroll));

        // Minimal scrollbar thumbs & overview ruler (diagnostic marks on vertical track).
        paint_editor_scrollbars(ui, viewport, scroll, max_scroll, self.diagnostics, self.buffer, self.theme);

        response
    }

    pub fn offset_to_screen_pos(
        &self,
        buffer: &TextBuffer,
        offset: usize,
        origin: Pos2,
        gutter_w: f32,
        line_height: f32,
        char_width: f32,
    ) -> Pos2 {
        let (line, col) = buffer.offset_to_line_col(offset);
        let x = origin.x + gutter_w + PADDING_LEFT + (col as f32) * char_width;
        let y = origin.y + PADDING_TOP + (line as f32) * line_height;
        Pos2::new(x, y)
    }

    pub fn screen_pos_to_offset(
        &self,
        buffer: &TextBuffer,
        pos: Pos2,
        origin: Pos2,
        gutter_w: f32,
        line_height: f32,
        char_width: f32,
    ) -> usize {
        let rel_x = pos.x - (origin.x + gutter_w + PADDING_LEFT);
        let rel_y = pos.y - (origin.y + PADDING_TOP);
        let target_line = ((rel_y / line_height).floor() as usize).min(buffer.line_count().saturating_sub(1));
        let target_col = (rel_x / char_width).round().max(0.0) as usize;
        buffer.line_col_to_offset(target_line, target_col)
    }

    // allow: parameters are current layout dimensions (origin, gutter, line height,
    // char width) for pure geometric calculation — order mirrors viewport transformation.
    #[allow(clippy::too_many_arguments)]
    pub fn range_to_screen_rects(
        &self,
        buffer: &TextBuffer,
        start_offset: usize,
        end_offset: usize,
        origin: Pos2,
        gutter_w: f32,
        line_height: f32,
        char_width: f32,
    ) -> Vec<Rect> {
        let (start_line, start_col) = buffer.offset_to_line_col(start_offset);
        let (end_line, end_col) = buffer.offset_to_line_col(end_offset);
        let mut rects = Vec::with_capacity(end_line - start_line + 1);

        for l_idx in start_line..=end_line {
            let line_y = origin.y + PADDING_TOP + (l_idx as f32) * line_height;
            let col_start = if l_idx == start_line { start_col } else { 0 };
            let col_end = if l_idx == end_line {
                end_col
            } else {
                buffer.line_at(l_idx).unwrap_or("").chars().count() + 1
            };
            let x1 = origin.x + gutter_w + PADDING_LEFT + (col_start as f32) * char_width;
            let width = ((col_end.saturating_sub(col_start)) as f32) * char_width;
            rects.push(Rect::from_min_size(
                Pos2::new(x1, line_y),
                Vec2::new(width, line_height),
            ));
        }

        rects
    }

    fn insert_text(&mut self, text: &str) {
        if !self.selection.is_empty() {
            self.delete_selection();
        }
        let offset = self.cursor.offset;
        self.buffer.insert(offset, text);
        self.cursor.set_offset(self.buffer, offset + text.len());
        self.selection.collapse_to_active();
    }

    fn handle_pair_text(&mut self, text: &str) -> PairTextResult {
        let mut characters = text.chars();
        let Some(character) = characters.next() else {
            return PairTextResult::NotHandled;
        };
        if characters.next().is_some() {
            return PairTextResult::NotHandled;
        }

        if brackets::closing_pair(character)
            && self.selection.is_empty()
            && self.buffer.char_at(self.cursor.offset) == Some(character)
        {
            let next = self.buffer.next_char_boundary(self.cursor.offset);
            self.cursor.set_offset(self.buffer, next);
            self.selection.collapse_to_active();
            return PairTextResult::SkippedExistingClosing;
        }

        let Some(closing) = brackets::opening_pair(character) else {
            return PairTextResult::NotHandled;
        };
        if self
            .cached_tokens
            .as_ref()
            .is_some_and(|cache| cache.is_in_string_or_comment(self.cursor.offset.saturating_sub(1)))
        {
            return PairTextResult::NotHandled;
        }

        if !self.selection.is_empty() {
            self.wrap_selection(character, closing);
        } else {
            let offset = self.cursor.offset;
            let before = EditorSnapshot {
                cursor_offset: offset,
                anchor_offset: self.selection.anchor,
            };
            let after = EditorSnapshot {
                cursor_offset: offset + character.len_utf8(),
                anchor_offset: offset + character.len_utf8(),
            };
            let pair = format!("{character}{closing}");
            self.buffer.replace_with_snapshot(offset, offset, &pair, before, after);
            self.cursor.set_offset(self.buffer, offset + character.len_utf8());
            self.selection.collapse_to_active();
        }
        PairTextResult::Inserted
    }

    fn wrap_selection(&mut self, opening: char, closing: char) {
        let (start, end) = self.selection.normalized();
        let selected = self.buffer.slice(start, end).to_owned();
        let wrapped = format!("{opening}{selected}{closing}");
        let before = EditorSnapshot {
            cursor_offset: self.cursor.offset,
            anchor_offset: self.selection.anchor,
        };
        let inner_start = start + opening.len_utf8();
        let inner_end = inner_start + selected.len();
        let (anchor, active) = if self.selection.anchor <= self.selection.active {
            (inner_start, inner_end)
        } else {
            (inner_end, inner_start)
        };
        self.buffer.replace_with_snapshot(
            start,
            end,
            &wrapped,
            before,
            EditorSnapshot {
                cursor_offset: active,
                anchor_offset: anchor,
            },
        );
        self.cursor.set_offset(self.buffer, active);
        *self.selection = SelectionRange::new(anchor, active);
    }

    fn insert_prediction_text(&mut self, replacement_range: (usize, usize), text: &str) {
        let (start, end) = replacement_range;
        self.buffer.replace(start, end, text);
        self.cursor.set_offset(self.buffer, start + text.len());
        self.selection.collapse_to_active();
    }

    fn type_text(&mut self, text: &str) {
        if !self.selection.is_empty() {
            self.delete_selection();
        }
        let offset = self.cursor.offset;
        let before = super::buffer::EditorSnapshot {
            cursor_offset: offset,
            anchor_offset: self.selection.anchor,
        };
        let after = super::buffer::EditorSnapshot {
            cursor_offset: offset + text.len(),
            anchor_offset: offset + text.len(),
        };
        self.buffer.type_text(offset, text, before, after);
        self.cursor.set_offset(self.buffer, offset + text.len());
        self.selection.collapse_to_active();
    }

    fn delete_selection(&mut self) {
        let (start, end) = self.selection.normalized();
        if start < end {
            self.buffer.delete(start, end);
            self.cursor.set_offset(self.buffer, start);
            self.selection.collapse_to_active();
        }
    }

    fn update_selection(&mut self, shift: bool) {
        self.selection.active = self.cursor.offset;
        if !shift {
            self.selection.collapse_to_active();
        }
    }

    fn indent_selection(&mut self) {
        let (start, end) = self.selection.normalized();
        let (start_line, _) = self.buffer.offset_to_line_col(start);
        let (end_line, end_col) = self.buffer.offset_to_line_col(end);
        let actual_end_line = if end_col == 0 && end_line > start_line {
            end_line - 1
        } else {
            end_line
        };

        for l_idx in (start_line..=actual_end_line).rev() {
            let l_start = self.buffer.line_start_offset(l_idx);
            self.buffer.insert(l_start, "  ");
        }
        let total_added = (actual_end_line - start_line + 1) * 2;
        *self.selection = SelectionRange::new(start + 2, end + total_added);
        self.cursor.set_offset(self.buffer, self.selection.active);
    }

    fn unindent_selection(&mut self) {
        let (start, end) = self.selection.normalized();
        let (start_line, _) = self.buffer.offset_to_line_col(start);
        let (end_line, end_col) = self.buffer.offset_to_line_col(end);
        let actual_end_line = if end_col == 0 && end_line > start_line {
            end_line - 1
        } else {
            end_line
        };

        let mut total_removed = 0;
        for l_idx in (start_line..=actual_end_line).rev() {
            let l_text = self.buffer.line_at(l_idx).unwrap_or("");
            let l_start = self.buffer.line_start_offset(l_idx);
            if l_text.starts_with("  ") {
                self.buffer.delete(l_start, l_start + 2);
                total_removed += 2;
            } else if l_text.starts_with(' ') || l_text.starts_with('\t') {
                self.buffer.delete(l_start, l_start + 1);
                total_removed += 1;
            }
        }
        *self.selection = SelectionRange::new(start.saturating_sub(2), end.saturating_sub(total_removed));
        self.cursor.set_offset(self.buffer, self.selection.active);
    }

    fn unindent_line(&mut self) {
        let line_text = self.buffer.line_at(self.cursor.line).unwrap_or("");
        let line_start = self.buffer.line_start_offset(self.cursor.line);
        if line_text.starts_with("  ") {
            self.buffer.delete(line_start, line_start + 2);
            self.cursor
                .set_offset(self.buffer, self.cursor.offset.saturating_sub(2));
            self.selection.collapse_to_active();
        } else if line_text.starts_with(' ') || line_text.starts_with('\t') {
            self.buffer.delete(line_start, line_start + 1);
            self.cursor
                .set_offset(self.buffer, self.cursor.offset.saturating_sub(1));
            self.selection.collapse_to_active();
        }
    }
}

fn paint_editor_scrollbars(
    ui: &Ui,
    viewport: Rect,
    scroll: Vec2,
    max_scroll: Vec2,
    diagnostics: &[Diagnostic],
    buffer: &TextBuffer,
    theme: &DbProTheme,
) {
    const THICK: f32 = 4.0;
    const PAD: f32 = 2.0;
    let painter = ui.painter();
    let track_h = (viewport.height() - PAD * 2.0).max(12.0);
    let track_x = viewport.max.x - PAD - THICK;

    // Overview ruler markers for diagnostics (JetBrains / DataGrip error stripes)
    let line_count = buffer.line_count().max(1);
    for diag in diagnostics {
        let (diag_line, _) = buffer.offset_to_line_col(diag.range.0);
        let frac = (diag_line as f32 / line_count as f32).clamp(0.0, 1.0);
        let mark_y = viewport.min.y + PAD + frac * (track_h - 3.0);
        let mark_rect = Rect::from_min_size(
            Pos2::new(track_x - 1.0, mark_y),
            Vec2::new(THICK + 2.0, 3.0),
        );
        let mark_color = match diag.severity {
            DiagnosticSeverity::Error => theme.danger,
            DiagnosticSeverity::Warning => theme.warning,
            _ => theme.accent,
        };
        painter.rect_filled(mark_rect, Rounding::same(1.0), mark_color);
    }

    if max_scroll.y > 1.0 {
        let thumb_h = ((viewport.height() / (viewport.height() + max_scroll.y)) * track_h).clamp(16.0, track_h);
        let t = (scroll.y / max_scroll.y).clamp(0.0, 1.0);
        let thumb_y = viewport.min.y + PAD + t * (track_h - thumb_h);
        let thumb = Rect::from_min_size(
            Pos2::new(track_x, thumb_y),
            Vec2::new(THICK, thumb_h),
        );
        painter.rect_filled(
            thumb,
            Rounding::same(THICK * 0.5),
            theme.text_muted.linear_multiply(0.55),
        );
    }
    if max_scroll.x > 1.0 {
        let track_w = (viewport.width() - PAD * 2.0).max(12.0);
        let thumb_w = ((viewport.width() / (viewport.width() + max_scroll.x)) * track_w).clamp(16.0, track_w);
        let t = (scroll.x / max_scroll.x).clamp(0.0, 1.0);
        let thumb_x = viewport.min.x + PAD + t * (track_w - thumb_w);
        let thumb = Rect::from_min_size(
            Pos2::new(thumb_x, viewport.max.y - PAD - THICK),
            Vec2::new(thumb_w, THICK),
        );
        painter.rect_filled(
            thumb,
            Rounding::same(THICK * 0.5),
            theme.text_muted.linear_multiply(0.45),
        );
    }
}

#[cfg(test)]
#[path = "renderer_tests.rs"]
mod tests;
