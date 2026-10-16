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

    // cc-scan:allow COMPLEXITY,LONG_FUNCTION — word selection expands a contiguous word/non-word grapheme run.
    pub fn select_word_at(&mut self, buffer: &TextBuffer, offset: usize) {
        let text = buffer.text();
        if text.is_empty() {
            self.anchor = 0;
            self.active = 0;
            return;
        }

        let clamped = buffer.floor_grapheme_boundary(offset.min(text.len()));
        let selected_start = if clamped == text.len() {
            buffer.prev_grapheme_boundary(clamped)
        } else {
            clamped
        };
        let selected_end = buffer.next_grapheme_boundary(selected_start);
        let is_word = is_word_grapheme(&text[selected_start..selected_end]);

        let mut start = selected_start;
        while start > 0 {
            let previous_start = buffer.prev_grapheme_boundary(start);
            let grapheme = &text[previous_start..start];
            if is_word_grapheme(grapheme) != is_word || grapheme.chars().any(char::is_whitespace) {
                break;
            }
            start = previous_start;
        }

        let mut end = selected_end;
        while end < text.len() {
            let next_end = buffer.next_grapheme_boundary(end);
            let grapheme = &text[end..next_end];
            if is_word_grapheme(grapheme) != is_word || grapheme.chars().any(char::is_whitespace) {
                break;
            }
            end = next_end;
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
#[path = "tests/selection_tests.rs"]
mod tests;

fn is_word_grapheme(grapheme: &str) -> bool {
    grapheme
        .chars()
        .any(|character| character.is_alphanumeric() || character == '_')
}
