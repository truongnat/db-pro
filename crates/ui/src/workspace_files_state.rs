use super::*;

/// Presentation state for the local SQL workspace and its file/Git activity.
///
/// Database connection state and workspace navigation stay outside this aggregate:
/// this state owns only local files, search/refactor drafts, Git feedback and the
/// file context that can be attached to an agent request.
#[derive(Debug, Default)]
pub(crate) struct WorkspaceFilesState {
    pub(crate) ide_workspace: ide_workspace::IdeWorkspaceState,
    pub(crate) git_status: Option<git_workspace::GitWorkspaceStatus>,
    pub(crate) git_diff: Option<git_workspace::GitDiffResult>,
    pub(crate) git_commit_message: String,
    pub(crate) git_last_error: Option<String>,
    pub(crate) workspace_file_mtimes: std::collections::HashMap<String, u64>,
    pub(crate) workspace_external_change: Option<String>,
    pub(crate) workspace_search_query: String,
    pub(crate) workspace_replace_query: String,
    pub(crate) workspace_search_hits: Vec<ide_workspace::SearchHit>,
    pub(crate) workspace_replace_previews: Vec<ide_workspace::ReplacePreview>,
    pub(crate) workspace_task_command: String,
    pub(crate) workspace_refactor_from: String,
    pub(crate) workspace_refactor_to: String,
    pub(crate) workspace_context_items: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::WorkspaceFilesState;

    #[test]
    fn default_workspace_files_state_starts_without_open_roots_or_drafts() {
        let state = WorkspaceFilesState::default();

        assert!(state.ide_workspace.roots.is_empty());
        assert!(state.workspace_search_query.is_empty());
        assert!(state.workspace_replace_previews.is_empty());
        assert!(state.workspace_context_items.is_empty());
        assert!(state.git_status.is_none());
    }
}
