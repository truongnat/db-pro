//! Tests for `../workspace_files_state.rs` (moved out of the source file).
    use super::{FilesPanelTab, WorkspaceFilesState, WorkspaceSearchDraft};

    #[test]
    fn default_workspace_files_state_starts_without_open_roots_or_drafts() {
        let state = WorkspaceFilesState::default();

        assert!(state.ide_workspace.roots.is_empty());
        assert!(state.workspace_search_query.is_empty());
        assert!(state.workspace_replace_previews.is_empty());
        assert!(state.workspace_context_items.is_empty());
        assert!(state.git_status.is_none());
        assert_eq!(state.panel_tab, FilesPanelTab::Tree);
    }

    #[test]
    fn panel_selection_and_invalid_root_selection_stay_inside_workspace_files_state() {
        let mut state = WorkspaceFilesState::default();

        state.select_panel_tab(FilesPanelTab::Search);

        assert_eq!(state.panel_tab(), FilesPanelTab::Search);
        assert!(!state.select_root(0));
    }

    #[test]
    fn search_draft_round_trip_preserves_all_workspace_search_inputs() {
        let mut state = WorkspaceFilesState::default();
        let draft = WorkspaceSearchDraft {
            search_query: "users".to_owned(),
            replace_query: "accounts".to_owned(),
            refactor_from: "user_id".to_owned(),
            refactor_to: "account_id".to_owned(),
        };

        state.apply_search_draft(draft.clone());

        assert_eq!(state.search_draft(), draft);
    }