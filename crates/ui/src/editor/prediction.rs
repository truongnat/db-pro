use crate::runtime::RequestId;
use serde::{Deserialize, Serialize};

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

pub trait AiSqlPredictionProvider: Send + Sync {
    fn request_prediction(&self, context: &AiSqlContext) -> Option<String>;
    fn cancel_prediction(&self, request_id: RequestId);
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AiSqlContext {
    pub sql_before_cursor: String,
    pub sql_after_cursor: String,
    pub current_statement: String,
    pub active_schema: String,
    pub referenced_tables: Vec<String>,
    pub relevant_columns: Vec<String>,
    pub fk_neighbors: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EditPrediction {
    pub anchor: usize,
    pub replacement_range: (usize, usize),
    pub text: String,
    pub request_id: Option<RequestId>,
    pub document_version: usize,
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
        document_version: usize,
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
}

#[derive(Debug, Clone, Default)]
pub struct PredictionState {
    pub active_prediction: Option<EditPrediction>,
    pub pending_request: Option<RequestId>,
    pub status: PredictionStatus,
    pub document_version: usize,
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
        request_version: usize,
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

    pub fn notify_buffer_changed(&mut self, new_version: usize) {
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

pub trait AiEditPredictionProvider {
    fn request_prediction(&mut self, buffer_text: &str, cursor_offset: usize, dialect_name: &str) -> Option<RequestId>;
    fn cancel_pending(&mut self);
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
}
