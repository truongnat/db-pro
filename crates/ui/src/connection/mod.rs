pub mod advanced_panels;
pub mod catalog;
pub mod config;
pub mod confirm_dialogs;
pub mod form_fields;
pub mod layout;
pub mod lifecycle;
pub mod logic;
pub mod mapper;
pub mod state;
pub mod view;

#[cfg(test)]
mod tests;

use crate::{DbProApp, UiConnectionSummary, UiDriver};

pub(crate) use catalog::ConnectionCatalogState;
pub(crate) use lifecycle::ConnectionLifecycleState;
pub(crate) use state::ConnectionDialogState;

impl DbProApp {
    /// Apply a driver choice from the connection dialog.
    pub fn select_connection_driver(&mut self, driver: UiDriver) {
        logic::select_driver(&mut self.connection_dialog.draft, driver);
    }

    /// Open connection edit dialog from saved summary.
    pub fn open_edit_connection(&mut self, connection: &UiConnectionSummary) {
        self.connection_lifecycle.clear_pending_request();
        self.connection_dialog
            .transition(state::ConnectionDialogAction::OpenEdit {
                connection_id: connection.id.clone(),
                draft: mapper::summary_to_edit_draft(connection),
            });
    }

    /// Open connection draft duplicating a saved summary.
    pub fn open_duplicate_connection(&mut self, connection: &UiConnectionSummary) {
        self.connection_lifecycle.clear_pending_request();
        self.connection_dialog
            .transition(state::ConnectionDialogAction::OpenDuplicate {
                draft: mapper::summary_to_duplicate_draft(connection),
            });
    }

    /// Save the SSH parameters in the active draft as a new reusable SSH Profile.
    pub fn save_draft_as_ssh_profile(&mut self) {
        match logic::save_ssh_profile(&mut self.connection_dialog.ssh_profiles, &self.connection_dialog.draft) {
            Ok(id) => {
                self.connection_dialog.draft.ssh_profile_id = id;
                self.feedback.runtime_message = "SSH profile saved — reusable by other connections".to_owned();
            }
            Err(err) => {
                self.feedback.runtime_message = err;
            }
        }
    }

    /// Apply an existing SSH Profile onto the active draft.
    pub fn apply_ssh_profile(&mut self, profile_id: &str) {
        let Some(profile) = self
            .connection_dialog
            .ssh_profiles
            .iter()
            .find(|p| p.id == profile_id)
            .cloned()
        else {
            return;
        };
        logic::apply_ssh_profile_to_draft(&mut self.connection_dialog.draft, &profile);
        self.connection_dialog.test_valid = false;
    }

    /// Apply a selected cloud preset to the active draft.
    pub fn apply_cloud_preset(&mut self) {
        let key = self.connection_dialog.draft.cloud_preset.clone();
        match mapper::apply_cloud_preset_to_draft(&mut self.connection_dialog.draft, &key) {
            Ok(guidance) => {
                self.connection_dialog.draft.cloud_guidance = guidance;
                self.connection_dialog.error.clear();
                self.connection_dialog.test_valid = false;
            }
            Err(err) => {
                self.connection_dialog.error = err;
            }
        }
    }

    /// Dispatch connection test or save command to runtime worker.
    pub fn dispatch_connection_command(&mut self, save: bool) {
        if let Err(err) = logic::validate_connection_draft(&self.connection_dialog.draft) {
            self.show_toast_error(err.clone());
            self.connection_dialog.error = err;
            return;
        }

        let request_id = self.task_bridge.next_request_id();
        let draft = self.connection_dialog.draft.clone();
        let command = logic::build_connection_command(
            draft,
            self.connection_dialog.editing_connection_id.clone(),
            request_id,
            save,
        );

        self.dispatch_command(command);
        self.connection_lifecycle.set_pending_request(Some(request_id));
        if save {
            self.connection_dialog.test_valid = false;
        } else {
            self.connection_dialog
                .transition(state::ConnectionDialogAction::TestStarted {
                    draft: self.connection_dialog.draft.clone(),
                });
        }
        self.connection_dialog.error.clear();
        self.feedback.runtime_message = if save {
            t!("status.saving").to_string()
        } else {
            t!("status.testing").to_string()
        };

        if !save {
            self.refresh_connection_diagnostics(false, &t!("status.auth_pending"));
        }
    }

    /// Probe connection stages and refresh diagnostics report.
    pub fn refresh_connection_diagnostics(&mut self, auth_ok: bool, auth_message: &str) {
        let report = logic::probe_draft_diagnostics(&self.connection_dialog.draft, auth_ok, auth_message);
        self.connection_dialog.diagnostics = Some(report);
    }
}
