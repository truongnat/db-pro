//! State transitions and command preparation for saved query documents.

use super::{FeedbackState, QueryLibraryState, QuerySessionState, RequestId, UiCommand};

pub(super) fn list_queries_command(request_id: RequestId, connection_id: String) -> UiCommand {
    UiCommand::ListSavedQueries {
        request_id,
        connection_id,
    }
}

pub(super) fn list_folders_command(request_id: RequestId, connection_id: String) -> UiCommand {
    UiCommand::ListQueryFolders {
        request_id,
        connection_id,
    }
}

pub(super) fn create_folder_command(
    library: &QueryLibraryState,
    request_id: RequestId,
    connection_id: String,
) -> Result<UiCommand, String> {
    Ok(UiCommand::CreateQueryFolder {
        request_id,
        connection_id,
        name: library.required_folder_name()?,
    })
}

pub(super) fn save_query_command(
    library: &QueryLibraryState,
    request_id: RequestId,
    connection_id: String,
    saved_query_id: Option<String>,
    name: String,
    sql: String,
) -> UiCommand {
    UiCommand::SaveQuery {
        request_id,
        connection_id,
        saved_query_id,
        name,
        sql,
        folder: library.normalized_folder(),
    }
}

pub(super) fn rename_query_command(request_id: RequestId, id: String, name: String) -> UiCommand {
    UiCommand::RenameSavedQuery { request_id, id, name }
}

pub(super) fn delete_query_command(request_id: RequestId, id: String) -> UiCommand {
    UiCommand::DeleteSavedQuery { request_id, id }
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
    ) -> Option<UiCommand> {
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
        let command = save_query_command(self.library, request_id, connection_id, saved_query_id, name, sql);
        Some(command)
    }

    pub(crate) fn commit_dispatched(&mut self, request_id: RequestId, document_index: usize) {
        if let Some(document_id) = self
            .session
            .documents
            .get(document_index)
            .map(|document| document.id.clone())
        {
            self.session.save_requests.insert(request_id, document_id);
        }
        self.feedback.set_runtime_message("Saving query…");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::query::QueryDocument;

    #[test]
    fn save_preparation_commits_document_request_after_dispatch() {
        let mut session = QuerySessionState::default();
        session.add_document(QueryDocument::new("doc-1", "Report", "SELECT 1"));
        let library = QueryLibraryState::default();
        let mut feedback = FeedbackState::default();
        let command = {
            let mut context = QuerySaveContext::new(&mut session, &library, &mut feedback);
            context
                .prepare_save(RequestId(7), 0, Some("conn-1".to_owned()))
                .expect("connection should prepare save")
        };

        assert!(matches!(
            command,
            UiCommand::SaveQuery {
                request_id: RequestId(7),
                connection_id,
                name,
                sql,
                folder: None,
                ..
            } if connection_id == "conn-1" && name == "Report" && sql == "SELECT 1"
        ));
        assert!(session.save_requests.is_empty());
        assert!(!feedback.runtime_message.contains("Saving"));
        let mut context = QuerySaveContext::new(&mut session, &library, &mut feedback);
        context.commit_dispatched(RequestId(7), 0);
        assert_eq!(
            session.save_requests.get(&RequestId(7)).map(String::as_str),
            Some("doc-1")
        );
        assert!(feedback.runtime_message.contains("Saving"));
    }

    #[test]
    fn save_preparation_requires_a_connection_without_registering_request() {
        let mut session = QuerySessionState::default();
        session.add_document(QueryDocument::new("doc-1", "Report", "SELECT 1"));
        let library = QueryLibraryState::default();
        let mut feedback = FeedbackState::default();
        let mut context = QuerySaveContext::new(&mut session, &library, &mut feedback);

        assert!(context.prepare_save(RequestId(7), 0, None).is_none());
        assert!(session.save_requests.is_empty());
        assert!(feedback.runtime_message.contains("connection"));
    }
}
