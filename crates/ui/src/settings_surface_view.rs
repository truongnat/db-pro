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
        let mut actions = Vec::new();
        let available = ui.available_size();
        ui.set_min_size(available);

        egui::Frame {
            fill: self.theme.surface_app,
            inner_margin: egui::Margin::ZERO,
            outer_margin: egui::Margin::ZERO,
            stroke: egui::Stroke::NONE,
            rounding: egui::Rounding::ZERO,
            ..Default::default()
        }
        .show(ui, |ui| {
            ui.set_min_size(ui.available_size());
            ui.with_layout(egui::Layout::left_to_right(egui::Align::Min), |ui| {
                let nav_width = (ui.available_width() * 0.22).clamp(196.0, 244.0);
                let available_height = ui.available_height();
                self.draw_navigation_panel(ui, nav_width, available_height, &mut actions);
                self.draw_content_panel(ui, available_height, &mut actions);
            });
        });
        actions
    }

    fn draw_navigation_panel(
        &self,
        ui: &mut egui::Ui,
        nav_width: f32,
        available_height: f32,
        actions: &mut Vec<SettingsSurfaceAction>,
    ) {
        ui.allocate_ui_with_layout(
            egui::vec2(nav_width, available_height),
            egui::Layout::top_down(egui::Align::Min),
            |ui| {
                egui::Frame {
                    fill: self.theme.surface_panel,
                    inner_margin: egui::Margin {
                        left: SPACE_LG,
                        right: SPACE_LG,
                        top: SPACE_2XL,
                        bottom: SPACE_LG,
                    },
                    stroke: egui::Stroke::NONE,
                    rounding: egui::Rounding::ZERO,
                    ..Default::default()
                }
                .show(ui, |ui| {
                    ui.set_min_size(ui.available_size());
                    actions.extend(self.draw_navigation(ui));
                });
            },
        );

        let (divider, _) = ui.allocate_exact_size(egui::vec2(STROKE_THIN, available_height), egui::Sense::hover());
        ui.painter().rect_filled(divider, egui::Rounding::ZERO, self.theme.border_subtle);
    }

    fn draw_content_panel(
        &mut self,
        ui: &mut egui::Ui,
        available_height: f32,
        actions: &mut Vec<SettingsSurfaceAction>,
    ) {
        let content_width = ui.available_width();
        ui.allocate_ui_with_layout(
            egui::vec2(content_width, available_height),
            egui::Layout::top_down(egui::Align::Min),
            |ui| {
                egui::ScrollArea::vertical()
                    .id_salt("settings-content")
                    .auto_shrink([false, false])
                    .show(ui, |ui| self.draw_content(ui, actions));
            },
        );
    }

    fn draw_content(&mut self, ui: &mut egui::Ui, actions: &mut Vec<SettingsSurfaceAction>) {
        ui.set_width((ui.available_width() - SPACE_4XL * 2.0).max(0.0));
        ui.add_space(SPACE_3XL);
        ui.label(
            egui::RichText::new("Settings")
                .font(font_page_title())
                .strong()
                .color(self.theme.text_primary),
        );
        ui.add_space(SPACE_XS);
        ui.label(
            egui::RichText::new("Shape the database workspace around the way you work.")
                .font(font_body_sm())
                .color(self.theme.text_muted),
        );
        ui.add_space(SPACE_LG);
        ui.separator();
        ui.add_space(SPACE_LG);
        ui.label(
            egui::RichText::new(self.selected.label())
                .font(font_section_title())
                .strong()
                .color(self.theme.text_primary),
        );
        ui.add_space(SPACE_MD);
        self.draw_selected_section(ui, actions);
        ui.add_space(SPACE_2XL);
        actions.extend(
            SettingsDiagnosticsContext {
                theme: *self.theme,
                summary: self.diagnostics,
            }
            .draw(ui)
            .into_iter()
            .map(SettingsSurfaceAction::Diagnostics),
        );
        ui.add_space(SPACE_3XL);
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
