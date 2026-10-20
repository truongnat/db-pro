use super::*;
use egui::RichText;

pub(super) struct SettingsSystemContext<'a> {
    pub(super) theme: DbProTheme,
    pub(super) preferences: &'a mut PreferencesState,
    pub(super) agent_auto_run_read_only: &'a mut bool,
    pub(super) agent_provider_label: &'a str,
}

impl SettingsSystemContext<'_> {
    pub(super) fn draw(&mut self, ui: &mut egui::Ui, section: SettingsSection) {
        match section {
            SettingsSection::DataGrid => self.draw_data_grid(ui),
            SettingsSection::Connections => self.draw_connections(ui),
            SettingsSection::Ai => self.draw_ai(ui),
            SettingsSection::Security => self.draw_security(ui),
            SettingsSection::Advanced => self.draw_advanced(ui),
            _ => {}
        }
    }

    fn draw_data_grid(&mut self, ui: &mut egui::Ui) {
        card_frame(self.theme).show(ui, |ui| {
            section_label(ui, "DATA GRID", self.theme);
            ui.add_space(10.0);
            ui.horizontal(|ui| {
                ui.label(RichText::new("Page size").color(self.theme.text_secondary));
                // item_spacing is zeroed in this shell — explicit gap or the
                // label fuses with the drag value ("Page size100").
                ui.add_space(8.0);
                ui.add(egui::DragValue::new(&mut self.preferences.settings.data_grid.page_size).range(25..=1_000));
            });
            Checkbox::new(&mut self.preferences.settings.data_grid.show_row_numbers, "Show row numbers", self.theme)
                .show(ui);
            Checkbox::new(&mut self.preferences.settings.data_grid.wrap_cell_text, "Wrap cell text", self.theme).show(ui);
        });
    }

    fn draw_connections(&mut self, ui: &mut egui::Ui) {
        card_frame(self.theme).show(ui, |ui| {
            section_label(ui, "CONNECTIONS", self.theme);
            ui.add_space(10.0);
            Checkbox::new(
                &mut self.preferences.settings.connections.auto_connect_last,
                "Reconnect last connection on startup",
                self.theme,
            )
            .show(ui);
            Checkbox::new(
                &mut self.preferences.settings.connections.default_ssl_prefer,
                "Prefer TLS for new server connections",
                self.theme,
            )
            .show(ui);
            ui.label(
                RichText::new("Passwords stay in secret storage — never written to settings JSON.")
                    .small()
                    .color(self.theme.text_muted),
            );
        });
    }

    fn draw_ai(&mut self, ui: &mut egui::Ui) {
        card_frame(self.theme).show(ui, |ui| {
            section_label(ui, "AI PROVIDERS", self.theme);
            ui.add_space(10.0);
            Checkbox::new(&mut self.preferences.settings.ai.enabled, "Enable Agent workspace", self.theme).show(ui);
            Checkbox::new(
                &mut self.preferences.settings.ai.auto_run_read_only,
                "Allow Agent to auto-run read-only queries",
                self.theme,
            )
            .show(ui);
            *self.agent_auto_run_read_only = self.preferences.settings.ai.auto_run_read_only;
            ui.label(
                RichText::new(format!("Active provider: {}", self.agent_provider_label))
                    .small()
                    .color(self.theme.text_secondary),
            );
            ui.label(
                RichText::new("API keys are configured in the Agent panel and stored as secrets.")
                    .small()
                    .color(self.theme.text_muted),
            );
        });
    }

    fn draw_security(&mut self, ui: &mut egui::Ui) {
        card_frame(self.theme).show(ui, |ui| {
            section_label(ui, "SECURITY", self.theme);
            ui.add_space(10.0);
            Checkbox::new(
                &mut self.preferences.settings.security.redact_secrets_in_logs,
                "Redact secrets in diagnostics and logs",
                self.theme,
            )
            .show(ui);
            Checkbox::new(
                &mut self.preferences.settings.security.lock_secret_export,
                "Block secret export by default",
                self.theme,
            )
            .show(ui);
        });
    }

    fn draw_advanced(&mut self, ui: &mut egui::Ui) {
        card_frame(self.theme).show(ui, |ui| {
            section_label(ui, "ADVANCED", self.theme);
            ui.add_space(10.0);
            Checkbox::new(
                &mut self.preferences.settings.advanced.verbose_runtime_log,
                "Verbose runtime logging",
                self.theme,
            )
            .show(ui);
            Checkbox::new(
                &mut self.preferences.settings.advanced.experimental_features,
                "Experimental features",
                self.theme,
            )
            .show(ui);
            ui.label(
                RichText::new(format!("Settings schema version {}", self.preferences.settings.version))
                    .small()
                    .color(self.theme.text_muted),
            );
        });
    }
}

#[cfg(test)]
#[path = "tests/settings_system_view_tests.rs"]
mod tests;

