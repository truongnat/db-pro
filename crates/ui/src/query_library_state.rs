use crate::{RequestId, UiCommand, UiQueryFolderSummary, UiSavedQuerySummary};

/// UI-owned state for the saved-query library and its editor drafts.
#[derive(Debug, Default)]
pub(crate) struct QueryLibraryState {
    pub(super) saved_queries: Vec<UiSavedQuerySummary>,
    pub(super) query_folders: Vec<UiQueryFolderSummary>,
    pub(super) query_folder: String,
}

impl QueryLibraryState {
    pub(super) fn list_queries_command(&self, request_id: RequestId, connection_id: String) -> UiCommand {
        UiCommand::ListSavedQueries {
            request_id,
            connection_id,
        }
    }

    pub(super) fn list_folders_command(&self, request_id: RequestId, connection_id: String) -> UiCommand {
        UiCommand::ListQueryFolders {
            request_id,
            connection_id,
        }
    }

    pub(super) fn create_folder_command(
        &self,
        request_id: RequestId,
        connection_id: String,
    ) -> Result<UiCommand, String> {
        let name = self.query_folder.trim();
        if name.is_empty() {
            return Err("Folder name is required".to_owned());
        }
        Ok(UiCommand::CreateQueryFolder {
            request_id,
            connection_id,
            name: name.to_owned(),
        })
    }

    pub(super) fn save_query_command(
        &self,
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
            folder: (!self.query_folder.trim().is_empty()).then(|| self.query_folder.trim().to_owned()),
        }
    }

    pub(super) fn rename_query_command(&self, request_id: RequestId, id: String, name: String) -> UiCommand {
        UiCommand::RenameSavedQuery { request_id, id, name }
    }

    pub(super) fn delete_query_command(&self, request_id: RequestId, id: String) -> UiCommand {
        UiCommand::DeleteSavedQuery { request_id, id }
    }
}

#[cfg(test)]
mod tests {
    use super::{QueryLibraryState, RequestId, UiCommand};

    #[test]
    fn default_library_has_no_remote_items_or_draft_folder() {
        let state = QueryLibraryState::default();

        assert!(state.saved_queries.is_empty());
        assert!(state.query_folders.is_empty());
        assert!(state.query_folder.is_empty());
    }

    #[test]
    fn refresh_commands_keep_query_library_effects_explicit() {
        let state = QueryLibraryState::default();

        assert!(matches!(
            state.list_queries_command(RequestId(1), "source".to_owned()),
            UiCommand::ListSavedQueries {
                request_id: RequestId(1),
                connection_id,
            } if connection_id == "source"
        ));
        assert!(matches!(
            state.list_folders_command(RequestId(2), "source".to_owned()),
            UiCommand::ListQueryFolders {
                request_id: RequestId(2),
                connection_id,
            } if connection_id == "source"
        ));
    }

    #[test]
    fn save_command_normalizes_optional_folder_and_create_requires_name() {
        let state = QueryLibraryState {
            query_folder: " reports ".to_owned(),
            ..QueryLibraryState::default()
        };
        assert!(matches!(
            state.save_query_command(
                RequestId(3),
                "source".to_owned(),
                Some("saved-1".to_owned()),
                "Query".to_owned(),
                "select 1".to_owned(),
            ),
            UiCommand::SaveQuery { folder: Some(folder), .. } if folder == "reports"
        ));

        assert_eq!(
            QueryLibraryState::default().create_folder_command(RequestId(4), "source".to_owned()),
            Err("Folder name is required".to_owned())
        );
    }
}
