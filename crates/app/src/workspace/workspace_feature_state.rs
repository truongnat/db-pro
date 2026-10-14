//! Feature-owned state for the native workspace surface.
//!
//! The workspace is one user-facing bounded context even though its shell,
//! local-file activity and named-session persistence have different internal
//! lifecycles. Keeping those sub-states behind one aggregate gives the
//! composition root one ownership boundary without flattening their APIs.

use super::{WorkspaceFilesState, WorkspaceSessionState, WorkspaceShellState};

#[derive(Debug, Default)]
pub(crate) struct WorkspaceFeatureState {
    pub(super) shell: WorkspaceShellState,
    pub(super) files: WorkspaceFilesState,
    pub(super) sessions: WorkspaceSessionState,
}

// Keep the shell's established field access readable inside the UI crate while
// making the aggregate the sole owner at the composition-root boundary.
impl std::ops::Deref for WorkspaceFeatureState {
    type Target = WorkspaceShellState;

    fn deref(&self) -> &Self::Target {
        &self.shell
    }
}

impl std::ops::DerefMut for WorkspaceFeatureState {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.shell
    }
}

#[cfg(test)]
mod tests {
    use super::super::WorkspaceTab;
    use super::WorkspaceFeatureState;

    #[test]
    fn default_workspace_feature_owns_all_workspace_substates() {
        let state = WorkspaceFeatureState::default();

        assert_eq!(state.shell.active_tab, WorkspaceTab::Welcome);
        assert!(state.files.ide_workspace.roots.is_empty());
        assert!(state.sessions.selected_id.is_none());
    }
}
