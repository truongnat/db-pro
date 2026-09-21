//! Settings activity sidebar panels (#205).
use super::settings_diagnostics_view::{SettingsDiagnosticsAction, SettingsDiagnosticsContext};
use super::settings_editor_view::SettingsEditorContext;
use super::settings_general_view::{SettingsGeneralAction, SettingsGeneralContext};
use super::settings_keybindings_view::{SettingsKeybindingsAction, SettingsKeybindingsContext};
use super::settings_navigation_view::{SettingsNavigationAction, SettingsNavigationContext};
use super::*;
use crate::editor::PredictionMode;
use egui::RichText;

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
        ui.horizontal(|ui| {
            ui.vertical(|ui| {
                let actions = SettingsNavigationContext {
                    theme: self.theme,
                    selected: self.preferences.section,
                }
                .draw(ui);
                for action in actions {
                    let SettingsNavigationAction::SelectSection(section) = action;
                    self.preferences.section = section;
                }
            });
            ui.separator();
            ui.vertical(|ui| match self.preferences.section {
                SettingsSection::General => self.draw_general_settings(ui),
                SettingsSection::Appearance => settings_appearance_view::SettingsAppearanceContext {
                    theme: &mut self.theme,
                    preferences: &mut self.preferences,
                }
                .draw(ui),
                SettingsSection::Editor => self.draw_editor_settings(ui),
                SettingsSection::DataGrid
                | SettingsSection::Connections
                | SettingsSection::Ai
                | SettingsSection::Security
                | SettingsSection::Advanced => self.draw_system_settings(ui),
                SettingsSection::Keybindings => self.draw_keybindings_settings(ui),
                SettingsSection::Backup => self.draw_backup_section(ui),
            });
        });
        ui.add_space(12.0);
        self.draw_diagnostics_settings(ui);
    }

    fn draw_general_settings(&mut self, ui: &mut egui::Ui) {
        let actions = SettingsGeneralContext {
            theme: self.theme,
            preferences: &mut self.preferences,
            sessions: &mut self.workspace.sessions,
        }
        .draw(ui);
        for action in actions {
            match action {
                SettingsGeneralAction::Save => self.save_named_workspace_session(),
                SettingsGeneralAction::Restore(id) => self.restore_named_workspace_session(&id),
                SettingsGeneralAction::Duplicate(id) => self.duplicate_named_workspace_session(&id),
                SettingsGeneralAction::Delete(id) => {
                    self.workspace.sessions.remove(&id);
                }
            }
        }
    }

    fn draw_editor_settings(&mut self, ui: &mut egui::Ui) {
        SettingsEditorContext {
            theme: self.theme,
            preferences: &mut self.preferences,
            query: &mut self.query,
        }
        .draw(ui);
    }

    fn draw_keybindings_settings(&mut self, ui: &mut egui::Ui) {
        let actions = SettingsKeybindingsContext {
            theme: self.theme,
            preferences: &mut self.preferences,
        }
        .draw(ui);
        for action in actions {
            if matches!(action, SettingsKeybindingsAction::ResetAll) {
                self.feedback.runtime_message = "Keybindings reset to defaults".to_owned();
            }
        }
    }

    fn draw_system_settings(&mut self, ui: &mut egui::Ui) {
        let section = self.preferences.section;
        let provider_label = self.agent.provider_label.clone();
        settings_system_view::SettingsSystemContext {
            theme: self.theme,
            preferences: &mut self.preferences,
            agent_auto_run_read_only: &mut self.agent.auto_run_read_only,
            agent_provider_label: &provider_label,
        }
        .draw(ui, section);
    }

    fn draw_backup_section(&mut self, ui: &mut egui::Ui) {
        card_frame(self.theme).show(ui, |ui| {
            section_label(ui, "DATABASE FILES", self.theme);
            if !self.supports_backup_restore() {
                ui.label(
                    RichText::new("Backup and restore are unavailable for the active provider")
                        .small()
                        .color(self.theme.text_muted),
                );
                return;
            }
            ui.add_space(10.0);
            let tool_hint = if self.active_driver().eq_ignore_ascii_case("sqlite") {
                "SQLite uses VACUUM INTO for consistent snapshots (including WAL). Restore refuses while the connection is active — disconnect first."
            } else if self.active_driver().eq_ignore_ascii_case("mysql") {
                "MySQL backup/restore is not available yet."
            } else {
                "PostgreSQL backups require `pg_dump` on PATH; restores use `psql` (plain) or `pg_restore` (custom). Missing tools are detected before spawn."
            };
            ui.label(
                RichText::new(tool_hint)
                    .small()
                    .color(self.theme.text_muted),
            );
            ui.add_space(10.0);
            let actions = settings_backup_view::SettingsBackupContext {
                theme: self.theme,
                overlay: &mut self.overlay,
            }
            .draw(ui);
            self.apply_backup_actions(actions);
        });
    }

    fn apply_backup_actions(&mut self, actions: Vec<settings_backup_view::SettingsBackupAction>) {
        use settings_backup_view::SettingsBackupAction;

        for action in actions {
            match action {
                SettingsBackupAction::PickBackup => {
                    let request_id = self.task_bridge.next_request_id();
                    self.dispatch_command(self.overlay.pick_backup_command(request_id));
                }
                SettingsBackupAction::CreateBackup => {
                    if let Some(connection) = self.active_connection().cloned() {
                        let request_id = self.task_bridge.next_request_id();
                        self.dispatch_command(self.overlay.backup_command(request_id, connection.id));
                    }
                }
                SettingsBackupAction::PickRestore => {
                    let request_id = self.task_bridge.next_request_id();
                    self.dispatch_command(self.overlay.pick_restore_command(request_id));
                }
                SettingsBackupAction::ConfirmRestore => {
                    if let Some(connection) = self.active_connection().cloned() {
                        let request_id = self.task_bridge.next_request_id();
                        self.dispatch_command(self.overlay.restore_command(request_id, connection.id));
                    }
                }
            }
        }
    }

    fn draw_diagnostics_settings(&mut self, ui: &mut egui::Ui) {
        let summary = self.build_diagnostics_summary();
        let actions = SettingsDiagnosticsContext {
            theme: self.theme,
            summary: &summary,
        }
        .draw(ui);
        for action in actions {
            match action {
                SettingsDiagnosticsAction::CopySummary => {
                    if let Ok(json) = serde_json::to_string_pretty(&summary) {
                        ui.ctx().copy_text(json);
                        self.feedback.runtime_message = "Diagnostics summary copied (secrets redacted)".to_owned();
                    }
                }
                SettingsDiagnosticsAction::ExportBundle => match self.export_support_bundle() {
                    Ok(path) => {
                        self.feedback.runtime_message = format!("Support bundle written to {path} (secrets redacted)");
                    }
                    Err(error) => {
                        self.feedback.runtime_message = format!("Support bundle export failed: {error}");
                    }
                },
            }
        }
    }

    fn supports_backup_restore(&self) -> bool {
        self.active_capabilities()
            .allows(|capabilities| capabilities.features.backup)
    }
}
