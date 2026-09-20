//! Feature-owned lifecycle state for query documents and their shell requests.

use super::{QueryDocument, QueryExecutionState};
use crate::runtime::UiQueryResult;
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

    pub(crate) fn active_document_mut(&mut self) -> Option<&mut QueryDocument> {
        self.documents.get_mut(self.active_document_index)
    }

    pub(crate) fn active_text(&self) -> &str {
        self.active_document().map(QueryDocument::text).unwrap_or("")
    }

    pub(crate) fn active_buffer_version(&self) -> u64 {
        self.active_document()
            .map(|document| document.buffer.version())
            .unwrap_or(0)
    }

    pub(crate) fn set_active_text(&mut self, text: impl Into<String>) -> bool {
        let Some(document) = self.active_document_mut() else {
            return false;
        };
        document.set_text(text);
        true
    }

    pub(crate) fn append_active_text(&mut self, text: &str) -> bool {
        let Some(document) = self.active_document_mut() else {
            return false;
        };
        let mut current = document.text().to_owned();
        if !current.trim().is_empty() {
            current.push_str("\n\n");
        }
        current.push_str(text);
        document.set_text(current);
        true
    }

    pub(crate) fn active_explain_plan(&self) -> Option<&str> {
        self.active_document()
            .and_then(|document| document.explain_plan.as_deref())
    }

    pub(crate) fn active_explain_request(&self) -> Option<RequestId> {
        self.active_document().and_then(|document| document.explain_request)
    }

    pub(crate) fn active_running_request(&self) -> Option<RequestId> {
        self.active_document()
            .and_then(|document| match document.execution_state {
                QueryExecutionState::Running(request_id) => Some(request_id),
                _ => None,
            })
    }

    pub(crate) fn active_result(&self) -> Option<&UiQueryResult> {
        self.active_document().and_then(|document| {
            document
                .query_results
                .get(document.active_result_index)
                .or(document.query_result.as_ref())
        })
    }

    pub(crate) fn active_result_count(&self) -> usize {
        self.active_document().map_or(0, |document| {
            document
                .query_results
                .len()
                .max(usize::from(document.query_result.is_some()))
        })
    }

    pub(crate) fn set_active_result(&mut self, index: usize) -> bool {
        let Some(document) = self.active_document_mut() else {
            return false;
        };
        if index >= document.query_results.len() || document.active_result_index == index {
            return false;
        }
        document.active_result_index = index;
        true
    }

    pub(crate) fn active_messages(&self) -> &[String] {
        self.active_document()
            .map(|document| document.query_messages.as_slice())
            .unwrap_or(&[])
    }

    pub(crate) fn active_connection_id(&self) -> Option<&str> {
        self.active_document()
            .and_then(|document| document.connection_id.as_deref())
    }

    pub(crate) fn active_schema(&self) -> Option<&str> {
        self.active_document().and_then(|document| document.schema.as_deref())
    }

    pub(crate) fn set_document_connection(&mut self, index: usize, connection_id: Option<String>) -> bool {
        let Some(document) = self.documents.get_mut(index) else {
            return false;
        };
        document.connection_id = connection_id;
        document.completion.clear();
        true
    }

    pub(crate) fn set_document_schema(&mut self, index: usize, schema: Option<String>) -> bool {
        let Some(document) = self.documents.get_mut(index) else {
            return false;
        };
        document.schema = schema;
        document.completion.clear();
        true
    }

    pub(crate) fn invalidate_prediction(&mut self, index: usize) -> Option<RequestId> {
        let document = self.documents.get_mut(index)?;
        let request_id = document.pending_prediction_request;
        if request_id.is_some() {
            document.prediction_requests_cancelled = document.prediction_requests_cancelled.saturating_add(1);
        }
        document.invalidate_prediction();
        request_id
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

    #[test]
    fn active_document_projections_are_owned_by_query_session_state() {
        let mut state = QuerySessionState::default();
        state.add_document(QueryDocument::new("query-1", "Query 1", "SELECT 1"));

        assert_eq!(state.active_text(), "SELECT 1");
        assert!(state.append_active_text("SELECT 2"));
        assert_eq!(state.active_text(), "SELECT 1\n\nSELECT 2");
        assert_eq!(state.active_result_count(), 0);
        assert!(state.active_result().is_none());
        assert!(state.active_messages().is_empty());
        assert!(!state.set_active_result(0));
    }

    #[test]
    fn document_metadata_and_prediction_invalidation_are_state_owned() {
        let mut state = QuerySessionState::default();
        state.add_document(QueryDocument::new("query-1", "Query 1", ""));

        assert!(state.set_document_connection(0, Some("conn-1".to_owned())));
        assert_eq!(state.active_connection_id(), Some("conn-1"));
        assert!(state.set_document_schema(0, Some("analytics".to_owned())));
        assert_eq!(state.active_schema(), Some("analytics"));
        assert!(!state.set_document_schema(9, None));
        assert!(state.invalidate_prediction(0).is_none());
    }
}
