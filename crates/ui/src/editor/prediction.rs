use std::hash::{Hash, Hasher};

use serde::{Deserialize, Serialize};

use crate::runtime::RequestId;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum PredictionMode {
    Eager,
    #[default]
    Off,
    Subtle,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PredictionStatus {
    #[default]
    Idle,
    Generating(RequestId),
    Ready,
    Rejected,
}

pub const MAX_SQL_CHARS: usize = 4000;
pub const MAX_SQL_BEFORE_CHARS: usize = MAX_SQL_CHARS;
pub const MAX_SQL_AFTER_CHARS: usize = 500;
pub const MAX_REFERENCED_TABLES: usize = 10;
pub const MAX_COLUMNS_PER_TABLE: usize = 30;
pub const MAX_FK_NEIGHBORS: usize = 15;
pub const MAX_CTES: usize = 20;
pub const MAX_PREDICTION_CHARS: usize = 4000;

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct AiSqlContext {
    pub sql_before_cursor: String,
    pub sql_after_cursor: String,
    pub current_statement: String,
    pub active_schema: String,
    pub dialect: String,
    pub referenced_tables: Vec<String>,
    pub table_aliases: std::collections::HashMap<String, String>,
    pub relevant_columns: Vec<String>,
    pub fk_neighbors: Vec<String>,
    pub cte_names: Vec<String>,
}

impl AiSqlContext {
    /// A process-local, deterministic fingerprint for request deduplication.
    /// HashMap iteration is sorted so equivalent contexts hash identically.
    pub fn fingerprint(&self, document_version: u64, anchor: usize) -> u64 {
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        document_version.hash(&mut hasher);
        anchor.hash(&mut hasher);
        self.sql_before_cursor.hash(&mut hasher);
        self.sql_after_cursor.hash(&mut hasher);
        self.current_statement.hash(&mut hasher);
        self.active_schema.hash(&mut hasher);
        self.dialect.hash(&mut hasher);
        self.referenced_tables.hash(&mut hasher);
        let mut aliases: Vec<_> = self.table_aliases.iter().collect();
        aliases.sort_unstable_by(|left, right| left.0.cmp(right.0).then_with(|| left.1.cmp(right.1)));
        aliases.hash(&mut hasher);
        self.relevant_columns.hash(&mut hasher);
        self.fk_neighbors.hash(&mut hasher);
        self.cte_names.hash(&mut hasher);
        hasher.finish()
    }
}

/// Removes text already present immediately around the caret without trimming
/// meaningful indentation or whitespace inside the provider response.
pub fn normalize_prediction_overlap(before: &str, after: &str, prediction: &str) -> String {
    let mut normalized = prediction.to_owned();
    if let Some(overlap_len) = longest_suffix_prefix_overlap(before, &normalized) {
        normalized.drain(..overlap_len);
    }
    if let Some(overlap_len) = longest_suffix_prefix_overlap(&normalized, after) {
        let keep = normalized.len().saturating_sub(overlap_len);
        normalized.truncate(keep);
    }
    normalized
}

fn longest_suffix_prefix_overlap(left: &str, right: &str) -> Option<usize> {
    for (start, _) in left.char_indices().rev() {
        let overlap_len = left.len() - start;
        if overlap_len == 0 || overlap_len > right.len() || !right.is_char_boundary(overlap_len) {
            continue;
        }
        let overlap = &left[start..];
        if overlap.trim().is_empty() {
            continue;
        }
        if overlap == &right[..overlap_len] {
            return Some(overlap_len);
        }
    }
    None
}

pub fn is_prediction_acceptable(prediction: &str) -> bool {
    let trimmed = prediction.trim();
    if trimmed.is_empty() || prediction.chars().count() > MAX_PREDICTION_CHARS {
        return false;
    }
    let first_line = trimmed.lines().next().unwrap_or_default().to_ascii_lowercase();
    !first_line.starts_with("here is")
        && !first_line.starts_with("here's")
        && !first_line.starts_with("suggestion:")
        && !first_line.starts_with("answer:")
        && !trimmed.contains("```")
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EditPrediction {
    pub anchor: usize,
    pub replacement_range: (usize, usize),
    pub text: String,
    pub request_id: Option<RequestId>,
    pub document_version: u64,
}

impl EditPrediction {
    pub fn new(anchor: usize, text: impl Into<String>, request_id: Option<RequestId>) -> Self {
        let t = text.into();
        Self {
            anchor,
            replacement_range: (anchor, anchor),
            text: t,
            request_id,
            document_version: 0,
        }
    }

    pub fn with_range_and_version(
        anchor: usize,
        replacement_range: (usize, usize),
        text: impl Into<String>,
        request_id: Option<RequestId>,
        document_version: u64,
    ) -> Self {
        Self {
            anchor,
            replacement_range,
            text: text.into(),
            request_id,
            document_version,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.text.is_empty()
    }

    pub fn accept_full(&self) -> &str {
        &self.text
    }

    pub fn accept_next_word(&self) -> &str {
        let mut saw_word = false;
        for (offset, character) in self.text.char_indices() {
            let is_separator = character.is_whitespace() || matches!(character, '(' | ',' | ';');
            if !saw_word {
                if !is_separator {
                    saw_word = true;
                }
            } else if is_separator {
                return &self.text[..offset];
            }
        }
        &self.text
    }

    pub fn accept_next_line(&self) -> &str {
        if let Some(pos) = self.text.find('\n') {
            &self.text[..=pos]
        } else {
            &self.text
        }
    }

    pub fn accept_word(&self) -> &str {
        self.accept_next_word()
    }

    pub fn accept_line(&self) -> &str {
        self.accept_next_line()
    }

    pub fn consume(&mut self, accepted_len: usize) {
        if accepted_len >= self.text.len() {
            self.text.clear();
        } else {
            self.text = self.text[accepted_len..].to_owned();
            let replacement_start = self.replacement_range.0;
            let replacement_is_non_empty = self.replacement_range.0 != self.replacement_range.1;
            self.anchor = if replacement_is_non_empty {
                replacement_start + accepted_len
            } else {
                self.anchor + accepted_len
            };
            self.replacement_range = (self.anchor, self.anchor);
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct PredictionState {
    pub active_prediction: Option<EditPrediction>,
    pub pending_request: Option<RequestId>,
    pub status: PredictionStatus,
    pub document_version: u64,
    pub enabled: bool,
}

impl PredictionState {
    pub fn new() -> Self {
        Self {
            active_prediction: None,
            pending_request: None,
            status: PredictionStatus::Idle,
            document_version: 0,
            enabled: true,
        }
    }

    pub fn set_prediction(&mut self, prediction: EditPrediction) {
        self.status = PredictionStatus::Ready;
        self.active_prediction = Some(prediction);
    }

    pub fn apply_response(
        &mut self,
        request_id: RequestId,
        request_version: u64,
        prediction_text: String,
        anchor: usize,
    ) -> bool {
        // Ignore stale predictions from older versions or different requests
        if self.pending_request != Some(request_id) || request_version != self.document_version {
            return false;
        }

        self.pending_request = None;
        if prediction_text.is_empty() {
            self.status = PredictionStatus::Idle;
            self.active_prediction = None;
            return false;
        }

        self.status = PredictionStatus::Ready;
        self.active_prediction = Some(EditPrediction::with_range_and_version(
            anchor,
            (anchor, anchor),
            prediction_text,
            Some(request_id),
            self.document_version,
        ));
        true
    }

    pub fn notify_buffer_changed(&mut self, new_version: u64) {
        if self.document_version != new_version {
            self.document_version = new_version;
            self.clear();
        }
    }

    pub fn invalidate_if_cursor_moved(&mut self, current_cursor_offset: usize) {
        if let Some(pred) = &self.active_prediction {
            if pred.anchor != current_cursor_offset {
                self.clear();
            }
        }
    }

    pub fn clear(&mut self) {
        self.active_prediction = None;
        self.pending_request = None;
        self.status = PredictionStatus::Idle;
    }
}

#[cfg(test)]
#[path = "tests/prediction_tests.rs"]
mod tests;
