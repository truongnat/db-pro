//! State transitions and command preparation for saved query documents.

use super::{FeedbackState, QueryLibraryState, QuerySessionState, RequestId};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct PreparedQuerySave {
    pub(crate) request_id: RequestId,
    pub(crate) connection_id: String,
    pub(crate) saved_query_id: Option<String>,
    pub(crate) name: String,
    pub(crate) sql: String,
    pub(crate) folder: Option<String>,
}

pub(crate) struct QuerySaveContext<'a> {
    session: &'a mut QuerySessionState,
    library: &'a QueryLibraryState,
    feedback: &'a mut FeedbackState,
}

impl<'a> QuerySaveContext<'a> {
    pub(crate) fn new(
        session: &'a mut QuerySessionState,
        library: &'a QueryLibraryState,
        feedback: &'a mut FeedbackState,
    ) -> Self {
        Self {
            session,
            library,
            feedback,
        }
    }

    pub(crate) fn prepare_save(
        &mut self,
        request_id: RequestId,
        document_index: usize,
        connection_id: Option<String>,
    ) -> Option<PreparedQuerySave> {
        let Some(connection_id) = connection_id else {
            self.feedback.set_runtime_message("Create or select a connection first");
            return None;
        };
        let name = self
            .session
            .documents
            .get(document_index)
            .map(|document| document.title.clone())
            .unwrap_or_else(|| "Saved query".to_owned());
        let saved_query_id = self
            .session
            .documents
            .get(document_index)
            .and_then(|document| document.saved_query_id.clone());
        let sql = self
            .session
            .documents
            .get(document_index)
            .map_or_else(String::new, |document| document.text().to_owned());
        Some(PreparedQuerySave {
            request_id,
            connection_id,
            saved_query_id,
            name,
            sql,
            folder: self.library.normalized_folder(),
        })
    }

    pub(crate) fn commit_dispatched(&mut self, request_id: RequestId, document_index: usize) {
        if let Some(doc) = self.session.documents.get(document_index) {
            let doc_id = doc.id.clone();
            let version = doc.buffer.version();
            self.session.save_requests.insert(request_id, (doc_id, version));
        }
        self.feedback.set_runtime_message("Saving query…");
    }
}

#[cfg(test)]
#[path = "tests/query_save_actions_tests.rs"]
mod tests;

