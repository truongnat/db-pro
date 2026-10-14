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
#[path = "tests/query_library_state_tests.rs"]
mod tests;

