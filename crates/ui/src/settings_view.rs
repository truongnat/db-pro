//! Settings activity sidebar panels (#205).
use super::*;
use super::{overlay_state::OverlayState, RequestId, UiCommand};
use crate::editor::PredictionMode;

pub(crate) fn apply_settings_state(
    preferences: &mut PreferencesState,
    query: &mut QueryFeatureState,
    agent: &mut AgentState,
    theme: &mut DbProTheme,
) {
    preferences.settings.general.language.apply();
    preferences.dark_mode = preferences.settings.appearance.dark_mode;
    preferences.reduce_motion = preferences.settings.appearance.reduce_motion;
    query.editor.editor_font_size = preferences.settings.editor.font_size;
    preferences.prediction_mode = match preferences.settings.editor.prediction_mode.as_str() {
        "off" => PredictionMode::Off,
        "subtle" => PredictionMode::Subtle,
        _ => PredictionMode::Eager,
    };
    agent.auto_run_read_only = preferences.settings.ai.auto_run_read_only;
    *theme = if preferences.dark_mode {
        DbProTheme::dark()
    } else {
        DbProTheme::light()
    };
}

pub(crate) fn sync_settings_state(preferences: &mut PreferencesState, query: &QueryFeatureState, agent: &AgentState) {
    preferences.settings.version = settings_model::SETTINGS_VERSION;
    preferences.settings.appearance.dark_mode = preferences.dark_mode;
    preferences.settings.appearance.reduce_motion = preferences.reduce_motion;
    preferences.settings.editor.font_size = query.editor.editor_font_size;
    preferences.settings.editor.prediction_mode = match preferences.prediction_mode {
        PredictionMode::Off => "off".to_owned(),
        PredictionMode::Subtle => "subtle".to_owned(),
        PredictionMode::Eager => "eager".to_owned(),
    };
    preferences.settings.ai.auto_run_read_only = agent.auto_run_read_only;
    preferences.settings.ai.provider_label = agent.provider_label.clone();
}

impl DbProApp {
    pub(crate) fn sync_settings_from_runtime(&mut self) {
        sync_settings_state(&mut self.preferences, &self.query, &self.agent);
    }

    pub(super) fn draw_settings(&mut self, ui: &mut egui::Ui) {
        let summary = self.build_diagnostics_summary();
        let active_driver = self.active_driver().to_owned();
        let backup_supported = self.supports_backup_restore();
        let context = ui.ctx().clone();
        let actions = {
            let mut context = settings_surface_view::SettingsSurfaceContext {
                theme: &mut self.theme,
                selected: self.preferences.section,
                preferences: &mut self.preferences,
                query: &mut self.query,
                sessions: &mut self.workspace.sessions,
                overlay: &mut self.overlay,
                agent_auto_run_read_only: &mut self.agent.auto_run_read_only,
                agent_provider_label: &self.agent.provider_label,
                active_driver: &active_driver,
                backup_supported,
                diagnostics: &summary,
            };
            context.draw(ui)
        };
        for action in actions {
            self.apply_settings_surface_action(action, &summary, &context);
        }
    }

    fn apply_settings_surface_action(
        &mut self,
        action: settings_surface_view::SettingsSurfaceAction,
        summary: &db_pro_core::domain::diagnostics::DiagnosticsSummary,
        context: &egui::Context,
    ) {
        match action {
            settings_surface_view::SettingsSurfaceAction::Navigation(action) => {
                let settings_navigation_view::SettingsNavigationAction::SelectSection(section) = action;
                self.preferences.section = section;
            }
            settings_surface_view::SettingsSurfaceAction::General(action) => match action {
                settings_general_view::SettingsGeneralAction::Save => self.save_named_workspace_session(),
                settings_general_view::SettingsGeneralAction::Restore(id) => self.restore_named_workspace_session(&id),
                settings_general_view::SettingsGeneralAction::Duplicate(id) => {
                    self.duplicate_named_workspace_session(&id)
                }
                settings_general_view::SettingsGeneralAction::Delete(id) => {
                    self.workspace.sessions.remove(&id);
                }
            },
            settings_surface_view::SettingsSurfaceAction::Keybindings(action) => {
                if matches!(action, settings_keybindings_view::SettingsKeybindingsAction::ResetAll) {
                    self.feedback.runtime_message = "Keybindings reset to defaults".to_owned();
                }
            }
            settings_surface_view::SettingsSurfaceAction::Backup(action) => self.apply_backup_actions(vec![action]),
            settings_surface_view::SettingsSurfaceAction::Diagnostics(action) => match action {
                settings_diagnostics_view::SettingsDiagnosticsAction::CopySummary => {
                    if let Ok(json) = serde_json::to_string_pretty(summary) {
                        self.feedback.runtime_message = "Diagnostics summary copied (secrets redacted)".to_owned();
                        context.copy_text(json);
                    }
                }
                settings_diagnostics_view::SettingsDiagnosticsAction::ExportBundle => {
                    match self.export_support_bundle() {
                        Ok(path) => {
                            self.feedback.runtime_message =
                                format!("Support bundle written to {path} (secrets redacted)");
                        }
                        Err(error) => {
                            self.feedback.runtime_message = format!("Support bundle export failed: {error}");
                        }
                    }
                }
            },
        }
    }

    fn apply_backup_actions(&mut self, actions: Vec<settings_backup_view::SettingsBackupAction>) {
        use settings_backup_view::SettingsBackupAction;

        for action in actions {
            match action {
                SettingsBackupAction::PickBackup => {
                    let request_id = self.next_request_id();
                    self.dispatch_command(pick_backup_command(request_id));
                }
                SettingsBackupAction::CreateBackup => {
                    if let Some(connection) = self.active_connection().cloned() {
                        let request_id = self.next_request_id();
                        self.dispatch_command(backup_command(&self.overlay, request_id, connection.id));
                    }
                }
                SettingsBackupAction::PickRestore => {
                    let request_id = self.next_request_id();
                    self.dispatch_command(pick_restore_command(request_id));
                }
                SettingsBackupAction::ConfirmRestore => {
                    if let Some(connection) = self.active_connection().cloned() {
                        let request_id = self.next_request_id();
                        self.dispatch_command(restore_command(&self.overlay, request_id, connection.id));
                    }
                }
            }
        }
    }

    fn supports_backup_restore(&self) -> bool {
        self.active_capabilities()
            .allows(|capabilities| capabilities.features.backup)
    }
}

fn pick_backup_command(request_id: RequestId) -> UiCommand {
    UiCommand::PickBackupFile { request_id }
}

fn backup_command(state: &OverlayState, request_id: RequestId, connection_id: String) -> UiCommand {
    UiCommand::Backup {
        request_id,
        connection_id,
        output_path: state.backup_output_path.clone(),
        custom_format: false,
    }
}

fn pick_restore_command(request_id: RequestId) -> UiCommand {
    UiCommand::PickRestoreFile { request_id }
}

fn restore_command(state: &OverlayState, request_id: RequestId, connection_id: String) -> UiCommand {
    UiCommand::Restore {
        request_id,
        connection_id,
        input_path: state.restore_input_path.clone(),
        custom_format: false,
    }
}

#[cfg(test)]
mod tests {
    use super::{backup_command, restore_command, OverlayState, RequestId, UiCommand};

    #[test]
    fn backup_and_restore_commands_read_overlay_paths() {
        let state = OverlayState {
            backup_output_path: "/tmp/backup.sql".to_owned(),
            restore_input_path: "/tmp/input.sql".to_owned(),
            ..OverlayState::default()
        };

        assert!(matches!(
            backup_command(&state, RequestId(1), "source".to_owned()),
            UiCommand::Backup { output_path, connection_id, custom_format: false, .. }
                if output_path == "/tmp/backup.sql" && connection_id == "source"
        ));
        assert!(matches!(
            restore_command(&state, RequestId(2), "source".to_owned()),
            UiCommand::Restore { input_path, connection_id, custom_format: false, .. }
                if input_path == "/tmp/input.sql" && connection_id == "source"
        ));
    }
}
