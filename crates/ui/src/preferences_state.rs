use super::settings_model::{AppSettings, SettingsSection};
use crate::editor::PredictionMode;

/// User preferences and settings-editor drafts owned by the settings feature.
#[derive(Debug)]
pub(crate) struct PreferencesState {
    pub dark_mode: bool,
    pub reduce_motion: bool,
    pub settings: AppSettings,
    pub section: SettingsSection,
    pub keybindings_filter: String,
    pub keybinding_edit_id: Option<String>,
    pub keybinding_edit_draft: String,
    pub prediction_mode: PredictionMode,
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
mod tests {
    use super::PreferencesState;

    #[test]
    fn default_preferences_are_light_and_non_predictive() {
        let state = PreferencesState::default();

        assert!(!state.dark_mode);
        assert!(!state.reduce_motion);
        assert!(state.keybindings_filter.is_empty());
        assert!(state.keybinding_edit_id.is_none());
        assert_eq!(state.prediction_mode, crate::editor::PredictionMode::Off);
    }
}
