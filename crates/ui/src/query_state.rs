//! Feature-owned lifecycle state for query documents and their shell requests.

use super::QueryDocument;
use crate::RequestId;
use std::collections::HashMap;

#[derive(Debug, Default)]
pub(crate) struct QuerySessionState {
    pub(super) documents: Vec<QueryDocument>,
    pub(super) active_document_index: usize,
    pub(super) selected_text: String,
    pub(super) document_requests: HashMap<RequestId, String>,
    pub(super) save_requests: HashMap<RequestId, String>,
    pub(super) pending_dirty_close: Option<usize>,
    pub(super) pending_close_after_save: Option<usize>,
    pub(super) save_as_name: String,
    pub(super) save_as_open: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_query_session_starts_without_documents_or_pending_requests() {
        let state = QuerySessionState::default();

        assert!(state.documents.is_empty());
        assert_eq!(state.active_document_index, 0);
        assert!(state.document_requests.is_empty());
        assert!(state.save_requests.is_empty());
        assert!(state.pending_dirty_close.is_none());
        assert!(state.pending_close_after_save.is_none());
        assert!(!state.save_as_open);
    }
}
