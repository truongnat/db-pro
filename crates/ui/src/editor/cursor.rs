use super::buffer::TextBuffer;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct CursorPosition {
    pub offset: usize,
    pub line: usize,
    pub col: usize,
}

impl CursorPosition {
    pub fn new(offset: usize, line: usize, col: usize) -> Self {
        Self { offset, line, col }
    }

    pub fn from_offset(buffer: &TextBuffer, offset: usize) -> Self {
        let clamped = offset.min(buffer.len_bytes());
        let (line, col) = buffer.offset_to_line_col(clamped);
        Self {
            offset: clamped,
            line,
            col,
        }
    }

    pub fn from_line_col(buffer: &TextBuffer, line: usize, col: usize) -> Self {
        let offset = buffer.line_col_to_offset(line, col);
        let (actual_line, actual_col) = buffer.offset_to_line_col(offset);
        Self {
            offset,
            line: actual_line,
            col: actual_col,
        }
    }

    pub fn set_offset(&mut self, buffer: &TextBuffer, offset: usize) {
        let clamped = offset.min(buffer.len_bytes());
        let (line, col) = buffer.offset_to_line_col(clamped);
        self.offset = clamped;
        self.line = line;
        self.col = col;
    }

    pub fn move_left(&mut self, buffer: &TextBuffer) {
        if self.offset > 0 {
            let mut prev = self.offset - 1;
            while prev > 0 && !buffer.text().is_char_boundary(prev) {
                prev -= 1;
            }
            self.set_offset(buffer, prev);
        }
    }

    pub fn move_right(&mut self, buffer: &TextBuffer) {
        let len = buffer.len_bytes();
        if self.offset < len {
            let mut next = self.offset + 1;
            while next < len && !buffer.text().is_char_boundary(next) {
                next += 1;
            }
            self.set_offset(buffer, next);
        }
    }

    pub fn move_up(&mut self, buffer: &TextBuffer) {
        if self.line > 0 {
            let target_line = self.line - 1;
            let offset = buffer.line_col_to_offset(target_line, self.col);
            self.set_offset(buffer, offset);
        } else {
            self.move_home(buffer);
        }
    }

    pub fn move_down(&mut self, buffer: &TextBuffer) {
        if self.line + 1 < buffer.line_count() {
            let target_line = self.line + 1;
            let offset = buffer.line_col_to_offset(target_line, self.col);
            self.set_offset(buffer, offset);
        } else {
            self.move_end(buffer);
        }
    }

    pub fn move_home(&mut self, buffer: &TextBuffer) {
        let start = buffer.line_start_offset(self.line);
        self.set_offset(buffer, start);
    }

    pub fn move_end(&mut self, buffer: &TextBuffer) {
        let end = buffer.line_end_offset(self.line);
        self.set_offset(buffer, end);
    }

    pub fn move_doc_start(&mut self, buffer: &TextBuffer) {
        self.set_offset(buffer, 0);
    }

    pub fn move_doc_end(&mut self, buffer: &TextBuffer) {
        self.set_offset(buffer, buffer.len_bytes());
    }

    pub fn move_word_left(&mut self, buffer: &TextBuffer) {
        if self.offset == 0 {
            return;
        }
        let text = buffer.text();
        let mut idx = self.offset;
        // Skip leading whitespace to the left
        while idx > 0 {
            let prev = prev_char_boundary(text, idx);
            if let Some(ch) = text[prev..idx].chars().next() {
                if ch.is_whitespace() {
                    idx = prev;
                    continue;
                }
            }
            break;
        }
        // Skip word characters or punctuation
        let is_alphanumeric_mode = if idx > 0 {
            let prev = prev_char_boundary(text, idx);
            text[prev..idx]
                .chars()
                .next()
                .is_some_and(|c| c.is_alphanumeric() || c == '_')
        } else {
            true
        };
        while idx > 0 {
            let prev = prev_char_boundary(text, idx);
            if let Some(ch) = text[prev..idx].chars().next() {
                if ch.is_whitespace() {
                    break;
                }
                let is_word_char = ch.is_alphanumeric() || ch == '_';
                if is_word_char != is_alphanumeric_mode {
                    break;
                }
                idx = prev;
            } else {
                break;
            }
        }
        self.set_offset(buffer, idx);
    }

    pub fn move_word_right(&mut self, buffer: &TextBuffer) {
        let len = buffer.len_bytes();
        if self.offset >= len {
            return;
        }
        let text = buffer.text();
        let mut idx = self.offset;
        // Skip leading whitespace to the right
        while idx < len {
            let next = next_char_boundary(text, idx);
            if let Some(ch) = text[idx..next].chars().next() {
                if ch.is_whitespace() {
                    idx = next;
                    continue;
                }
            }
            break;
        }
        // Skip word chars or punctuation
        let is_alphanumeric_mode = if idx < len {
            let next = next_char_boundary(text, idx);
            text[idx..next]
                .chars()
                .next()
                .is_some_and(|c| c.is_alphanumeric() || c == '_')
        } else {
            true
        };
        while idx < len {
            let next = next_char_boundary(text, idx);
            if let Some(ch) = text[idx..next].chars().next() {
                if ch.is_whitespace() {
                    break;
                }
                let is_word_char = ch.is_alphanumeric() || ch == '_';
                if is_word_char != is_alphanumeric_mode {
                    break;
                }
                idx = next;
            } else {
                break;
            }
        }
        self.set_offset(buffer, idx);
    }
}

fn prev_char_boundary(text: &str, mut idx: usize) -> usize {
    if idx == 0 {
        return 0;
    }
    idx -= 1;
    while idx > 0 && !text.is_char_boundary(idx) {
        idx -= 1;
    }
    idx
}

fn next_char_boundary(text: &str, mut idx: usize) -> usize {
    let len = text.len();
    if idx >= len {
        return len;
    }
    idx += 1;
    while idx < len && !text.is_char_boundary(idx) {
        idx += 1;
    }
    idx
}
