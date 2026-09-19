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

impl QuerySessionState {
    pub(crate) fn active_document(&self) -> Option<&QueryDocument> {
        self.documents.get(self.active_document_index)
    }

    pub(crate) fn add_document(&mut self, document: QueryDocument) -> usize {
        self.documents.push(document);
        self.active_document_index = self.documents.len() - 1;
        self.active_document_index
    }

    pub(crate) fn select_document(&mut self, index: usize) -> bool {
        if index >= self.documents.len() {
            return false;
        }
        self.active_document_index = index;
        true
    }

    pub(crate) fn remove_document(&mut self, index: usize) -> Option<QueryDocument> {
        if index >= self.documents.len() {
            return None;
        }

        let document = self.documents.remove(index);
        if self.documents.is_empty() {
            self.active_document_index = 0;
        } else if self.active_document_index > index {
            self.active_document_index -= 1;
        } else if self.active_document_index == index {
            self.active_document_index = self.active_document_index.min(self.documents.len() - 1);
        }
        Some(document)
    }

    pub(crate) fn keep_document(&mut self, index: usize) -> bool {
        let Some(document) = self.documents.get(index).cloned() else {
            return false;
        };
        self.documents = vec![document];
        self.active_document_index = 0;
        true
    }

    pub(crate) fn close_documents_to_right(&mut self, index: usize) -> bool {
        if index >= self.documents.len() {
            return false;
        }
        self.documents.truncate(index + 1);
        if self.active_document_index > index {
            self.active_document_index = index;
        }
        true
    }

    pub(crate) fn replace_with_document(&mut self, document: QueryDocument) {
        self.documents = vec![document];
        self.active_document_index = 0;
    }
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

    #[test]
    fn document_collection_operations_keep_active_index_valid() {
        let mut state = QuerySessionState::default();
        let first = QueryDocument::new("query-1", "Query 1", "");
        let second = QueryDocument::new("query-2", "Query 2", "");
        let third = QueryDocument::new("query-3", "Query 3", "");

        assert_eq!(state.add_document(first), 0);
        assert_eq!(state.add_document(second), 1);
        assert_eq!(state.add_document(third), 2);
        assert!(state.select_document(1));

        assert_eq!(state.remove_document(0).expect("first document").id, "query-1");
        assert_eq!(state.active_document().map(|doc| doc.id.as_str()), Some("query-2"));
        assert!(state.close_documents_to_right(0));
        assert_eq!(state.documents.len(), 1);
        assert_eq!(state.active_document().map(|doc| doc.id.as_str()), Some("query-2"));
    }

    #[test]
    fn keep_document_replaces_collection_and_selects_it() {
        let mut state = QuerySessionState::default();
        state.add_document(QueryDocument::new("query-1", "Query 1", ""));
        state.add_document(QueryDocument::new("query-2", "Query 2", ""));

        assert!(state.keep_document(0));
        assert_eq!(state.documents.len(), 1);
        assert_eq!(state.active_document().map(|doc| doc.id.as_str()), Some("query-1"));
        assert!(!state.keep_document(4));
    }
}
