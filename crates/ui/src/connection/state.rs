use crate::UiConnectionDraft;

/// UI-owned state for the connection editor dialog.
///
/// This aggregate deliberately contains draft/edit/test state only. The saved
/// connection collection and lifecycle are siblings in `ConnectionFeatureState`;
/// they share the feature boundary without sharing mutable storage.
#[derive(Debug, Clone, Default)]
pub(crate) struct ConnectionDialogState {
    pub(super) open: bool,
    pub(super) focus_name_on_open: bool,
    pub(super) editing_connection_id: Option<String>,
    pub(super) draft: UiConnectionDraft,
    pub(super) show_password: bool,
    pub(super) error: String,
    pub(super) test_valid: bool,
    pub(super) test_draft: Option<UiConnectionDraft>,
    pub(super) diagnostics: Option<db_pro_core::domain::connection_diagnostics::ConnectionDiagnosticsReport>,
    pub(super) ssh_profiles: Vec<db_pro_core::domain::connection::SshProfile>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum ConnectionDialogAction {
    OpenNew,
    OpenEdit {
        connection_id: String,
        draft: UiConnectionDraft,
    },
    OpenDuplicate {
        draft: UiConnectionDraft,
    },
    DraftChanged,
    TestStarted {
        draft: UiConnectionDraft,
    },
    Close,
}

impl ConnectionDialogState {
    pub(crate) fn is_open(&self) -> bool {
        self.open
    }

    pub(crate) fn draft(&self) -> &UiConnectionDraft {
        &self.draft
    }

    pub(crate) fn draft_mut(&mut self) -> &mut UiConnectionDraft {
        &mut self.draft
    }

    pub(crate) fn set_draft(&mut self, draft: UiConnectionDraft) {
        self.draft = draft;
    }

    pub(crate) fn set_editing_connection_id(&mut self, connection_id: Option<String>) {
        self.editing_connection_id = connection_id;
    }

    pub(crate) fn set_error(&mut self, error: impl Into<String>) {
        self.error = error.into();
    }

    #[cfg(test)]
    pub(crate) fn error(&self) -> &str {
        &self.error
    }

    pub(crate) fn clear_error(&mut self) {
        self.error.clear();
    }

    pub(crate) fn set_test_valid(&mut self, valid: bool) {
        self.test_valid = valid;
    }

    #[cfg(test)]
    pub(crate) fn test_valid(&self) -> bool {
        self.test_valid
    }

    pub(crate) fn test_draft(&self) -> Option<&UiConnectionDraft> {
        self.test_draft.as_ref()
    }

    pub(crate) fn clear_test(&mut self) {
        self.invalidate_test();
    }

    pub(crate) fn diagnostics(
        &self,
    ) -> Option<&db_pro_core::domain::connection_diagnostics::ConnectionDiagnosticsReport> {
        self.diagnostics.as_ref()
    }

    pub(crate) fn ssh_profiles(&self) -> &[db_pro_core::domain::connection::SshProfile] {
        &self.ssh_profiles
    }

    pub(crate) fn set_ssh_profiles(&mut self, profiles: Vec<db_pro_core::domain::connection::SshProfile>) {
        self.ssh_profiles = profiles;
    }

    pub(crate) fn set_open(&mut self, open: bool) {
        self.open = open;
    }

    pub(crate) fn set_focus_name_on_open(&mut self, focus: bool) {
        self.focus_name_on_open = focus;
    }

    pub(crate) fn transition(&mut self, action: ConnectionDialogAction) {
        match action {
            ConnectionDialogAction::OpenNew => {
                self.editing_connection_id = None;
                self.draft = UiConnectionDraft::default();
                self.diagnostics = None;
                self.show_password = false;
                self.focus_name_on_open = true;
                self.open = true;
                self.invalidate_test();
            }
            ConnectionDialogAction::OpenEdit { connection_id, draft } => {
                self.editing_connection_id = Some(connection_id);
                self.draft = draft;
                self.focus_name_on_open = true;
                self.open = true;
                self.invalidate_test();
            }
            ConnectionDialogAction::OpenDuplicate { draft } => {
                self.editing_connection_id = None;
                self.draft = draft;
                self.focus_name_on_open = true;
                self.open = true;
                self.invalidate_test();
            }
            ConnectionDialogAction::DraftChanged => self.invalidate_test(),
            ConnectionDialogAction::TestStarted { draft } => {
                self.test_valid = false;
                self.test_draft = Some(draft);
                self.error.clear();
            }
            ConnectionDialogAction::Close => {
                self.open = false;
                self.invalidate_test();
            }
        }
    }

    fn invalidate_test(&mut self) {
        self.test_valid = false;
        self.test_draft = None;
        self.error.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_dialog_starts_with_a_fresh_draft_and_focus() {
        let mut state = ConnectionDialogState {
            error: "stale error".to_owned(),
            test_valid: true,
            ..Default::default()
        };

        state.transition(ConnectionDialogAction::OpenNew);

        assert!(state.open);
        assert!(state.focus_name_on_open);
        assert!(state.editing_connection_id.is_none());
        assert!(state.error.is_empty());
        assert!(!state.test_valid);
        assert!(state.test_draft.is_none());
    }

    #[test]
    fn closing_dialog_clears_test_state() {
        let mut state = ConnectionDialogState {
            open: true,
            test_valid: true,
            ..Default::default()
        };
        state.test_draft = Some(state.draft.clone());

        state.transition(ConnectionDialogAction::Close);

        assert!(!state.open);
        assert!(!state.test_valid);
        assert!(state.test_draft.is_none());
    }
}
