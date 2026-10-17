use unicode_segmentation::UnicodeSegmentation;

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

    // cc-scan:allow COMPLEXITY,LONG_FUNCTION — classifier/dispatch ladder — one case per branch
    pub fn select_word_at(&mut self, buffer: &TextBuffer, offset: usize) {
        let len = buffer.len_bytes();
        if len == 0 {
            self.anchor = 0;
            self.active = 0;
            return;
        }
        let clamped = buffer.floor_char_boundary(offset.min(len));
        let text = buffer.text();

        // Expand by grapheme cluster, not per-char: combining marks (U+0301,
        // Vietnamese combining tones) are not alphanumeric and would split
        // words like "café" at the mark. A cluster is word-classed by its
        // base char; the cluster itself becomes indivisible.
        let is_word_class = |grapheme: &str| grapheme.chars().next().is_some_and(|c| c.is_alphanumeric() || c == '_');
        let is_space_class = |grapheme: &str| grapheme.chars().all(char::is_whitespace);

        // `ends[i]..ends[i+1]` is grapheme i. `partition_point` lands a
        // mid-cluster offset on the owning cluster; an offset at end-of-
        // buffer selects the last grapheme (previous `prev_char` behavior).
        let ends: Vec<usize> = text
            .grapheme_indices(true)
            .map(|(i, _)| i)
            .chain(std::iter::once(len))
            .collect();
        let at = ends
            .partition_point(|end| *end <= clamped)
            .saturating_sub(1)
            .min(ends.len() - 2);
        let clicked_start = ends[at];
        let clicked_end = ends[at + 1];
        let clicked_is_word = is_word_class(&text[clicked_start..clicked_end]);

        // Expand left/right across same-class graphemes; whitespace always breaks.
        let mut start = at;
        while start > 0 {
            let g = &text[ends[start - 1]..ends[start]];
            if is_space_class(g) || is_word_class(g) != clicked_is_word {
                break;
            }
            start -= 1;
        }
        let mut end = at + 1;
        while end + 1 < ends.len() {
            let g = &text[ends[end]..ends[end + 1]];
            if is_space_class(g) || is_word_class(g) != clicked_is_word {
                break;
            }
            end += 1;
        }

        self.anchor = ends[start];
        self.active = ends[end];
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
