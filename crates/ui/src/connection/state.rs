use crate::UiConnectionDraft;

/// UI-owned state for the connection editor dialog.
///
/// This aggregate deliberately contains draft/edit/test state only. The saved
/// connection collection and connection lifecycle belong to the connection
/// session state in `DbProApp` until that boundary is migrated separately.
#[derive(Debug, Clone, Default)]
pub(crate) struct ConnectionDialogState {
    pub(in crate::app) open: bool,
    pub(in crate::app) focus_name_on_open: bool,
    pub(in crate::app) editing_connection_id: Option<String>,
    pub(in crate::app) draft: UiConnectionDraft,
    pub(in crate::app) show_password: bool,
    pub(in crate::app) error: String,
    pub(in crate::app) test_valid: bool,
    pub(in crate::app) test_draft: Option<UiConnectionDraft>,
    pub(in crate::app) diagnostics: Option<db_pro_core::domain::connection_diagnostics::ConnectionDiagnosticsReport>,
    pub(in crate::app) ssh_profiles: Vec<db_pro_core::domain::connection::SshProfile>,
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
