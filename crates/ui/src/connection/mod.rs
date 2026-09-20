pub mod advanced_panels;
pub mod catalog;
pub mod config;
pub mod delete_dialog;
pub mod form_fields;
pub mod layout;
pub mod lifecycle;
pub mod logic;
pub mod mapper;
pub mod state;
pub mod view;

#[cfg(test)]
mod tests;

use crate::{UiConnectionDraft, UiConnectionSummary, UiDriver};

pub(crate) use catalog::ConnectionCatalogState;
pub(crate) use lifecycle::ConnectionLifecycleState;
pub(crate) use state::ConnectionDialogState;

/// Composition boundary for the connection feature.
///
/// The feature owns its saved-connection read model, lifecycle reducer state
/// and editor dialog together; the app root only composes this aggregate with
/// the other feature aggregates.
#[derive(Debug)]
pub(crate) struct ConnectionFeatureState {
    pub(super) catalog: ConnectionCatalogState,
    pub(super) lifecycle: ConnectionLifecycleState,
    pub(super) dialog: ConnectionDialogState,
}

impl Default for ConnectionFeatureState {
    fn default() -> Self {
        Self {
            catalog: ConnectionCatalogState::default(),
            lifecycle: ConnectionLifecycleState::with_fallback_name("Local PostgreSQL"),
            dialog: ConnectionDialogState::default(),
        }
    }
}

impl ConnectionFeatureState {
    pub(crate) fn open_new(&mut self) {
        self.lifecycle.clear_pending_request();
        self.dialog.transition(state::ConnectionDialogAction::OpenNew);
    }
}

/// Apply a driver choice to the connection draft.
pub(crate) fn select_connection_driver(draft: &mut UiConnectionDraft, driver: UiDriver) {
    logic::select_driver(draft, driver);
}

/// Transition the dialog into edit mode for a saved connection.
pub(crate) fn open_edit_connection(
    dialog: &mut ConnectionDialogState,
    lifecycle: &mut ConnectionLifecycleState,
    connection: &UiConnectionSummary,
) {
    lifecycle.clear_pending_request();
    dialog.transition(state::ConnectionDialogAction::OpenEdit {
        connection_id: connection.id.clone(),
        draft: mapper::summary_to_edit_draft(connection),
    });
}

/// Transition the dialog into duplicate mode for a saved connection.
pub(crate) fn open_duplicate_connection(
    dialog: &mut ConnectionDialogState,
    lifecycle: &mut ConnectionLifecycleState,
    connection: &UiConnectionSummary,
) {
    lifecycle.clear_pending_request();
    dialog.transition(state::ConnectionDialogAction::OpenDuplicate {
        draft: mapper::summary_to_duplicate_draft(connection),
    });
}

/// Save the active draft's SSH parameters as a reusable profile.
pub(crate) fn save_draft_as_ssh_profile(dialog: &mut ConnectionDialogState) -> Result<String, String> {
    logic::save_ssh_profile(&mut dialog.ssh_profiles, &dialog.draft)
}

/// Apply a reusable SSH profile to the active draft.
pub(crate) fn apply_ssh_profile(dialog: &mut ConnectionDialogState, profile_id: &str) -> bool {
    let Some(profile) = dialog
        .ssh_profiles
        .iter()
        .find(|profile| profile.id == profile_id)
        .cloned()
    else {
        return false;
    };
    logic::apply_ssh_profile_to_draft(&mut dialog.draft, &profile);
    dialog.set_test_valid(false);
    true
}

/// Apply the selected cloud preset to the active draft.
pub(crate) fn apply_cloud_preset(dialog: &mut ConnectionDialogState) -> Result<(), String> {
    let key = dialog.draft.cloud_preset.clone();
    let guidance = mapper::apply_cloud_preset_to_draft(&mut dialog.draft, &key)?;
    dialog.draft.cloud_guidance = guidance;
    dialog.clear_error();
    dialog.set_test_valid(false);
    Ok(())
}

/// Refresh the diagnostic report for the active draft.
pub(crate) fn refresh_connection_diagnostics(dialog: &mut ConnectionDialogState, auth_ok: bool, auth_message: &str) {
    dialog.diagnostics = Some(logic::probe_draft_diagnostics(&dialog.draft, auth_ok, auth_message));
}
