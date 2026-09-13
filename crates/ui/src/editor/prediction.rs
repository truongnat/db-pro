use crate::runtime::RequestId;
use serde::{Deserialize, Serialize};
use std::hash::{Hash, Hasher};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum PredictionMode {
    #[default]
    Eager,
    Subtle,
    Off,
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
    let max_len = left.len().min(right.len());
    (1..=max_len)
        .rev()
        .find(|len| left.as_bytes()[left.len() - len..] == right.as_bytes()[..*len])
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
        if let Some(pos) = self
            .text
            .find(|c: char| c.is_whitespace() || c == '(' || c == ',' || c == ';')
        {
            let next_pos = if pos == 0 {
                // Skip leading punctuation/whitespace if needed
                self.text[1..]
                    .find(|c: char| c.is_whitespace() || c == '(' || c == ',' || c == ';')
                    .map(|p| p + 1)
                    .unwrap_or(self.text.len())
            } else {
                pos
            };
            &self.text[..next_pos]
        } else {
            &self.text
        }
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
mod tests {
    use super::*;

    #[test]
    fn test_edit_prediction_accept_helpers() {
        let pred = EditPrediction::new(10, "WHERE id = 100\nLIMIT 10;", None);
        assert_eq!(pred.accept_full(), "WHERE id = 100\nLIMIT 10;");
        assert_eq!(pred.accept_next_word(), "WHERE");
        assert_eq!(pred.accept_next_line(), "WHERE id = 100\n");
    }

    #[test]
    fn partial_accept_moves_anchor_after_replacement_range() {
        let mut pred = EditPrediction::with_range_and_version(8, (4, 8), "SELECT", None, 1);
        pred.consume(3);
        assert_eq!(pred.anchor, 7);
        assert_eq!(pred.replacement_range, (7, 7));
        assert_eq!(pred.text, "ECT");
    }

    #[test]
    fn test_prediction_state_stale_rejection() {
        let mut state = PredictionState::new();
        state.document_version = 2;
        let req_id = RequestId(101);
        state.pending_request = Some(req_id);
        state.status = PredictionStatus::Generating(req_id);

        // Response from older version (v1) should be rejected
        let accepted = state.apply_response(req_id, 1, "SELECT 1;".to_owned(), 5);
        assert!(!accepted);
        assert_eq!(state.active_prediction, None);

        // Response from matching version (v2) should be accepted
        let accepted = state.apply_response(req_id, 2, "SELECT 1;".to_owned(), 5);
        assert!(accepted);
        assert_eq!(state.status, PredictionStatus::Ready);
        assert_eq!(state.active_prediction.as_ref().unwrap().text, "SELECT 1;");

        // Buffer changed -> invalidation
        state.notify_buffer_changed(3);
        assert_eq!(state.active_prediction, None);
        assert_eq!(state.status, PredictionStatus::Idle);
    }

    #[test]
    fn context_fingerprint_is_stable_for_alias_map_order() {
        let mut first = AiSqlContext::default();
        first.table_aliases.insert("u".to_owned(), "users".to_owned());
        first.table_aliases.insert("o".to_owned(), "orders".to_owned());
        let mut second = first.clone();
        second.table_aliases.clear();
        second.table_aliases.insert("o".to_owned(), "orders".to_owned());
        second.table_aliases.insert("u".to_owned(), "users".to_owned());
        assert_eq!(first.fingerprint(3, 12), second.fingerprint(3, 12));
    }

    #[test]
    fn overlap_normalization_removes_duplicate_prefix_and_suffix() {
        assert_eq!(
            normalize_prediction_overlap(
                "SELECT * FROM users WHERE ",
                " LIMIT 10",
                "WHERE active = true LIMIT 10"
            ),
            "active = true"
        );
        assert_eq!(
            normalize_prediction_overlap("ORDER BY created_at", " DESC", " DESC"),
            ""
        );
        assert_eq!(normalize_prediction_overlap("SELECT ", "", "users"), "users");
    }
}
