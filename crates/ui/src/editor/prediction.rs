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
            let next_pos = if pos == 0 {
                // Return at least the punctuation/space
                1
            } else {
                pos
            };
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
