use crate::runtime::RequestId;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EditPrediction {
    pub anchor: usize,
    pub text: String,
    pub request_id: Option<RequestId>,
}

impl EditPrediction {
    pub fn new(anchor: usize, text: impl Into<String>, request_id: Option<RequestId>) -> Self {
        Self {
            anchor,
            text: text.into(),
            request_id,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.text.is_empty()
    }

    pub fn accept_full(&self) -> &str {
        &self.text
    }

    pub fn accept_word(&self) -> &str {
        if let Some(pos) = self
            .text
            .find(|c: char| c.is_whitespace() || c == '(' || c == ',' || c == ';')
        {
            let next_pos = if pos == 0 { 1 } else { pos };
            &self.text[..next_pos]
        } else {
            &self.text
        }
    }

    pub fn accept_line(&self) -> &str {
        if let Some(pos) = self.text.find('\n') {
            &self.text[..=pos]
        } else {
            &self.text
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct PredictionState {
    pub active_prediction: Option<EditPrediction>,
    pub pending_request: Option<RequestId>,
    pub enabled: bool,
}

impl PredictionState {
    pub fn new() -> Self {
        Self {
            active_prediction: None,
            pending_request: None,
            enabled: true,
        }
    }

    pub fn set_prediction(&mut self, prediction: EditPrediction) {
        self.active_prediction = Some(prediction);
    }

    pub fn clear(&mut self) {
        self.active_prediction = None;
        self.pending_request = None;
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
        assert_eq!(pred.accept_word(), "WHERE");
        assert_eq!(pred.accept_line(), "WHERE id = 100\n");
    }
}
