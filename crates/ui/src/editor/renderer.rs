use super::buffer::TextBuffer;
use super::cursor::CursorPosition;
use super::decorations::DiagnosticSeverity;
use super::diagnostics::Diagnostic;
use super::prediction::EditPrediction;
use super::selection::SelectionRange;
use super::syntax::{SqlDialect, SqlHighlighter};
use crate::DbProTheme;
use egui::{
    text::{LayoutJob, TextFormat},
    Event, FontId, Key, Pos2, Rect, Rounding, Sense, Stroke, Ui, Vec2,
};

const GUTTER_WIDTH: f32 = 46.0;
const LINE_HEIGHT: f32 = 22.0;
const FONT_SIZE: f32 = 13.5;
const PADDING_LEFT: f32 = 10.0;

#[derive(Debug, Clone, Default)]
pub struct SqlEditorResponse {
    pub changed: bool,
    pub cursor_screen_pos: Pos2,
    pub wants_completion: bool,
    pub wants_execute_statement: bool,
    pub wants_execute_all: bool,
    pub wants_format: bool,
    pub focused: bool,
}

pub struct SqlEditor<'a> {
    pub buffer: &'a mut TextBuffer,
    pub cursor: &'a mut CursorPosition,
    pub selection: &'a mut SelectionRange,
    pub dialect: SqlDialect,
    pub theme: &'a DbProTheme,
    pub diagnostics: &'a [Diagnostic],
    pub prediction: Option<&'a EditPrediction>,
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
            font_size: FONT_SIZE,
            id_salt,
        }
    }

    pub fn show(mut self, ui: &mut Ui, available_size: Vec2) -> SqlEditorResponse {
        let mut response = SqlEditorResponse::default();
        let editor_id = ui.make_persistent_id(self.id_salt);

        let (rect, resp) = ui.allocate_exact_size(available_size, Sense::click_and_drag());
        let focused = ui.memory(|m| m.has_focus(editor_id)) || resp.has_focus();
        response.focused = focused;

        if resp.clicked() {
            ui.memory_mut(|m| m.request_focus(editor_id));
        }

        let char_width = self.font_size * 0.60;
        let line_height = LINE_HEIGHT;
        let line_count = self.buffer.line_count();
        let total_content_height = (line_count as f32) * line_height + 40.0;

        // Background
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

        // Gutter background
        let gutter_rect = Rect::from_min_size(rect.min, Vec2::new(GUTTER_WIDTH, rect.height()));
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
                                if !self.selection.is_empty() {
                                    self.delete_selection();
                                    response.changed = true;
                                } else if self.cursor.offset > 0 {
                                    let prev_off = self.cursor.offset - 1;
                                    self.buffer.delete(prev_off, self.cursor.offset);
                                    self.cursor.set_offset(self.buffer, prev_off);
                                    self.selection.collapse_to_active();
                                    response.changed = true;
                                }
                            }
                            Key::Delete => {
                                if !self.selection.is_empty() {
                                    self.delete_selection();
                                    response.changed = true;
                                } else if self.cursor.offset < self.buffer.len_bytes() {
                                    let next_off = self.cursor.offset + 1;
                                    self.buffer.delete(self.cursor.offset, next_off);
                                    self.selection.collapse_to_active();
                                    response.changed = true;
                                }
                            }
                            Key::Tab => {
                                if shift {
                                    // Unindent
                                    self.unindent();
                                    response.changed = true;
                                } else {
                                    // Indent
                                    self.insert_text("  ");
                                    response.changed = true;
                                }
                            }
                            Key::ArrowLeft => {
                                if is_cmd {
                                    self.cursor.move_word_left(self.buffer);
                                } else {
                                    self.cursor.move_left(self.buffer);
                                }
                                self.update_selection(shift);
                            }
                            Key::ArrowRight => {
                                if is_cmd {
                                    self.cursor.move_word_right(self.buffer);
                                } else {
                                    self.cursor.move_right(self.buffer);
                                }
                                self.update_selection(shift);
                            }
                            Key::ArrowUp => {
                                if is_cmd {
                                    self.cursor.move_doc_start(self.buffer);
                                } else {
                                    self.cursor.move_up(self.buffer);
                                }
                                self.update_selection(shift);
                            }
                            Key::ArrowDown => {
                                if is_cmd {
                                    self.cursor.move_doc_end(self.buffer);
                                } else {
                                    self.cursor.move_down(self.buffer);
                                }
                                self.update_selection(shift);
                            }
                            Key::Home => {
                                self.cursor.move_home(self.buffer);
                                self.update_selection(shift);
                            }
                            Key::End => {
                                self.cursor.move_end(self.buffer);
                                self.update_selection(shift);
                            }
                            Key::PageUp => {
                                self.cursor.move_up(self.buffer);
                                self.update_selection(shift);
                            }
                            Key::PageDown => {
                                self.cursor.move_down(self.buffer);
                                self.update_selection(shift);
                            }
                            Key::A if is_cmd => {
                                self.selection.select_all(self.buffer.len_bytes());
                                self.cursor.set_offset(self.buffer, self.buffer.len_bytes());
                            }
                            Key::Z if is_cmd => {
                                if shift {
                                    if let Some(offset) = self.buffer.redo() {
                                        self.cursor.set_offset(self.buffer, offset);
                                        self.selection.collapse_to_active();
                                        response.changed = true;
                                    }
                                } else {
                                    if let Some(offset) = self.buffer.undo() {
                                        self.cursor.set_offset(self.buffer, offset);
                                        self.selection.collapse_to_active();
                                        response.changed = true;
                                    }
                                }
                            }
                            Key::Space if event_mods.ctrl => {
                                response.wants_completion = true;
                            }
                            Key::Escape => {
                                self.selection.collapse_to_active();
                            }
                            _ => {}
                        }
                    }
                    Event::Text(text) if !cmd_or_ctrl && !text.is_empty() => {
                        self.insert_text(&text);
                        response.changed = true;
                        if text == "." || text.chars().all(|c| c.is_alphanumeric() || c == '_') {
                            response.wants_completion = true;
                        }
                    }
                    Event::Cut if !self.selection.is_empty() => {
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
                        self.insert_text(&text);
                        response.changed = true;
                    }
                    _ => {}
                }
            }
        }

        // Mouse click positioning
        if resp.clicked() {
            if let Some(mouse_pos) = resp.interact_pointer_pos() {
                let rel_x = mouse_pos.x - (rect.min.x + GUTTER_WIDTH + PADDING_LEFT);
                let rel_y = mouse_pos.y - (rect.min.y + 6.0);
                let target_line =
                    ((rel_y / line_height).floor() as usize).min(self.buffer.line_count().saturating_sub(1));
                let target_col = (rel_x / char_width).round().max(0.0) as usize;
                let offset = self.buffer.line_col_to_offset(target_line, target_col);
                self.cursor.set_offset(self.buffer, offset);
                self.selection.collapse_to_active();
            }
        }

        // Current Line Highlight
        let current_line_top = rect.min.y + 6.0 + (self.cursor.line as f32) * line_height;
        let current_line_rect = Rect::from_min_size(
            Pos2::new(rect.min.x + GUTTER_WIDTH, current_line_top),
            Vec2::new(rect.width() - GUTTER_WIDTH, line_height),
        );
        ui.painter().rect_filled(
            current_line_rect,
            Rounding::ZERO,
            self.theme.surface_hover.linear_multiply(0.4),
        );

        // Selection Highlight
        if !self.selection.is_empty() {
            let (sel_start, sel_end) = self.selection.normalized();
            let (start_line, start_col) = self.buffer.offset_to_line_col(sel_start);
            let (end_line, end_col) = self.buffer.offset_to_line_col(sel_end);

            for line_idx in start_line..=end_line {
                let line_y = rect.min.y + 6.0 + (line_idx as f32) * line_height;
                let line_len_chars = self.buffer.line_at(line_idx).unwrap_or("").chars().count();
                let col_start = if line_idx == start_line { start_col } else { 0 };
                let col_end = if line_idx == end_line {
                    end_col
                } else {
                    line_len_chars + 1
                };
                let sel_rect = Rect::from_min_size(
                    Pos2::new(
                        rect.min.x + GUTTER_WIDTH + PADDING_LEFT + (col_start as f32) * char_width,
                        line_y,
                    ),
                    Vec2::new(((col_end.saturating_sub(col_start)) as f32) * char_width, line_height),
                );
                ui.painter()
                    .rect_filled(sel_rect, Rounding::same(2.0), self.theme.accent.linear_multiply(0.24));
            }
        }

        // Syntax Highlighting Tokens
        let highlighter = SqlHighlighter::new(self.dialect);
        let tokens = highlighter.tokenize(self.buffer.text());

        // Render Text Lines
        for line_idx in 0..self.buffer.line_count() {
            let line_y = rect.min.y + 6.0 + (line_idx as f32) * line_height;

            // Gutter Line Number
            let line_num_str = format!("{:>3}", line_idx + 1);
            let is_curr = line_idx == self.cursor.line;
            ui.painter().text(
                Pos2::new(rect.min.x + GUTTER_WIDTH - 8.0, line_y + line_height * 0.5),
                egui::Align2::RIGHT_CENTER,
                line_num_str,
                FontId::monospace(11.5),
                if is_curr {
                    self.theme.accent
                } else {
                    self.theme.text_muted
                },
            );

            // Line Text
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
                                font_id: FontId::monospace(self.font_size),
                                color,
                                ..Default::default()
                            },
                        );
                    }
                }
                let galley = ui.painter().layout_job(job);
                ui.painter().galley(
                    Pos2::new(rect.min.x + GUTTER_WIDTH + PADDING_LEFT, line_y),
                    galley,
                    self.theme.text_primary,
                );
            }
        }

        // Diagnostics Underlines (Squiggles)
        for diag in self.diagnostics {
            let (start_line, start_col) = self.buffer.offset_to_line_col(diag.range.0);
            let (end_line, end_col) = self.buffer.offset_to_line_col(diag.range.1);
            let diag_color = match diag.severity {
                DiagnosticSeverity::Error => self.theme.danger,
                DiagnosticSeverity::Warning => self.theme.warning,
                DiagnosticSeverity::Information | DiagnosticSeverity::Hint => self.theme.accent,
            };
            for line_idx in start_line..=end_line {
                let line_y = rect.min.y + 6.0 + (line_idx as f32) * line_height + line_height - 2.0;
                let col_start = if line_idx == start_line { start_col } else { 0 };
                let col_end = if line_idx == end_line {
                    end_col.max(col_start + 1)
                } else {
                    col_start + 4
                };
                let x1 = rect.min.x + GUTTER_WIDTH + PADDING_LEFT + (col_start as f32) * char_width;
                let x2 = rect.min.x + GUTTER_WIDTH + PADDING_LEFT + (col_end as f32) * char_width;
                ui.painter().line_segment(
                    [Pos2::new(x1, line_y), Pos2::new(x2, line_y)],
                    Stroke::new(1.5, diag_color),
                );
            }
        }

        // Cursor Position calculation
        let cursor_x = rect.min.x + GUTTER_WIDTH + PADDING_LEFT + (self.cursor.col as f32) * char_width;
        let cursor_y = rect.min.y + 6.0 + (self.cursor.line as f32) * line_height;
        response.cursor_screen_pos = Pos2::new(cursor_x, cursor_y + line_height);

        // Inline AI Prediction Ghost Text
        if let Some(pred) = self.prediction {
            if !pred.is_empty() && pred.anchor == self.cursor.offset {
                let ghost_x = cursor_x;
                let ghost_y = cursor_y;
                ui.painter().text(
                    Pos2::new(ghost_x, ghost_y),
                    egui::Align2::LEFT_TOP,
                    &pred.text,
                    FontId::monospace(self.font_size),
                    self.theme.text_muted.linear_multiply(0.6),
                );
            }
        }

        // Caret Line / Blinking Cursor
        if focused {
            let cursor_rect = Rect::from_min_size(Pos2::new(cursor_x, cursor_y), Vec2::new(2.0, line_height));
            ui.painter()
                .rect_filled(cursor_rect, Rounding::same(1.0), self.theme.accent);
        }

        let _ = total_content_height;
        response
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

    fn unindent(&mut self) {
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
