use super::buffer::TextBuffer;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct CursorPosition {
    pub offset: usize,
    pub line: usize,
    pub col: usize,
    pub preferred_column: Option<usize>,
}

impl CursorPosition {
    pub fn new(offset: usize, line: usize, col: usize) -> Self {
        Self {
            offset,
            line,
            col,
            preferred_column: None,
        }
    }

    pub fn from_offset(buffer: &TextBuffer, offset: usize) -> Self {
        let clamped = buffer.floor_char_boundary(offset.min(buffer.len_bytes()));
        let (line, col) = buffer.offset_to_line_col(clamped);
        Self {
            offset: clamped,
            line,
            col,
            preferred_column: None,
        }
    }

    pub fn from_line_col(buffer: &TextBuffer, line: usize, col: usize) -> Self {
        let offset = buffer.line_col_to_offset(line, col);
        let (actual_line, actual_col) = buffer.offset_to_line_col(offset);
        Self {
            offset,
            line: actual_line,
            col: actual_col,
            preferred_column: None,
        }
    }

    pub fn set_offset(&mut self, buffer: &TextBuffer, offset: usize) {
        let clamped = buffer.floor_char_boundary(offset.min(buffer.len_bytes()));
        let (line, col) = buffer.offset_to_line_col(clamped);
        self.offset = clamped;
        self.line = line;
        self.col = col;
        self.preferred_column = None;
    }

    pub fn move_left(&mut self, buffer: &TextBuffer) {
        if self.offset > 0 {
            let prev = buffer.prev_char_boundary(self.offset);
            self.set_offset(buffer, prev);
        }
    }

    pub fn move_right(&mut self, buffer: &TextBuffer) {
        let len = buffer.len_bytes();
        if self.offset < len {
            let next = buffer.next_char_boundary(self.offset);
            self.set_offset(buffer, next);
        }
    }

    pub fn move_up(&mut self, buffer: &TextBuffer) {
        if self.line > 0 {
            let pref_col = self.preferred_column.unwrap_or(self.col);
            let target_line = self.line - 1;
            let offset = buffer.line_col_to_offset(target_line, pref_col);
            let (actual_line, actual_col) = buffer.offset_to_line_col(offset);
            self.offset = offset;
            self.line = actual_line;
            self.col = actual_col;
            self.preferred_column = Some(pref_col);
        } else {
            self.move_home(buffer);
        }
    }

    pub fn move_down(&mut self, buffer: &TextBuffer) {
        if self.line + 1 < buffer.line_count() {
            let pref_col = self.preferred_column.unwrap_or(self.col);
            let target_line = self.line + 1;
            let offset = buffer.line_col_to_offset(target_line, pref_col);
            let (actual_line, actual_col) = buffer.offset_to_line_col(offset);
            self.offset = offset;
            self.line = actual_line;
            self.col = actual_col;
            self.preferred_column = Some(pref_col);
        } else {
            self.move_end(buffer);
        }
    }

    pub fn move_page_up(&mut self, buffer: &TextBuffer, page_lines: usize) {
        let pref_col = self.preferred_column.unwrap_or(self.col);
        let target_line = self.line.saturating_sub(page_lines);
        let offset = buffer.line_col_to_offset(target_line, pref_col);
        let (actual_line, actual_col) = buffer.offset_to_line_col(offset);
        self.offset = offset;
        self.line = actual_line;
        self.col = actual_col;
        self.preferred_column = Some(pref_col);
    }

    pub fn move_page_down(&mut self, buffer: &TextBuffer, page_lines: usize) {
        let pref_col = self.preferred_column.unwrap_or(self.col);
        let target_line = (self.line + page_lines).min(buffer.line_count().saturating_sub(1));
        let offset = buffer.line_col_to_offset(target_line, pref_col);
        let (actual_line, actual_col) = buffer.offset_to_line_col(offset);
        self.offset = offset;
        self.line = actual_line;
        self.col = actual_col;
        self.preferred_column = Some(pref_col);
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
            let prev = buffer.prev_char_boundary(idx);
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
            let prev = buffer.prev_char_boundary(idx);
            text[prev..idx]
                .chars()
                .next()
                .is_some_and(|c| c.is_alphanumeric() || c == '_')
        } else {
            true
        };
        while idx > 0 {
            let prev = buffer.prev_char_boundary(idx);
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
            let next = buffer.next_char_boundary(idx);
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
            let next = buffer.next_char_boundary(idx);
            text[idx..next]
                .chars()
                .next()
                .is_some_and(|c| c.is_alphanumeric() || c == '_')
        } else {
            true
        };
        while idx < len {
            let next = buffer.next_char_boundary(idx);
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cursor_vertical_movement_preserves_preferred_column() {
        let text = "SELECT very_long_column_name_here\nFROM tbl\nWHERE id = 100;";
        let buf = TextBuffer::from_string(text);

        let mut cursor = CursorPosition::from_line_col(&buf, 0, 20);
        assert_eq!(cursor.line, 0);
        assert_eq!(cursor.col, 20);

        // Move down to short line (len 8 chars)
        cursor.move_down(&buf);
        assert_eq!(cursor.line, 1);
        assert_eq!(cursor.col, 8); // clamped to end of "FROM tbl"
        assert_eq!(cursor.preferred_column, Some(20));

        // Move down to third line
        cursor.move_down(&buf);
        assert_eq!(cursor.line, 2);
        assert_eq!(cursor.col, 15); // "WHERE id = 100;" is 15 chars, clamped to 15
        assert_eq!(cursor.preferred_column, Some(20));

        // Move up back to first line
        cursor.move_up(&buf);
        cursor.move_up(&buf);
        assert_eq!(cursor.line, 0);
        assert_eq!(cursor.col, 20); // restored preferred column 20!
    }

    #[test]
    fn test_cursor_utf8_boundary_safety() {
        let text = "SELECT 'Tiếng Việt';";
        let buf = TextBuffer::from_string(text);

        let mut cursor = CursorPosition::from_offset(&buf, 0);
        // Step through characters
        for _ in 0..20 {
            cursor.move_right(&buf);
            assert!(buf.text().is_char_boundary(cursor.offset));
        }
        for _ in 0..20 {
            cursor.move_left(&buf);
            assert!(buf.text().is_char_boundary(cursor.offset));
        }
    }
}
