use std::collections::VecDeque;

const MAX_UNDO_HISTORY: usize = 200;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UndoAction {
    Insert { offset: usize, text: String },
    Delete { offset: usize, text: String },
    Group(Vec<UndoAction>),
}

#[derive(Debug, Clone, Default)]
pub struct UndoStack {
    undo_list: VecDeque<UndoAction>,
    redo_list: VecDeque<UndoAction>,
    group_open: bool,
    current_group: Vec<UndoAction>,
}

impl UndoStack {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn begin_group(&mut self) {
        self.group_open = true;
        self.current_group.clear();
    }

    pub fn end_group(&mut self) {
        self.group_open = false;
        if !self.current_group.is_empty() {
            let group = std::mem::take(&mut self.current_group);
            self.push(UndoAction::Group(group));
        }
    }

    pub fn push(&mut self, action: UndoAction) {
        self.redo_list.clear();
        if self.group_open {
            self.current_group.push(action);
        } else {
            if self.undo_list.len() >= MAX_UNDO_HISTORY {
                self.undo_list.pop_front();
            }
            self.undo_list.push_back(action);
        }
    }

    pub fn can_undo(&self) -> bool {
        !self.undo_list.is_empty()
    }

    pub fn can_redo(&self) -> bool {
        !self.redo_list.is_empty()
    }

    pub fn clear(&mut self) {
        self.undo_list.clear();
        self.redo_list.clear();
        self.current_group.clear();
        self.group_open = false;
    }
}

/// TextBuffer provides an indexed text storage with line/column/offset conversions
/// and an integrated undo/redo history stack.
#[derive(Debug, Clone, Default)]
pub struct TextBuffer {
    content: String,
    /// Byte offsets of line starts (line 0 starts at 0).
    line_starts: Vec<usize>,
    pub undo_stack: UndoStack,
    version: u64,
}

impl TextBuffer {
    pub fn new() -> Self {
        let mut buffer = Self {
            content: String::new(),
            line_starts: vec![0],
            undo_stack: UndoStack::new(),
            version: 0,
        };
        buffer.rebuild_line_index();
        buffer
    }

    pub fn from_string(text: impl Into<String>) -> Self {
        let mut buffer = Self {
            content: text.into(),
            line_starts: vec![0],
            undo_stack: UndoStack::new(),
            version: 0,
        };
        buffer.rebuild_line_index();
        buffer
    }

    pub fn text(&self) -> &str {
        &self.content
    }

    pub fn version(&self) -> u64 {
        self.version
    }

    pub fn len_bytes(&self) -> usize {
        self.content.len()
    }

    pub fn len_chars(&self) -> usize {
        self.content.chars().count()
    }

    pub fn is_empty(&self) -> bool {
        self.content.is_empty()
    }

    pub fn line_count(&self) -> usize {
        self.line_starts.len().max(1)
    }

    pub fn line_at(&self, line_idx: usize) -> Option<&str> {
        if line_idx >= self.line_starts.len() {
            return None;
        }
        let start = self.line_starts[line_idx];
        let end = if line_idx + 1 < self.line_starts.len() {
            // Trim trailing \n or \r\n
            let mut next_start = self.line_starts[line_idx + 1];
            if next_start > 0 && self.content.as_bytes().get(next_start - 1) == Some(&b'\n') {
                next_start -= 1;
                if next_start > 0 && self.content.as_bytes().get(next_start - 1) == Some(&b'\r') {
                    next_start -= 1;
                }
            }
            next_start
        } else {
            self.content.len()
        };
        self.content.get(start..end)
    }

    pub fn split_at(&self, byte_offset: usize) -> (&str, &str) {
        let clamped = byte_offset.min(self.content.len());
        self.content.split_at(clamped)
    }

    pub fn line_start_offset(&self, line_idx: usize) -> usize {
        if line_idx >= self.line_starts.len() {
            self.content.len()
        } else {
            self.line_starts[line_idx]
        }
    }

    pub fn line_end_offset(&self, line_idx: usize) -> usize {
        if line_idx >= self.line_starts.len() {
            self.content.len()
        } else if line_idx + 1 < self.line_starts.len() {
            let mut next = self.line_starts[line_idx + 1];
            if next > 0 && self.content.as_bytes().get(next - 1) == Some(&b'\n') {
                next -= 1;
                if next > 0 && self.content.as_bytes().get(next - 1) == Some(&b'\r') {
                    next -= 1;
                }
            }
            next
        } else {
            self.content.len()
        }
    }

    pub fn offset_to_line_col(&self, byte_offset: usize) -> (usize, usize) {
        let clamped = byte_offset.min(self.content.len());
        let line_idx = match self.line_starts.binary_search(&clamped) {
            Ok(idx) => idx,
            Err(idx) => idx.saturating_sub(1),
        };
        let line_start = self.line_starts.get(line_idx).copied().unwrap_or(0);
        let col_byte_offset = clamped.saturating_sub(line_start);
        let col_char_idx = self.content[line_start..clamped].chars().count();
        let _ = col_byte_offset;
        (line_idx, col_char_idx)
    }

    pub fn line_col_to_offset(&self, line_idx: usize, col_char_idx: usize) -> usize {
        if line_idx >= self.line_starts.len() {
            return self.content.len();
        }
        let line_start = self.line_starts[line_idx];
        let line_text = self.line_at(line_idx).unwrap_or("");
        let mut byte_advance = 0;
        for (char_count, (idx, ch)) in line_text.char_indices().enumerate() {
            if char_count == col_char_idx {
                return line_start + idx;
            }
            byte_advance = idx + ch.len_utf8();
        }
        line_start + byte_advance
    }

    pub fn slice(&self, start: usize, end: usize) -> &str {
        let clamped_start = start.min(self.content.len());
        let clamped_end = end.min(self.content.len()).max(clamped_start);
        &self.content[clamped_start..clamped_end]
    }

    pub fn insert(&mut self, offset: usize, text: &str) {
        if text.is_empty() {
            return;
        }
        let clamped = offset.min(self.content.len());
        self.undo_stack.push(UndoAction::Insert {
            offset: clamped,
            text: text.to_owned(),
        });
        self.content.insert_str(clamped, text);
        self.version = self.version.wrapping_add(1);
        self.rebuild_line_index();
    }

    pub fn delete(&mut self, start: usize, end: usize) -> String {
        let clamped_start = start.min(self.content.len());
        let clamped_end = end.min(self.content.len()).max(clamped_start);
        if clamped_start == clamped_end {
            return String::new();
        }
        let deleted = self.content[clamped_start..clamped_end].to_owned();
        self.undo_stack.push(UndoAction::Delete {
            offset: clamped_start,
            text: deleted.clone(),
        });
        self.content.replace_range(clamped_start..clamped_end, "");
        self.version = self.version.wrapping_add(1);
        self.rebuild_line_index();
        deleted
    }

    pub fn replace(&mut self, start: usize, end: usize, text: &str) {
        let clamped_start = start.min(self.content.len());
        let clamped_end = end.min(self.content.len()).max(clamped_start);
        if clamped_start == clamped_end && text.is_empty() {
            return;
        }
        self.undo_stack.begin_group();
        if clamped_start != clamped_end {
            let deleted = self.content[clamped_start..clamped_end].to_owned();
            self.undo_stack.push(UndoAction::Delete {
                offset: clamped_start,
                text: deleted,
            });
            self.content.replace_range(clamped_start..clamped_end, "");
        }
        if !text.is_empty() {
            self.undo_stack.push(UndoAction::Insert {
                offset: clamped_start,
                text: text.to_owned(),
            });
            self.content.insert_str(clamped_start, text);
        }
        self.undo_stack.end_group();
        self.version = self.version.wrapping_add(1);
        self.rebuild_line_index();
    }

    pub fn set_text(&mut self, text: impl Into<String>) {
        let new_text = text.into();
        let len = self.content.len();
        self.replace(0, len, &new_text);
    }

    pub fn undo(&mut self) -> Option<usize> {
        let action = self.undo_stack.undo_list.pop_back()?;
        let restore_offset = self.apply_undo_action(&action, true);
        self.undo_stack.redo_list.push_back(action);
        self.version = self.version.wrapping_add(1);
        self.rebuild_line_index();
        Some(restore_offset)
    }

    pub fn redo(&mut self) -> Option<usize> {
        let action = self.undo_stack.redo_list.pop_back()?;
        let restore_offset = self.apply_undo_action(&action, false);
        self.undo_stack.undo_list.push_back(action);
        self.version = self.version.wrapping_add(1);
        self.rebuild_line_index();
        Some(restore_offset)
    }

    fn apply_undo_action(&mut self, action: &UndoAction, is_undo: bool) -> usize {
        match action {
            UndoAction::Insert { offset, text } => {
                if is_undo {
                    let end = *offset + text.len();
                    if end <= self.content.len() {
                        self.content.replace_range(*offset..end, "");
                    }
                    *offset
                } else {
                    let off = (*offset).min(self.content.len());
                    self.content.insert_str(off, text);
                    off + text.len()
                }
            }
            UndoAction::Delete { offset, text } => {
                if is_undo {
                    let off = (*offset).min(self.content.len());
                    self.content.insert_str(off, text);
                    off + text.len()
                } else {
                    let end = *offset + text.len();
                    if end <= self.content.len() {
                        self.content.replace_range(*offset..end, "");
                    }
                    *offset
                }
            }
            UndoAction::Group(actions) => {
                let mut last_offset = 0;
                if is_undo {
                    for sub in actions.iter().rev() {
                        last_offset = self.apply_undo_action(sub, true);
                    }
                } else {
                    for sub in actions.iter() {
                        last_offset = self.apply_undo_action(sub, false);
                    }
                }
                last_offset
            }
        }
    }

    fn rebuild_line_index(&mut self) {
        self.line_starts.clear();
        self.line_starts.push(0);
        for (idx, b) in self.content.as_bytes().iter().enumerate() {
            if *b == b'\n' && idx < self.content.len() {
                self.line_starts.push(idx + 1);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_text_buffer_insert_delete_and_lines() {
        let mut buf = TextBuffer::from_string("SELECT * FROM users;\nSELECT * FROM orders;");
        assert_eq!(buf.line_count(), 2);
        assert_eq!(buf.line_at(0), Some("SELECT * FROM users;"));
        assert_eq!(buf.line_at(1), Some("SELECT * FROM orders;"));

        buf.insert(0, "-- Comment\n");
        assert_eq!(buf.line_count(), 3);
        assert_eq!(buf.line_at(0), Some("-- Comment"));
        assert_eq!(buf.line_at(1), Some("SELECT * FROM users;"));

        let (line, col) = buf.offset_to_line_col(11);
        assert_eq!(line, 1);
        assert_eq!(col, 0);

        let off = buf.line_col_to_offset(1, 7);
        assert_eq!(off, 18);
        assert_eq!(buf.slice(off, off + 6), "* FROM");
    }

    #[test]
    fn test_text_buffer_undo_redo() {
        let mut buf = TextBuffer::new();
        buf.insert(0, "SELECT 1;");
        assert_eq!(buf.text(), "SELECT 1;");

        buf.insert(9, "\nSELECT 2;");
        assert_eq!(buf.text(), "SELECT 1;\nSELECT 2;");

        let undo_off = buf.undo();
        assert_eq!(undo_off, Some(9));
        assert_eq!(buf.text(), "SELECT 1;");

        let redo_off = buf.redo();
        assert_eq!(redo_off, Some(19));
        assert_eq!(buf.text(), "SELECT 1;\nSELECT 2;");
    }

    #[test]
    fn test_text_buffer_split_at() {
        let buf = TextBuffer::from_string("hello world");
        let (left, right) = buf.split_at(5);
        assert_eq!(left, "hello");
        assert_eq!(right, " world");
    }
}
