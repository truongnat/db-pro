use super::buffer::TextBuffer;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct SelectionRange {
    pub anchor: usize,
    pub active: usize,
}

impl SelectionRange {
    pub fn new(anchor: usize, active: usize) -> Self {
        Self { anchor, active }
    }

    pub fn point(offset: usize) -> Self {
        Self {
            anchor: offset,
            active: offset,
        }
    }

    pub fn normalized(&self) -> (usize, usize) {
        if self.anchor <= self.active {
            (self.anchor, self.active)
        } else {
            (self.active, self.anchor)
        }
    }

    pub fn is_empty(&self) -> bool {
        self.anchor == self.active
    }

    pub fn len(&self) -> usize {
        let (start, end) = self.normalized();
        end - start
    }

    pub fn collapse_to_active(&mut self) {
        self.anchor = self.active;
    }

    pub fn collapse_to_start(&mut self) {
        let (start, _) = self.normalized();
        self.anchor = start;
        self.active = start;
    }

    pub fn grow_to(&mut self, offset: usize) {
        self.active = offset;
    }

    pub fn select_all(&mut self, len: usize) {
        self.anchor = 0;
        self.active = len;
    }

    pub fn select_word_at(&mut self, buffer: &TextBuffer, offset: usize) {
        let len = buffer.len_bytes();
        if len == 0 {
            self.anchor = 0;
            self.active = 0;
            return;
        }
        let clamped = buffer.floor_char_boundary(offset.min(len));
        let text = buffer.text();

        let ch_at = if clamped < len {
            buffer.char_at(clamped)
        } else {
            buffer.prev_char(clamped)
        };

        let is_word = ch_at.is_some_and(|c| c.is_alphanumeric() || c == '_');

        // Expand left
        let mut start = clamped;
        while start > 0 {
            let prev = buffer.prev_char_boundary(start);
            if let Some(c) = text[prev..start].chars().next() {
                let prev_is_word = c.is_alphanumeric() || c == '_';
                if prev_is_word != is_word || c.is_whitespace() {
                    break;
                }
                start = prev;
            } else {
                break;
            }
        }

        // Expand right
        let mut end = clamped;
        while end < len {
            let next = buffer.next_char_boundary(end);
            if let Some(c) = text[end..next].chars().next() {
                let next_is_word = c.is_alphanumeric() || c == '_';
                if next_is_word != is_word || c.is_whitespace() {
                    break;
                }
                end = next;
            } else {
                break;
            }
        }

        self.anchor = start;
        self.active = end;
    }

    pub fn select_line_at(&mut self, buffer: &TextBuffer, line: usize) {
        let start = buffer.line_start_offset(line);
        let end = if line + 1 < buffer.line_count() {
            buffer.line_start_offset(line + 1)
        } else {
            buffer.len_bytes()
        };
        self.anchor = start;
        self.active = end;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_select_word_at() {
        let text = "SELECT user_id, user_name FROM users;";
        let buf = TextBuffer::from_string(text);

        let mut sel = SelectionRange::default();
        sel.select_word_at(&buf, 9); // inside "user_id"
        assert_eq!(sel.normalized(), (7, 14));
        assert_eq!(buf.slice(sel.anchor, sel.active), "user_id");

        sel.select_word_at(&buf, 0); // "SELECT"
        assert_eq!(sel.normalized(), (0, 6));
        assert_eq!(buf.slice(sel.anchor, sel.active), "SELECT");
    }

    #[test]
    fn test_select_line_at() {
        let text = "SELECT 1;\nSELECT 2;\nSELECT 3;";
        let buf = TextBuffer::from_string(text);

        let mut sel = SelectionRange::default();
        sel.select_line_at(&buf, 1);
        assert_eq!(buf.slice(sel.anchor, sel.active), "SELECT 2;\n");
    }
}
