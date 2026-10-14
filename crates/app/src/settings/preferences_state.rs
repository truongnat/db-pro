use super::settings_model::{AppSettings, SettingsSection};
use crate::editor::PredictionMode;

/// User preferences and settings-editor drafts owned by the settings feature.
#[derive(Debug)]
pub(crate) struct PreferencesState {
    pub(super) dark_mode: bool,
    pub(super) reduce_motion: bool,
    pub(super) settings: AppSettings,
    pub(super) section: SettingsSection,
    pub(super) keybindings_filter: String,
    pub(super) keybinding_edit_id: Option<String>,
    pub(super) keybinding_edit_draft: String,
    pub(super) prediction_mode: PredictionMode,
}

impl Default for PreferencesState {
    fn default() -> Self {
        Self {
            dark_mode: false,
            reduce_motion: false,
            settings: AppSettings::default(),
            section: SettingsSection::General,
            keybindings_filter: String::new(),
            keybinding_edit_id: None,
            keybinding_edit_draft: String::new(),
            prediction_mode: PredictionMode::default(),
        }
    }
}

#[cfg(test)]
#[path = "tests/preferences_state_tests.rs"]
mod tests;

