use crate::{UiQueryFolderSummary, UiSavedQuerySummary};

/// UI-owned state for the saved-query library and its editor drafts.
#[derive(Debug, Default)]
pub(crate) struct QueryLibraryState {
    pub(super) saved_queries: Vec<UiSavedQuerySummary>,
    pub(super) query_folders: Vec<UiQueryFolderSummary>,
    pub(super) query_folder: String,
}

impl QueryLibraryState {
    pub(super) fn normalized_folder(&self) -> Option<String> {
        let name = self.query_folder.trim();
        (!name.is_empty()).then(|| name.to_owned())
    }

    pub(super) fn required_folder_name(&self) -> Result<String, String> {
        self.normalized_folder()
            .ok_or_else(|| "Folder name is required".to_owned())
    }
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

    #[test]
    fn folder_draft_normalization_stays_in_library_state() {
        let state = QueryLibraryState::default();
        assert_eq!(state.normalized_folder(), None);
        assert_eq!(state.required_folder_name(), Err("Folder name is required".to_owned()));
    }

    #[test]
    fn folder_draft_trims_whitespace() {
        let state = QueryLibraryState {
            query_folder: " reports ".to_owned(),
            ..QueryLibraryState::default()
        };
        assert_eq!(state.normalized_folder().as_deref(), Some("reports"));
        assert_eq!(state.required_folder_name().unwrap(), "reports");
    }
}
