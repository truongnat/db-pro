use super::*;

impl DbProApp {
    pub(super) fn draw_pg_settings_activity(&mut self, ui: &mut egui::Ui) {
        ui.add_space(SPACE_MD);
        section_label(ui, "SERVER SETTINGS (pg_settings)", self.theme);
        ui.add_space(SPACE_SM);
        ui.label(
            RichText::new("PostgreSQL-only · session SET/RESET for user-context GUCs · ALTER SYSTEM is preview-only")
                .small()
                .color(self.theme.text_muted),
        );
        ui.horizontal(|ui| {
            if secondary_button_with_icon(ui, Icon::RefreshCw, "Load settings", self.theme).clicked() {
                self.request_pg_settings();
            }
        });
        ui.add_space(SPACE_XS);
        ui.horizontal(|ui| {
            ui.label(RichText::new("Filter").small().color(self.theme.text_muted));
            ui.text_edit_singleline(&mut self.management.pg_settings.pg_settings_filter);
        });
        if let Some(error) = &self.management.pg_settings.pg_settings_error {
            ui.colored_label(self.theme.danger, error);
        }
        if let Some(snapshot) = self.management.pg_settings.pg_settings.clone() {
            ui.label(
                RichText::new(format!(
                    "{} · fetched @ {} ms",
                    snapshot.message, snapshot.fetched_at_ms
                ))
                .small()
                .color(self.theme.text_muted),
            );
            let filter = self.management.pg_settings.pg_settings_filter.to_ascii_lowercase();
            let rows: Vec<_> = snapshot
                .settings
                .iter()
                .filter(|s| {
                    filter.is_empty()
                        || s.name.to_ascii_lowercase().contains(&filter)
                        || s.category.to_ascii_lowercase().contains(&filter)
                        || s.source.to_ascii_lowercase().contains(&filter)
                })
                .take(60)
                .collect();
            for setting in rows {
                card_frame(self.theme).show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.label(
                            RichText::new(&setting.name)
                                .strong()
                                .monospace()
                                .color(self.theme.text_primary),
                        );
                        if setting.pending_restart {
                            badge(ui, "pending restart", self.theme.warning, self.theme.text_primary);
                        }
                        if setting.sensitive {
                            badge(ui, "redacted", self.theme.surface_active, self.theme.text_secondary);
                        }
                    });
                    ui.label(
                        RichText::new(format!(
                            "{} · context={} · source={} · {}",
                            setting.category,
                            setting.context,
                            setting.source,
                            setting.display_setting()
                        ))
                        .small()
                        .color(self.theme.text_secondary),
                    );
                    if let Some(desc) = &setting.short_desc {
                        ui.label(RichText::new(desc).small().color(self.theme.text_muted));
                    }
                    ui.label(
                        RichText::new(setting.mutability_reason())
                            .small()
                            .color(self.theme.text_muted),
                    );
                    ui.horizontal(|ui| {
                        if setting.session_mutable() && !setting.sensitive {
                            if ghost_button_with_icon(ui, Icon::Pencil, "Edit session", self.theme).clicked() {
                                self.management.pg_settings.pg_settings_edit_name = setting.name.clone();
                                self.management.pg_settings.pg_settings_edit_value = setting.setting.clone();
                            }
                            if secondary_button(ui, "RESET", self.theme).clicked() {
                                self.reset_pg_setting_session(&setting.name);
                            }
                        }
                        if !setting.sensitive
                            && ghost_button_with_icon(ui, Icon::FileCode2, "Preview ALTER SYSTEM", self.theme).clicked()
                        {
                            // allow: preview is best-effort — preview generation error (name validation) only hides preview without blocking ALTER SYSTEM
                            self.management.pg_settings.pg_settings_preview =
                                db_pro_core::domain::pg_settings::preview_alter_system(&setting.name, &setting.setting)
                                    .ok();
                        }
                    });
                });
                ui.add_space(SPACE_SM);
            }
        }

        if !self.management.pg_settings.pg_settings_edit_name.is_empty() {
            egui::Window::new(format!(
                "SET SESSION · {}",
                self.management.pg_settings.pg_settings_edit_name
            ))
            .collapsible(false)
            .resizable(false)
            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
            .show(ui.ctx(), |ui| {
                ui.text_edit_singleline(&mut self.management.pg_settings.pg_settings_edit_value);
                ui.horizontal(|ui| {
                    if secondary_button(ui, "Apply SET", self.theme).clicked() {
                        let name = self.management.pg_settings.pg_settings_edit_name.clone();
                        let value = self.management.pg_settings.pg_settings_edit_value.clone();
                        self.set_pg_setting_session(&name, &value);
                        self.management.pg_settings.pg_settings_edit_name.clear();
                    }
                    if ghost_button_with_icon(ui, Icon::X, "Cancel", self.theme).clicked() {
                        self.management.pg_settings.pg_settings_edit_name.clear();
                    }
                });
            });
        }

        if let Some(preview) = self.management.pg_settings.pg_settings_preview.clone() {
            egui::Window::new("ALTER SYSTEM preview")
                .collapsible(false)
                .resizable(true)
                .default_width(480.0)
                .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
                .show(ui.ctx(), |ui| {
                    ui.label(RichText::new(&preview.note).small().color(self.theme.warning));
                    ui.label(RichText::new(&preview.sql).monospace());
                    if secondary_button(ui, "Close", self.theme).clicked() {
                        self.management.pg_settings.pg_settings_preview = None;
                    }
                });
        }
    }

    fn request_pg_settings(&mut self) {
        let Some(connection_id) = self.connection.lifecycle.active_connection_id().map(str::to_owned) else {
            self.management.pg_settings.pg_settings_error = Some("Connect a PostgreSQL database first".into());
            return;
        };
        let driver = self.active_driver().to_ascii_lowercase();
        if !(driver.contains("postgres")) {
            self.management.pg_settings.pg_settings_error = Some("pg_settings is PostgreSQL-only".into());
            return;
        }
        let request_id = self.task_bridge.next_request_id();
        self.dispatch_command(self.management.pg_settings.list_command(request_id, connection_id));
    }

    fn set_pg_setting_session(&mut self, name: &str, value: &str) {
        let Some(connection_id) = self.connection.lifecycle.active_connection_id().map(str::to_owned) else {
            return;
        };
        let request_id = self.task_bridge.next_request_id();
        self.dispatch_command(self.management.pg_settings.set_session_command(
            request_id,
            connection_id,
            name.to_owned(),
            value.to_owned(),
        ));
    }

    fn reset_pg_setting_session(&mut self, name: &str) {
        let Some(connection_id) = self.connection.lifecycle.active_connection_id().map(str::to_owned) else {
            return;
        };
        let request_id = self.task_bridge.next_request_id();
        self.dispatch_command(self.management.pg_settings.reset_session_command(
            request_id,
            connection_id,
            name.to_owned(),
        ));
    }
}
