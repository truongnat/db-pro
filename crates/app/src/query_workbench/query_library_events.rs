//! Saved-query library reducers over explicit feature state.

use super::*;
use crate::UiQueryFolderSummary;

pub(super) fn on_saved_queries_loaded(query_library: &mut QueryLibraryState, queries: Vec<UiSavedQuerySummary>) {
    query_library.saved_queries = queries;
}

pub(super) fn on_query_folders_loaded(query_library: &mut QueryLibraryState, folders: Vec<UiQueryFolderSummary>) {
    query_library.query_folders = folders;
}

#[cfg(test)]
#[path = "tests/query_library_events_tests.rs"]
mod tests;

