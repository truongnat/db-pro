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
}
