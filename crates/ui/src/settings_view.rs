//! Settings activity sidebar panels.
use super::*;
use egui::RichText;
use lucide_icons::Icon;

impl DbProApp {
    pub(super) fn draw_settings(&mut self, ui: &mut egui::Ui) {
        self.draw_appearance_settings(ui);
        ui.add_space(12.0);
        self.draw_editor_settings(ui);
        ui.add_space(12.0);
        self.draw_diagnostics_settings(ui);
        ui.add_space(12.0);
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
            self.draw_backup_settings(ui);
            ui.add_space(14.0);
            self.draw_restore_settings(ui);
        });
    }

    fn draw_editor_settings(&mut self, ui: &mut egui::Ui) {
        card_frame(self.theme).show(ui, |ui| {
            section_label(ui, "EDITOR", self.theme);
            ui.add_space(10.0);
            ui.horizontal(|ui| {
                ui.label(RichText::new("Font size").color(self.theme.text_secondary));
                if compact_button(ui, "−", self.theme).clicked() {
                    self.editor_font_size = (self.editor_font_size - 1.0).max(10.0);
                }
                ui.label(RichText::new(format!("{:.0} px", self.editor_font_size)).color(self.theme.text_primary));
                if compact_button(ui, "+", self.theme).clicked() {
                    self.editor_font_size = (self.editor_font_size + 1.0).min(24.0);
                }
            });
            ui.add_space(8.0);
            ui.label(RichText::new("AI prediction").color(self.theme.text_secondary));
            ui.horizontal_wrapped(|ui| {
                for (mode, label) in [
                    (PredictionMode::Off, "Off"),
                    (PredictionMode::Subtle, "Subtle"),
                    (PredictionMode::Eager, "Eager"),
                ] {
                    if ui.selectable_label(self.prediction_mode == mode, label).clicked() {
                        self.prediction_mode = mode;
                    }
                }
            });
            ui.label(
                RichText::new(
                    "Sends the SQL around your cursor and its schema context to your configured AI provider.",
                )
                .small()
                .color(self.theme.text_muted),
            );
        });
    }

    fn draw_diagnostics_settings(&mut self, ui: &mut egui::Ui) {
        card_frame(self.theme).show(ui, |ui| {
            section_label(ui, "DIAGNOSTICS", self.theme);
            ui.add_space(10.0);
            let summary = self.build_diagnostics_summary();
            ui.label(
                RichText::new(format!(
                    "DB Pro {} · {} / {}",
                    summary.app_version, summary.os, summary.architecture
                ))
                .color(self.theme.text_primary),
            );
            ui.label(
                RichText::new(format!(
                    "Drivers: {}",
                    summary
                        .drivers
                        .iter()
                        .map(|d| format!("{}{}", d.driver, if d.available { "" } else { " (n/a)" }))
                        .collect::<Vec<_>>()
                        .join(", ")
                ))
                .small()
                .color(self.theme.text_secondary),
            );
            ui.label(
                RichText::new(format!(
                    "Connections: {} · active executions tracked: {}",
                    summary.connections.len(),
                    summary.runtime.active_executions
                ))
                .small()
                .color(self.theme.text_secondary),
            );
            if summary.recent_errors.is_empty() {
                ui.label(
                    RichText::new("No recent structured errors in this session.")
                        .small()
                        .color(self.theme.text_muted),
                );
            } else {
                for error in summary.recent_errors.iter().take(5) {
                    ui.label(
                        RichText::new(format!("• [{}] {}", error.error_code, error.message))
                            .small()
                            .color(self.theme.warning),
                    );
                }
            }
            ui.add_space(8.0);
            ui.horizontal(|ui| {
                if compact_button_with_icon(ui, Icon::Copy, "Copy diagnostics summary", self.theme).clicked() {
                    if let Ok(json) = serde_json::to_string_pretty(&summary) {
                        ui.ctx().copy_text(json);
                        self.runtime_message = "Diagnostics summary copied (secrets redacted)".to_owned();
                    }
                }
                if compact_button_with_icon(ui, Icon::Download, "Export support bundle", self.theme).clicked() {
                    match self.export_support_bundle() {
                        Ok(path) => {
                            self.runtime_message = format!("Support bundle written to {path} (secrets redacted)");
                        }
                        Err(error) => {
                            self.runtime_message = format!("Support bundle export failed: {error}");
                        }
                    }
                }
            });
            ui.label(
                RichText::new("Passwords, tokens, and embedded URL credentials are redacted.")
                    .small()
                    .color(self.theme.text_muted),
            );
        });
    }

    fn draw_appearance_settings(&mut self, ui: &mut egui::Ui) {
        card_frame(self.theme).show(ui, |ui| {
            section_label(ui, "APPEARANCE", self.theme);
            ui.add_space(10.0);
            ui.horizontal_wrapped(|ui| {
                ui.label(icon_text(
                    if self.dark_mode { Icon::Moon } else { Icon::Sun },
                    "",
                    self.theme.accent,
                ));
                ui.selectable_value(&mut self.dark_mode, false, "Light");
                ui.selectable_value(&mut self.dark_mode, true, "Dark");
            });
            ui.label(
                RichText::new(
                    "Quiet surfaces, violet focus states, and high-contrast data. The choice is saved locally.",
                )
                .small()
                .color(self.theme.text_muted),
            );
            ui.checkbox(&mut self.reduce_motion, "Reduce motion");
            ui.label(
                RichText::new("Loading states keep a static status icon instead of a spinner.")
                    .small()
                    .color(self.theme.text_muted),
            );
        });
    }

    fn supports_backup_restore(&self) -> bool {
        self.active_capabilities()
            .allows(|capabilities| capabilities.features.backup)
    }

    fn draw_backup_settings(&mut self, ui: &mut egui::Ui) {
        ui.label(
            RichText::new("Backup destination")
                .small()
                .color(self.theme.text_secondary),
        );
        input_full_width(
            ui,
            &mut self.backup_output_path,
            "Choose a .sql backup path",
            self.theme,
        );
        ui.horizontal_wrapped(|ui| {
            if compact_button_with_icon(ui, Icon::FolderOpen, "Choose path", self.theme).clicked() {
                let request_id = self.task_bridge.next_request_id();
                self.dispatch_command(UiCommand::PickBackupFile { request_id });
            }
            if secondary_button_with_icon(ui, Icon::Archive, "Create backup", self.theme).clicked() {
                if let Some(connection) = self.active_connection().cloned() {
                    let request_id = self.task_bridge.next_request_id();
                    self.dispatch_command(UiCommand::Backup {
                        request_id,
                        connection_id: connection.id,
                        output_path: self.backup_output_path.clone(),
                        custom_format: false,
                    });
                }
            }
        });
    }

    fn draw_restore_settings(&mut self, ui: &mut egui::Ui) {
        ui.label(
            RichText::new("Restore from backup")
                .small()
                .color(self.theme.text_secondary),
        );
        input_full_width(ui, &mut self.restore_input_path, "Choose a backup file", self.theme);
        ui.horizontal_wrapped(|ui| {
            if compact_button_with_icon(ui, Icon::FolderOpen, "Choose file", self.theme).clicked() {
                let request_id = self.task_bridge.next_request_id();
                self.dispatch_command(UiCommand::PickRestoreFile { request_id });
            }
            if secondary_button_with_icon(ui, Icon::RotateCcw, "Restore database", self.theme).clicked() {
                self.restore_confirmation = true;
            }
        });
        if self.restore_confirmation {
            ui.add_space(10.0);
            ui.colored_label(self.theme.warning, "Overwrite the active database?");
            ui.horizontal(|ui| {
                if danger_button(ui, "Confirm restore", self.theme).clicked() {
                    if let Some(connection) = self.active_connection().cloned() {
                        let request_id = self.task_bridge.next_request_id();
                        self.dispatch_command(UiCommand::Restore {
                            request_id,
                            connection_id: connection.id,
                            input_path: self.restore_input_path.clone(),
                            custom_format: false,
                        });
                    }
                    self.restore_confirmation = false;
                }
                if ghost_button_with_icon(ui, Icon::X, "Cancel", self.theme).clicked() {
                    self.restore_confirmation = false;
                }
            });
        }
    }
}
