//! Settings activity sidebar panels (#205).
use super::*;
use crate::editor::PredictionMode;
use egui::RichText;
use lucide_icons::Icon;

impl DbProApp {
    pub(crate) fn apply_settings_to_runtime(&mut self) {
        self.dark_mode = self.settings.appearance.dark_mode;
        self.reduce_motion = self.settings.appearance.reduce_motion;
        self.editor_font_size = self.settings.editor.font_size;
        self.prediction_mode = match self.settings.editor.prediction_mode.as_str() {
            "off" => PredictionMode::Off,
            "subtle" => PredictionMode::Subtle,
            _ => PredictionMode::Eager,
        };
        self.agent_auto_run_read_only = self.settings.ai.auto_run_read_only;
        self.theme = if self.dark_mode {
            DbProTheme::dark()
        } else {
            DbProTheme::light()
        };
    }

    pub(crate) fn sync_settings_from_runtime(&mut self) {
        self.settings.version = settings_model::SETTINGS_VERSION;
        self.settings.appearance.dark_mode = self.dark_mode;
        self.settings.appearance.reduce_motion = self.reduce_motion;
        self.settings.editor.font_size = self.editor_font_size;
        self.settings.editor.prediction_mode = match self.prediction_mode {
            PredictionMode::Off => "off".to_owned(),
            PredictionMode::Subtle => "subtle".to_owned(),
            PredictionMode::Eager => "eager".to_owned(),
        };
        self.settings.ai.auto_run_read_only = self.agent_auto_run_read_only;
        self.settings.ai.provider_label = self.agent_provider_label.clone();
    }

    pub(super) fn draw_settings(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.vertical(|ui| {
                ui.set_width(128.0);
                for section in SettingsSection::all() {
                    let selected = self.settings_section == *section;
                    let label = RichText::new(section.label()).color(if selected {
                        self.theme.accent
                    } else {
                        self.theme.text_secondary
                    });
                    if ui.selectable_label(selected, label).clicked() {
                        self.settings_section = *section;
                    }
                }
            });
            ui.separator();
            ui.vertical(|ui| match self.settings_section {
                SettingsSection::General => self.draw_general_settings(ui),
                SettingsSection::Appearance => self.draw_appearance_settings(ui),
                SettingsSection::Editor => self.draw_editor_settings(ui),
                SettingsSection::DataGrid => self.draw_data_grid_settings(ui),
                SettingsSection::Connections => self.draw_connection_settings(ui),
                SettingsSection::Ai => self.draw_ai_settings(ui),
                SettingsSection::Keybindings => self.draw_keybindings_settings(ui),
                SettingsSection::Backup => self.draw_backup_section(ui),
                SettingsSection::Security => self.draw_security_settings(ui),
                SettingsSection::Advanced => self.draw_advanced_settings(ui),
            });
        });
        ui.add_space(12.0);
        self.draw_diagnostics_settings(ui);
    }

    fn draw_general_settings(&mut self, ui: &mut egui::Ui) {
        card_frame(self.theme).show(ui, |ui| {
            section_label(ui, "GENERAL", self.theme);
            ui.add_space(10.0);
            ui.checkbox(
                &mut self.settings.general.confirm_destructive_queries,
                "Confirm destructive queries",
            );
            ui.checkbox(
                &mut self.settings.general.restore_tabs_on_startup,
                "Restore query tabs on startup",
            );
            ui.add_space(12.0);
            section_label(ui, "WORKSPACE SESSIONS", self.theme);
            ui.add_space(6.0);
            ui.label(
                RichText::new(
                    "Named sessions store layout and tab references — not SQL text, secrets, or result grids.",
                )
                .small()
                .color(self.theme.text_muted),
            );
            input_full_width(ui, &mut self.session_name_draft, "Session name", self.theme);
            ui.horizontal(|ui| {
                if Button::new(self.theme)
                    .text("Save workspace")
                    .variant(ButtonVariant::Default)
                    .size(ButtonSize::Sm)
                    .show(ui)
                    .clicked()
                {
                    self.save_named_workspace_session();
                }
            });
            ui.add_space(SPACE_SM);
            let sessions = self.named_session_store.sessions.clone();
            for session in sessions {
                ui.horizontal(|ui| {
                    let selected = self.selected_named_session_id.as_deref() == Some(session.id.as_str());
                    if ui.selectable_label(selected, &session.name).clicked() {
                        self.selected_named_session_id = Some(session.id.clone());
                    }
                    if Button::new(self.theme)
                        .text("Restore")
                        .variant(ButtonVariant::Secondary)
                        .size(ButtonSize::Sm)
                        .show(ui)
                        .clicked()
                    {
                        self.restore_named_workspace_session(&session.id);
                    }
                    if Button::new(self.theme)
                        .text("Duplicate")
                        .variant(ButtonVariant::Secondary)
                        .size(ButtonSize::Sm)
                        .show(ui)
                        .clicked()
                    {
                        self.duplicate_named_workspace_session(&session.id);
                    }
                    if Button::new(self.theme)
                        .text("Delete")
                        .variant(ButtonVariant::Destructive)
                        .size(ButtonSize::Sm)
                        .show(ui)
                        .clicked()
                    {
                        self.named_session_store.remove(&session.id);
                    }
                });
            }
            if !self.last_session_restore_notes.is_empty() {
                ui.add_space(6.0);
                for note in &self.last_session_restore_notes {
                    ui.label(RichText::new(note).small().color(self.theme.warning));
                }
            }
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
                    self.settings.editor.font_size = self.editor_font_size;
                }
                ui.label(RichText::new(format!("{:.0} px", self.editor_font_size)).color(self.theme.text_primary));
                if compact_button(ui, "+", self.theme).clicked() {
                    self.editor_font_size = (self.editor_font_size + 1.0).min(24.0);
                    self.settings.editor.font_size = self.editor_font_size;
                }
            });
            ui.add_space(8.0);
            ui.horizontal(|ui| {
                ui.label(RichText::new("Tab width").color(self.theme.text_secondary));
                ui.add(egui::DragValue::new(&mut self.settings.editor.tab_width).range(2..=8));
            });
            ui.checkbox(&mut self.settings.editor.completion_enabled, "Schema completion");
            ui.checkbox(&mut self.settings.editor.format_on_save, "Format SQL on save");
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
                        self.settings.editor.prediction_mode = label.to_ascii_lowercase();
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
            ui.add_space(12.0);
            section_label(ui, "SQL LINT", self.theme);
            ui.add_space(6.0);
            ui.checkbox(&mut self.settings.editor.lint.enabled, "Enable SQL lint warnings");
            ui.add_enabled_ui(self.settings.editor.lint.enabled, |ui| {
                ui.checkbox(&mut self.settings.editor.lint.select_star, "Warn on SELECT *");
                ui.checkbox(&mut self.settings.editor.lint.null_compare, "Warn on = NULL / != NULL");
                ui.checkbox(
                    &mut self.settings.editor.lint.delete_no_where,
                    "Warn on DELETE without WHERE",
                );
                ui.checkbox(
                    &mut self.settings.editor.lint.update_no_where,
                    "Warn on UPDATE without WHERE",
                );
                ui.checkbox(
                    &mut self.settings.editor.lint.order_by_ordinal,
                    "Warn on ORDER BY ordinal",
                );
                ui.checkbox(
                    &mut self.settings.editor.lint.comma_join,
                    "Warn on comma / cartesian joins",
                );
                ui.checkbox(
                    &mut self.settings.editor.lint.duplicate_alias,
                    "Warn on duplicate projection aliases",
                );
            });
            ui.label(
                RichText::new("Lint warnings stay local and never call AI or the network.")
                    .small()
                    .color(self.theme.text_muted),
            );
        });
    }

    fn draw_data_grid_settings(&mut self, ui: &mut egui::Ui) {
        card_frame(self.theme).show(ui, |ui| {
            section_label(ui, "DATA GRID", self.theme);
            ui.add_space(10.0);
            ui.horizontal(|ui| {
                ui.label(RichText::new("Page size").color(self.theme.text_secondary));
                ui.add(egui::DragValue::new(&mut self.settings.data_grid.page_size).range(25..=1_000));
            });
            ui.checkbox(&mut self.settings.data_grid.show_row_numbers, "Show row numbers");
            ui.checkbox(&mut self.settings.data_grid.wrap_cell_text, "Wrap cell text");
        });
    }

    fn draw_connection_settings(&mut self, ui: &mut egui::Ui) {
        card_frame(self.theme).show(ui, |ui| {
            section_label(ui, "CONNECTIONS", self.theme);
            ui.add_space(10.0);
            ui.checkbox(
                &mut self.settings.connections.auto_connect_last,
                "Reconnect last connection on startup",
            );
            ui.checkbox(
                &mut self.settings.connections.default_ssl_prefer,
                "Prefer TLS for new server connections",
            );
            ui.label(
                RichText::new("Passwords stay in secret storage — never written to settings JSON.")
                    .small()
                    .color(self.theme.text_muted),
            );
        });
    }

    fn draw_ai_settings(&mut self, ui: &mut egui::Ui) {
        card_frame(self.theme).show(ui, |ui| {
            section_label(ui, "AI PROVIDERS", self.theme);
            ui.add_space(10.0);
            ui.checkbox(&mut self.settings.ai.enabled, "Enable Agent workspace");
            ui.checkbox(
                &mut self.settings.ai.auto_run_read_only,
                "Allow Agent to auto-run read-only queries",
            );
            self.agent_auto_run_read_only = self.settings.ai.auto_run_read_only;
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

    fn draw_keybindings_settings(&mut self, ui: &mut egui::Ui) {
        card_frame(self.theme).show(ui, |ui| {
            section_label(ui, "KEYBINDINGS", self.theme);
            ui.add_space(10.0);
            input_full_width(ui, &mut self.keybindings_filter, "Search commands…", self.theme);
            ui.add_space(6.0);
            ui.horizontal(|ui| {
                if compact_button(ui, "Reset all to defaults", self.theme).clicked() {
                    self.settings.keybindings.reset_all();
                    self.keybinding_edit_id = None;
                    self.runtime_message = "Keybindings reset to defaults".to_owned();
                }
            });
            let conflicts = self.settings.keybindings.conflict_ids();
            if !conflicts.is_empty() {
                ui.colored_label(
                    self.theme.warning,
                    format!("{} keybinding conflict(s) detected", conflicts.len()),
                );
            }
            ui.add_space(8.0);
            let filter = self.keybindings_filter.trim().to_ascii_lowercase();
            let catalog: Vec<_> = default_keybinding_catalog()
                .iter()
                .filter(|cmd| {
                    filter.is_empty() || cmd.title.to_ascii_lowercase().contains(&filter) || cmd.id.contains(&filter)
                })
                .copied()
                .collect();
            egui::ScrollArea::vertical().max_height(280.0).show(ui, |ui| {
                for cmd in catalog {
                    let resolved = self.settings.keybindings.resolved(cmd.id);
                    let is_conflict = conflicts.contains(cmd.id);
                    let editing = self.keybinding_edit_id.as_deref() == Some(cmd.id);
                    ui.horizontal(|ui| {
                        ui.label(RichText::new(cmd.title).color(if is_conflict {
                            self.theme.warning
                        } else {
                            self.theme.text_primary
                        }));
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if editing {
                                let response = ui.add(
                                    egui::TextEdit::singleline(&mut self.keybinding_edit_draft)
                                        .desired_width(120.0)
                                        .hint_text("mod+k"),
                                );
                                if (response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)))
                                    || compact_button(ui, "Save", self.theme).clicked()
                                {
                                    let draft = self.keybinding_edit_draft.trim().to_ascii_lowercase();
                                    if draft.is_empty() || draft == cmd.default_shortcut {
                                        self.settings.keybindings.reset_one(cmd.id);
                                    } else {
                                        self.settings.keybindings.overrides.insert(cmd.id.to_owned(), draft);
                                    }
                                    self.keybinding_edit_id = None;
                                }
                                if compact_button(ui, "Cancel", self.theme).clicked() {
                                    self.keybinding_edit_id = None;
                                }
                            } else {
                                if compact_button(ui, "Edit", self.theme).clicked() {
                                    self.keybinding_edit_id = Some(cmd.id.to_owned());
                                    self.keybinding_edit_draft = resolved.clone();
                                }
                                if self.settings.keybindings.overrides.contains_key(cmd.id)
                                    && compact_button(ui, "Reset", self.theme).clicked()
                                {
                                    self.settings.keybindings.reset_one(cmd.id);
                                }
                                ui.label(RichText::new(resolved).monospace().color(self.theme.text_secondary));
                            }
                        });
                    });
                    ui.add_space(4.0);
                }
            });
        });
    }

    fn draw_security_settings(&mut self, ui: &mut egui::Ui) {
        card_frame(self.theme).show(ui, |ui| {
            section_label(ui, "SECURITY", self.theme);
            ui.add_space(10.0);
            ui.checkbox(
                &mut self.settings.security.redact_secrets_in_logs,
                "Redact secrets in diagnostics and logs",
            );
            ui.checkbox(
                &mut self.settings.security.lock_secret_export,
                "Block secret export by default",
            );
        });
    }

    fn draw_advanced_settings(&mut self, ui: &mut egui::Ui) {
        card_frame(self.theme).show(ui, |ui| {
            section_label(ui, "ADVANCED", self.theme);
            ui.add_space(10.0);
            ui.checkbox(
                &mut self.settings.advanced.verbose_runtime_log,
                "Verbose runtime logging",
            );
            ui.checkbox(
                &mut self.settings.advanced.experimental_features,
                "Experimental features",
            );
            ui.label(
                RichText::new(format!("Settings schema version {}", self.settings.version))
                    .small()
                    .color(self.theme.text_muted),
            );
        });
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
            self.draw_backup_settings(ui);
            ui.add_space(14.0);
            self.draw_restore_settings(ui);
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
                if ui.selectable_value(&mut self.dark_mode, false, "Light").changed() {
                    self.settings.appearance.dark_mode = false;
                    self.theme = DbProTheme::light();
                }
                if ui.selectable_value(&mut self.dark_mode, true, "Dark").changed() {
                    self.settings.appearance.dark_mode = true;
                    self.theme = DbProTheme::dark();
                }
            });
            ui.label(
                RichText::new(
                    "Quiet surfaces, violet focus states, and high-contrast data. The choice is saved locally.",
                )
                .small()
                .color(self.theme.text_muted),
            );
            if ui.checkbox(&mut self.reduce_motion, "Reduce motion").changed() {
                self.settings.appearance.reduce_motion = self.reduce_motion;
            }
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
