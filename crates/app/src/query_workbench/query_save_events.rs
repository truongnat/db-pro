//! Saved-query lifecycle reducers over explicit query and library state.

use super::*;
use crate::RequestId;

pub(super) fn on_query_saved(
    query_session: &mut QuerySessionState,
    query_library: &mut QueryLibraryState,
    feedback: &mut FeedbackState,
    request_id: RequestId,
    query: UiSavedQuerySummary,
) -> Option<usize> {
    let save_target = query_session.save_requests.remove(&request_id);
    let mut close_index = None;
    if let Some((document_id, saved_version)) = save_target {
        if let Some((index, doc)) = query_session
            .documents
            .iter_mut()
            .enumerate()
            .find(|(_, doc)| doc.id == document_id)
        {
            doc.saved_query_id = Some(query.id.clone());
            // Only mark_saved if buffer hasn't been edited since dispatch (#446).
            if doc.buffer.version() == saved_version {
                doc.mark_saved();
                if query_session.pending_close_after_save == Some(index) {
                    close_index = Some(index);
                }
            } else {
                feedback.set_runtime_message("Saved an earlier version — current edits remain unsaved");
            }
        }
    }
    query_library.saved_queries.retain(|saved| saved.id != query.id);
    query_library.saved_queries.push(query);
    feedback.runtime_message = "Query saved".to_owned();
    if close_index.is_some() {
        query_session.pending_close_after_save = None;
    }
    close_index
}
