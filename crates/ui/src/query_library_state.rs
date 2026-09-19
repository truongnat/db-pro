use crate::{UiQueryFolderSummary, UiSavedQuerySummary};

/// UI-owned state for the saved-query library and its editor drafts.
#[derive(Debug, Default)]
pub(crate) struct QueryLibraryState {
    pub saved_queries: Vec<UiSavedQuerySummary>,
    pub query_folders: Vec<UiQueryFolderSummary>,
    pub query_folder: String,
}

#[cfg(test)]
mod tests {
    use super::QueryLibraryState;

    #[test]
    fn default_library_has_no_remote_items_or_draft_folder() {
        let state = QueryLibraryState::default();

        assert!(state.saved_queries.is_empty());
        assert!(state.query_folders.is_empty());
        assert!(state.query_folder.is_empty());
    }
}
