//! Tests for `../workspace_feature_state.rs` (moved out of the source file).
    use super::super::WorkspaceTab;
    use super::WorkspaceFeatureState;

    #[test]
    fn default_workspace_feature_owns_all_workspace_substates() {
        let state = WorkspaceFeatureState::default();

        assert_eq!(state.shell.active_tab, WorkspaceTab::Welcome);
        assert!(state.files.ide_workspace.roots.is_empty());
        assert!(state.sessions.selected_id.is_none());
    }