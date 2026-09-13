use super::brackets;
use super::buffer::{EditorSnapshot, TextBuffer};
use super::cursor::CursorPosition;
use super::decorations::DiagnosticSeverity;
use super::diagnostics::Diagnostic;
use super::prediction::EditPrediction;
use super::selection::SelectionRange;
use super::syntax::{CachedSqlTokens, SqlDialect, SqlHighlighter};
use crate::DbProTheme;
use egui::{
    text::{LayoutJob, TextFormat},
    Event, FontId, Key, Pos2, Rect, Rounding, Sense, Stroke, Ui, Vec2,
};

const DEFAULT_LINE_HEIGHT: f32 = 22.0;
const FONT_SIZE: f32 = 13.5;
const PADDING_LEFT: f32 = 10.0;
const PADDING_TOP: f32 = 6.0;

#[derive(Debug, Clone, Default)]
pub struct SqlEditorResponse {
    pub changed: bool,
    pub cursor_screen_pos: Pos2,
    pub wants_completion: bool,
    pub wants_manual_completion: bool,
    pub wants_manual_prediction: bool,
    pub wants_execute_statement: bool,
    pub wants_execute_all: bool,
    pub wants_format: bool,
    pub wants_dismiss_prediction: bool,
    pub accepted_prediction_len: Option<usize>,
    pub focused: bool,
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
    pub execution_range: Option<(usize, usize)>,
    pub font_size: f32,
    pub id_salt: &'a str,
}

impl<'a> SqlEditor<'a> {
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

    pub fn gutter_width(&self, char_width: f32) -> f32 {
        let lines = self.buffer.line_count().max(1);
        let digits = lines.to_string().len().max(2);
        (digits as f32) * char_width + 24.0
    }

    pub fn show(mut self, ui: &mut Ui, available_size: Vec2) -> SqlEditorResponse {
        let mut response = SqlEditorResponse::default();
        let editor_id = ui.make_persistent_id(self.id_salt);

        let font_id = FontId::monospace(self.font_size);
        let line_height = ui.fonts(|f| f.row_height(&font_id)).max(DEFAULT_LINE_HEIGHT);
        let char_width = ui.fonts(|f| f.glyph_width(&font_id, 'M')).max(self.font_size * 0.6);
        let gutter_w = self.gutter_width(char_width);
        let line_count = self.buffer.line_count().max(1);

        let max_line_chars = self.buffer.max_line_len_chars().max(40);
        let content_width = gutter_w + PADDING_LEFT + (max_line_chars as f32) * char_width + 80.0;
        let content_height = (line_count as f32) * line_height + 60.0;

        let (rect, resp) = ui.allocate_exact_size(available_size, Sense::click_and_drag());
        let focused = ui.memory(|m| m.has_focus(editor_id)) || resp.has_focus();
        response.focused = focused;

        if resp.clicked() {
            ui.memory_mut(|m| m.request_focus(editor_id));
        }

        // Background Outer Frame
        ui.painter()
            .rect_filled(rect, Rounding::same(4.0), self.theme.surface_editor);
        ui.painter().rect_stroke(
            rect,
            Rounding::same(4.0),
            Stroke::new(
                1.0,
                if focused {
                    self.theme.accent.linear_multiply(0.6)
                } else {
                    self.theme.border_subtle
                },
            ),
        );

        // Gutter Outer background
        let gutter_rect = Rect::from_min_size(rect.min, Vec2::new(gutter_w, rect.height()));
        ui.painter().rect_filled(
            gutter_rect,
            Rounding::ZERO,
            self.theme.surface_panel.linear_multiply(0.4),
        );
        ui.painter().vline(
            gutter_rect.right(),
            rect.y_range(),
            Stroke::new(1.0, self.theme.border_subtle.linear_multiply(0.4)),
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
                                self.buffer.break_typing_group();
                                if is_cmd {
                                    self.cursor.move_doc_start(self.buffer);
                                } else {
                                    self.cursor.move_up(self.buffer);
                                }
                                self.update_selection(shift);
                            }
                            Key::ArrowDown => {
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
                                self.buffer.break_typing_group();
                                self.cursor.move_page_up(self.buffer, 15);
                                self.update_selection(shift);
                            }
                            Key::PageDown => {
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
                            Key::Space if event_mods.ctrl && event_mods.alt => {
                                response.wants_manual_prediction = true;
                            }
                            Key::F if is_cmd && shift => {
                                response.wants_format = true;
                            }
                            Key::Space if event_mods.ctrl => {
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
                        if text == "." || text.chars().all(|c| c.is_alphanumeric() || c == '_') {
                            response.wants_completion = true;
                        }
                    }
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

        // Current Line Highlight
        let current_line_top = rect.min.y + PADDING_TOP + (self.cursor.line as f32) * line_height;
        let current_line_rect = Rect::from_min_size(
            Pos2::new(rect.min.x + gutter_w, current_line_top),
            Vec2::new((rect.width() - gutter_w).max(0.0), line_height),
        );
        ui.painter().rect_filled(
            current_line_rect,
            Rounding::ZERO,
            self.theme.surface_hover.linear_multiply(0.35),
        );

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
                    .rect_filled(s_rect, Rounding::same(2.0), self.theme.accent.linear_multiply(0.24));
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

        // Syntax Highlighting Tokens
        let highlighter = SqlHighlighter::new(self.dialect);
        let tokens = if let Some(cache) = self.cached_tokens.as_mut() {
            cache.get_or_recompute(self.buffer, self.dialect).to_vec()
        } else {
            highlighter.tokenize(self.buffer.text())
        };

        // Virtualized Visible Line Range
        let view_top = rect.min.y;
        let view_bottom = rect.max.y;
        let first_visible_line =
            (((view_top - rect.min.y - PADDING_TOP) / line_height).floor() as usize).saturating_sub(2);
        let last_visible_line =
            (((view_bottom - rect.min.y - PADDING_TOP) / line_height).ceil() as usize + 2).min(line_count);

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
                FontId::monospace(11.5),
                if is_curr {
                    self.theme.accent
                } else {
                    self.theme.text_muted
                },
            );

            // Line Text Layout & Paint
            let line_text = self.buffer.line_at(line_idx).unwrap_or("");
            if !line_text.is_empty() {
                let line_start_off = self.buffer.line_start_offset(line_idx);
                let line_end_off = self.buffer.line_end_offset(line_idx);

                let mut job = LayoutJob::default();
                for token in &tokens {
                    if token.range.1 <= line_start_off || token.range.0 >= line_end_off {
                        continue;
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
                DiagnosticSeverity::Information | DiagnosticSeverity::Hint => self.theme.accent,
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

        // Caret Line / Blinking Cursor
        if focused {
            let cursor_rect = Rect::from_min_size(cursor_screen, Vec2::new(2.0, line_height));
            ui.painter()
                .rect_filled(cursor_rect, Rounding::same(1.0), self.theme.accent);
        }

        let _ = (content_width, content_height);
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_caret_geometry_calculation() {
        let text = "SELECT 1;\nSELECT 2;\nSELECT 3;";
        let buf = TextBuffer::from_string(text);
        let mut buf_clone = buf.clone();
        let mut cursor = CursorPosition::default();
        let mut selection = SelectionRange::default();
        let theme = DbProTheme::dark();
        let editor = SqlEditor::new(
            &mut buf_clone,
            &mut cursor,
            &mut selection,
            SqlDialect::Postgres,
            &theme,
            &[],
            None,
            "test",
        );

        let origin = Pos2::new(100.0, 50.0);
        let gutter_w = 40.0;
        let line_height = 20.0;
        let char_width = 8.0;

        // Line 1 ("SELECT 2;"), col 3 ("E") -> offset = 10 + 3 = 13
        let screen_pos = editor.offset_to_screen_pos(&buf, 13, origin, gutter_w, line_height, char_width);
        assert_eq!(screen_pos.x, 100.0 + 40.0 + PADDING_LEFT + 3.0 * 8.0);
        assert_eq!(screen_pos.y, 50.0 + PADDING_TOP + 1.0 * 20.0);

        let resolved_offset = editor.screen_pos_to_offset(&buf, screen_pos, origin, gutter_w, line_height, char_width);
        assert_eq!(resolved_offset, 13);
    }
}
