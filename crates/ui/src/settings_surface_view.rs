//! Settings surface composition and typed intents.
use super::settings_appearance_view::SettingsAppearanceContext;
use super::settings_backup_view::{SettingsBackupAction, SettingsBackupContext};
use super::settings_diagnostics_view::{SettingsDiagnosticsAction, SettingsDiagnosticsContext};
use super::settings_editor_view::SettingsEditorContext;
use super::settings_general_view::{SettingsGeneralAction, SettingsGeneralContext};
use super::settings_keybindings_view::{SettingsKeybindingsAction, SettingsKeybindingsContext};
use super::settings_navigation_view::{SettingsNavigationAction, SettingsNavigationContext};
use super::settings_system_view::SettingsSystemContext;
use super::*;
use db_pro_core::domain::diagnostics::DiagnosticsSummary;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum SettingsSurfaceAction {
    Navigation(SettingsNavigationAction),
    General(SettingsGeneralAction),
    Keybindings(SettingsKeybindingsAction),
    Backup(SettingsBackupAction),
    Diagnostics(SettingsDiagnosticsAction),
}

pub(super) struct SettingsSurfaceContext<'a> {
    pub(super) theme: &'a mut DbProTheme,
    pub(super) selected: SettingsSection,
    pub(super) preferences: &'a mut PreferencesState,
    pub(super) query: &'a mut QueryFeatureState,
    pub(super) sessions: &'a mut WorkspaceSessionState,
    pub(super) overlay: &'a mut OverlayState,
    pub(super) agent_auto_run_read_only: &'a mut bool,
    pub(super) agent_provider_label: &'a str,
    pub(super) active_driver: &'a str,
    pub(super) backup_supported: bool,
    pub(super) diagnostics: &'a DiagnosticsSummary,
}

impl SettingsSurfaceContext<'_> {
    pub(super) fn draw(&mut self, ui: &mut egui::Ui) -> Vec<SettingsSurfaceAction> {
        let mut actions = self.draw_navigation(ui);
        ui.separator();
        self.draw_selected_section(ui, &mut actions);
        ui.add_space(SPACE_MD);
        actions.extend(
            SettingsDiagnosticsContext {
                theme: *self.theme,
                summary: self.diagnostics,
            }
            .draw(ui)
            .into_iter()
            .map(SettingsSurfaceAction::Diagnostics),
        );
        actions
    }

    fn draw_navigation(&self, ui: &mut egui::Ui) -> Vec<SettingsSurfaceAction> {
        SettingsNavigationContext {
            theme: *self.theme,
            selected: self.selected,
        }
        .draw(ui)
        .into_iter()
        .map(SettingsSurfaceAction::Navigation)
        .collect()
    }

    fn draw_selected_section(&mut self, ui: &mut egui::Ui, actions: &mut Vec<SettingsSurfaceAction>) {
        match self.selected {
            SettingsSection::General => self.draw_general(ui, actions),
            SettingsSection::Appearance => self.draw_appearance(ui),
            SettingsSection::Editor => self.draw_editor(ui),
            SettingsSection::DataGrid
            | SettingsSection::Connections
            | SettingsSection::Ai
            | SettingsSection::Security
            | SettingsSection::Advanced => self.draw_system(ui),
            SettingsSection::Keybindings => self.draw_keybindings(ui, actions),
            SettingsSection::Backup => self.draw_backup(ui, actions),
        }
    }

    fn draw_general(&mut self, ui: &mut egui::Ui, actions: &mut Vec<SettingsSurfaceAction>) {
        actions.extend(
            SettingsGeneralContext {
                theme: *self.theme,
                preferences: self.preferences,
                sessions: self.sessions,
            }
            .draw(ui)
            .into_iter()
            .map(SettingsSurfaceAction::General),
        );
    }

    fn draw_appearance(&mut self, ui: &mut egui::Ui) {
        SettingsAppearanceContext {
            theme: self.theme,
            preferences: self.preferences,
        }
        .draw(ui);
    }

    fn draw_editor(&mut self, ui: &mut egui::Ui) {
        SettingsEditorContext {
            theme: *self.theme,
            preferences: self.preferences,
            query: self.query,
        }
        .draw(ui);
    }

    fn draw_system(&mut self, ui: &mut egui::Ui) {
        SettingsSystemContext {
            theme: *self.theme,
            preferences: self.preferences,
            agent_auto_run_read_only: self.agent_auto_run_read_only,
            agent_provider_label: self.agent_provider_label,
        }
        .draw(ui, self.selected);
    }

    fn draw_keybindings(&mut self, ui: &mut egui::Ui, actions: &mut Vec<SettingsSurfaceAction>) {
        actions.extend(
            SettingsKeybindingsContext {
                theme: *self.theme,
                preferences: self.preferences,
            }
            .draw(ui)
            .into_iter()
            .map(SettingsSurfaceAction::Keybindings),
        );
    }

    fn draw_backup(&mut self, ui: &mut egui::Ui, actions: &mut Vec<SettingsSurfaceAction>) {
        actions.extend(
            SettingsBackupContext {
                theme: *self.theme,
                overlay: self.overlay,
                active_driver: self.active_driver,
                supported: self.backup_supported,
            }
            .draw(ui)
            .into_iter()
            .map(SettingsSurfaceAction::Backup),
        );
    }
}
