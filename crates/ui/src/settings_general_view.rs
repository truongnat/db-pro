//! General settings presentation and workspace-session intents.

use super::workspace_session::WorkspaceSession;
use super::*;
use crate::components::button::{Button, ButtonSize, ButtonVariant};

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum SettingsGeneralAction {
    Save,
    Restore(String),
    Duplicate(String),
    Delete(String),
}

pub(crate) struct SettingsGeneralContext<'a> {
    pub(crate) theme: DbProTheme,
    pub(crate) preferences: &'a mut PreferencesState,
    pub(crate) sessions: &'a mut WorkspaceSessionState,
}

impl SettingsGeneralContext<'_> {
    pub(crate) fn draw(&mut self, ui: &mut egui::Ui) -> Vec<SettingsGeneralAction> {
        let mut actions = Vec::new();
        card_frame(self.theme).show(ui, |ui| {
            self.draw_general_preferences(ui);
            actions.extend(self.draw_workspace_sessions(ui));
        });
        actions
    }

    fn draw_general_preferences(&mut self, ui: &mut egui::Ui) {
        section_label(ui, "GENERAL", self.theme);
        ui.add_space(10.0);
        ui.horizontal(|ui| {
            ui.label(RichText::new("Language").color(self.theme.text_secondary));
            for language in crate::UiLanguage::ALL {
                if ui
                    .selectable_label(
                        self.preferences.settings.general.language == *language,
                        language.label(),
                    )
                    .clicked()
                {
                    self.preferences.settings.general.language = *language;
                    language.apply();
                }
            }
        });
        ui.add_space(8.0);
        ui.checkbox(
            &mut self.preferences.settings.general.confirm_destructive_queries,
            "Confirm destructive queries",
        );
        ui.checkbox(
            &mut self.preferences.settings.general.restore_tabs_on_startup,
            "Restore query tabs on startup",
        );
    }

    fn draw_workspace_sessions(&mut self, ui: &mut egui::Ui) -> Vec<SettingsGeneralAction> {
        let mut actions = Vec::new();
        ui.add_space(12.0);
        section_label(ui, "WORKSPACE SESSIONS", self.theme);
        ui.add_space(6.0);
        ui.label(
            RichText::new("Named sessions store layout and tab references — not SQL text, secrets, or result grids.")
                .small()
                .color(self.theme.text_muted),
        );
        input_full_width(ui, &mut self.sessions.name_draft, "Session name", self.theme);
        ui.horizontal(|ui| {
            if Button::new(self.theme)
                .text("Save workspace")
                .variant(ButtonVariant::Default)
                .size(ButtonSize::Sm)
                .show(ui)
                .clicked()
            {
                actions.push(SettingsGeneralAction::Save);
            }
        });
        ui.add_space(SPACE_SM);
        let sessions = self.sessions.store.sessions.clone();
        for session in sessions {
            actions.extend(self.draw_session_row(ui, &session));
        }
        if !self.sessions.last_restore_notes.is_empty() {
            ui.add_space(6.0);
            for note in &self.sessions.last_restore_notes {
                ui.label(RichText::new(note).small().color(self.theme.warning));
            }
        }
        actions
    }

    fn draw_session_row(&mut self, ui: &mut egui::Ui, session: &WorkspaceSession) -> Vec<SettingsGeneralAction> {
        let mut actions = Vec::new();
        ui.horizontal(|ui| {
            let selected = self.sessions.selected_id.as_deref() == Some(session.id.as_str());
            if ui.selectable_label(selected, &session.name).clicked() {
                self.sessions.selected_id = Some(session.id.clone());
            }
            if Button::new(self.theme)
                .text("Restore")
                .variant(ButtonVariant::Secondary)
                .size(ButtonSize::Sm)
                .show(ui)
                .clicked()
            {
                actions.push(SettingsGeneralAction::Restore(session.id.clone()));
            }
            if Button::new(self.theme)
                .text("Duplicate")
                .variant(ButtonVariant::Secondary)
                .size(ButtonSize::Sm)
                .show(ui)
                .clicked()
            {
                actions.push(SettingsGeneralAction::Duplicate(session.id.clone()));
            }
            if Button::new(self.theme)
                .text("Delete")
                .variant(ButtonVariant::Destructive)
                .size(ButtonSize::Sm)
                .show(ui)
                .clicked()
            {
                actions.push(SettingsGeneralAction::Delete(session.id.clone()));
            }
        });
        actions
    }
}
