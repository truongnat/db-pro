//! Diagnostics settings presentation and export intents.

use super::*;
use db_pro_core::domain::diagnostics::DiagnosticsSummary;
use lucide_icons::Icon;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum SettingsDiagnosticsAction {
    CopySummary,
    ExportBundle,
}

pub(crate) struct SettingsDiagnosticsContext<'a> {
    pub(crate) theme: DbProTheme,
    pub(crate) summary: &'a DiagnosticsSummary,
}

impl SettingsDiagnosticsContext<'_> {
    pub(crate) fn draw(&self, ui: &mut egui::Ui) -> Vec<SettingsDiagnosticsAction> {
        let mut actions = Vec::new();
        card_frame(self.theme).show(ui, |ui| {
            self.draw_summary(ui);
            actions.extend(self.draw_actions(ui));
            self.draw_redaction_note(ui);
        });
        actions
    }

    fn draw_summary(&self, ui: &mut egui::Ui) {
        section_label(ui, "DIAGNOSTICS", self.theme);
        ui.add_space(10.0);
        ui.label(
            RichText::new(format!(
                "DB Pro {} · {} / {}",
                self.summary.app_version, self.summary.os, self.summary.architecture
            ))
            .color(self.theme.text_primary),
        );
        ui.label(
            RichText::new(format!(
                "Drivers: {}",
                self.summary
                    .drivers
                    .iter()
                    .map(|driver| format!("{}{}", driver.driver, if driver.available { "" } else { " (n/a)" }))
                    .collect::<Vec<_>>()
                    .join(", ")
            ))
            .small()
            .color(self.theme.text_secondary),
        );
        ui.label(
            RichText::new(format!(
                "Connections: {} · active executions tracked: {}",
                self.summary.connections.len(),
                self.summary.runtime.active_executions
            ))
            .small()
            .color(self.theme.text_secondary),
        );
        if self.summary.recent_errors.is_empty() {
            ui.label(
                RichText::new("No recent structured errors in this session.")
                    .small()
                    .color(self.theme.text_muted),
            );
        } else {
            for error in self.summary.recent_errors.iter().take(5) {
                ui.label(
                    RichText::new(format!("• [{}] {}", error.error_code, error.message))
                        .small()
                        .color(self.theme.warning),
                );
            }
        }
        ui.add_space(8.0);
    }

    fn draw_actions(&self, ui: &mut egui::Ui) -> Vec<SettingsDiagnosticsAction> {
        let mut actions = Vec::new();
        ui.horizontal(|ui| {
            if compact_button_with_icon(ui, Icon::Copy, "Copy diagnostics summary", self.theme).clicked() {
                actions.push(SettingsDiagnosticsAction::CopySummary);
            }
            if compact_button_with_icon(ui, Icon::Download, "Export support bundle", self.theme).clicked() {
                actions.push(SettingsDiagnosticsAction::ExportBundle);
            }
        });
        actions
    }

    fn draw_redaction_note(&self, ui: &mut egui::Ui) {
        ui.label(
            RichText::new("Passwords, tokens, and embedded URL credentials are redacted.")
                .small()
                .color(self.theme.text_muted),
        );
    }
}
