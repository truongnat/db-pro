pub mod advanced_panels;
pub mod config;
pub mod confirm_dialogs;
pub mod form_fields;
pub mod layout;
pub mod logic;
pub mod mapper;
pub mod view;

#[cfg(test)]
mod tests;

use crate::{DbProApp, UiConnectionSummary, UiDriver};

impl DbProApp {
    /// Apply a driver choice from the connection dialog.
    pub fn select_connection_driver(&mut self, driver: UiDriver) {
        logic::select_driver(&mut self.connection_draft, driver);
    }

    /// Open connection edit dialog from saved summary.
    pub fn open_edit_connection(&mut self, connection: &UiConnectionSummary) {
        self.pending_connection_request = None;
        self.editing_connection_id = Some(connection.id.clone());
        self.connection_draft = mapper::summary_to_edit_draft(connection);
        self.connection_error.clear();
        self.connection_test_valid = false;
        self.connection_test_draft = None;
        self.connection_focus_name_on_open = true;
        self.connection_dialog_open = true;
    }

    /// Open connection draft duplicating a saved summary.
    pub fn open_duplicate_connection(&mut self, connection: &UiConnectionSummary) {
        self.pending_connection_request = None;
        self.editing_connection_id = None;
        self.connection_draft = mapper::summary_to_duplicate_draft(connection);
        self.connection_error.clear();
        self.connection_test_valid = false;
        self.connection_test_draft = None;
        self.connection_focus_name_on_open = true;
        self.connection_dialog_open = true;
    }

    /// Save the SSH parameters in the active draft as a new reusable SSH Profile.
    pub fn save_draft_as_ssh_profile(&mut self) {
        match logic::save_ssh_profile(&mut self.ssh_profiles, &self.connection_draft) {
            Ok(id) => {
                self.connection_draft.ssh_profile_id = id;
                self.runtime_message = "SSH profile saved — reusable by other connections".to_owned();
            }
            Err(err) => {
                self.runtime_message = err;
            }
        }
    }

    /// Apply an existing SSH Profile onto the active draft.
    pub fn apply_ssh_profile(&mut self, profile_id: &str) {
        let Some(profile) = self.ssh_profiles.iter().find(|p| p.id == profile_id).cloned() else {
            return;
        };
        logic::apply_ssh_profile_to_draft(&mut self.connection_draft, &profile);
        self.connection_test_valid = false;
    }

    /// Apply a selected cloud preset to the active draft.
    pub fn apply_cloud_preset(&mut self) {
        let key = self.connection_draft.cloud_preset.clone();
        match mapper::apply_cloud_preset_to_draft(&mut self.connection_draft, &key) {
            Ok(guidance) => {
                self.connection_draft.cloud_guidance = guidance;
                self.connection_error.clear();
                self.connection_test_valid = false;
            }
            Err(err) => {
                self.connection_error = err;
            }
        }
    }

    /// Dispatch connection test or save command to runtime worker.
    pub fn dispatch_connection_command(&mut self, save: bool) {
        if let Err(err) = logic::validate_connection_draft(&self.connection_draft) {
            self.show_toast_error(err.clone());
            self.connection_error = err;
            return;
        }

        let request_id = self.task_bridge.next_request_id();
        let draft = self.connection_draft.clone();
        let command = logic::build_connection_command(draft, self.editing_connection_id.clone(), request_id, save);

        self.dispatch_command(command);
        self.pending_connection_request = Some(request_id);
        if save {
            self.connection_test_valid = false;
        } else {
            self.connection_test_valid = false;
            self.connection_test_draft = Some(self.connection_draft.clone());
        }
        self.connection_error.clear();
        self.runtime_message = if save {
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
        let report = logic::probe_draft_diagnostics(&self.connection_draft, auth_ok, auth_message);
        self.connection_diagnostics = Some(report);
    }
}
