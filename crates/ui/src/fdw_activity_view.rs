use super::*;

impl DbProApp {
    pub(super) fn draw_fdw_activity(&mut self, ui: &mut egui::Ui) {
        ui.add_space(SPACE_MD);
        section_label(ui, "FOREIGN DATA (FDW)", self.theme);
        ui.add_space(SPACE_SM);
        ui.label(
            RichText::new("PostgreSQL-only · passwords/options redacted · CREATE EXTENSION never auto-run")
                .small()
                .color(self.theme.text_muted),
        );
        if secondary_button_with_icon(ui, Icon::RefreshCw, "Load FDW inventory", self.theme).clicked() {
            self.request_fdw_inventory();
        }
        if let Some(error) = &self.management.fdw.fdw_error {
            ui.colored_label(self.theme.danger, error);
        }
        if let Some(inv) = self.management.fdw.fdw_inventory.clone() {
            ui.label(RichText::new(&inv.message).small().color(self.theme.text_secondary));
            if let Some(hint) = &inv.extension_hint {
                ui.colored_label(self.theme.warning, hint);
            }
            for w in inv.wrappers.iter().take(20) {
                ui.label(
                    RichText::new(format!("wrapper {} · handler={:?}", w.name, w.handler))
                        .small()
                        .monospace(),
                );
            }
            for s in inv.servers.iter().take(30) {
                card_frame(self.theme).show(ui, |ui| {
                    ui.label(
                        RichText::new(format!("server {} · fdw={}", s.name, s.fdw_name))
                            .strong()
                            .monospace(),
                    );
                    let opts = s
                        .options
                        .iter()
                        .map(|o| format!("{}={}", o.key, o.display_value()))
                        .collect::<Vec<_>>()
                        .join(", ");
                    if !opts.is_empty() {
                        ui.label(RichText::new(opts).small().color(self.theme.text_muted));
                    }
                    ui.horizontal(|ui| {
                        if ghost_button_with_icon(ui, Icon::FileCode2, "Preview DROP", self.theme).clicked() {
                            // allow: preview is best-effort — preview generation error (name validation) only hides preview without blocking Drop
                            self.management.fdw.fdw_ddl_preview =
                                db_pro_core::domain::fdw::preview_drop_server(&s.name, true).ok();
                        }
                        if danger_button(ui, "Drop…", self.theme).clicked() {
                            self.management.fdw.fdw_drop_confirm = Some(s.name.clone());
                        }
                    });
                });
                ui.add_space(SPACE_XS);
            }
            for m in inv.user_mappings.iter().take(30) {
                let opts = m
                    .options
                    .iter()
                    .map(|o| format!("{}={}", o.key, o.display_value()))
                    .collect::<Vec<_>>()
                    .join(", ");
                ui.label(
                    RichText::new(format!("mapping {}@{} · {}", m.user_name, m.server_name, opts))
                        .small()
                        .monospace()
                        .color(self.theme.text_secondary),
                );
            }
            for t in inv.foreign_tables.iter().take(40) {
                ui.label(
                    RichText::new(format!("foreign {}.{} → {}", t.schema, t.name, t.server_name))
                        .small()
                        .monospace(),
                );
            }
        }

        ui.add_space(SPACE_SM);
        ui.label(RichText::new("Create foreign server").small().strong());
        ui.horizontal(|ui| {
            ui.add(egui::TextEdit::singleline(&mut self.management.fdw.fdw_create_name).hint_text("server name"));
            ui.add(egui::TextEdit::singleline(&mut self.management.fdw.fdw_create_wrapper).hint_text("fdw"));
        });
        ui.horizontal(|ui| {
            ui.add(egui::TextEdit::singleline(&mut self.management.fdw.fdw_create_host).hint_text("host"));
            ui.add(egui::TextEdit::singleline(&mut self.management.fdw.fdw_create_dbname).hint_text("dbname"));
            ui.add(egui::TextEdit::singleline(&mut self.management.fdw.fdw_create_port).hint_text("port"));
        });
        ui.horizontal(|ui| {
            if ghost_button_with_icon(ui, Icon::FileCode2, "Preview CREATE", self.theme).clicked() {
                // allow: preview is best-effort — preview generation error (name validation) only hides preview without blocking Create
                self.management.fdw.fdw_ddl_preview = db_pro_core::domain::fdw::preview_create_server(
                    &self.management.fdw.fdw_create_name,
                    &self.management.fdw.fdw_create_wrapper,
                    &self.management.fdw.fdw_create_host,
                    &self.management.fdw.fdw_create_dbname,
                    &self.management.fdw.fdw_create_port,
                )
                .ok();
            }
            if secondary_button(ui, "Create (confirm)", self.theme).clicked() {
                self.create_fdw_server_confirmed();
            }
        });

        if let Some(preview) = self.management.fdw.fdw_ddl_preview.clone() {
            egui::Window::new("FDW DDL preview")
                .collapsible(false)
                .resizable(true)
                .default_width(480.0)
                .show(ui.ctx(), |ui| {
                    ui.label(RichText::new(preview).monospace());
                    if secondary_button(ui, "Close", self.theme).clicked() {
                        self.management.fdw.fdw_ddl_preview = None;
                    }
                });
        }
        if let Some(name) = self.management.fdw.fdw_drop_confirm.clone() {
            egui::Window::new("Drop foreign server?")
                .collapsible(false)
                .resizable(false)
                .show(ui.ctx(), |ui| {
                    ui.label(format!(
                        "Drop server `{name}` CASCADE? This removes dependent foreign tables/mappings."
                    ));
                    ui.horizontal(|ui| {
                        if danger_button(ui, "Drop CASCADE", self.theme).clicked() {
                            self.drop_fdw_server_confirmed(&name, true);
                        }
                        if secondary_button(ui, "Cancel", self.theme).clicked() {
                            self.management.fdw.fdw_drop_confirm = None;
                        }
                    });
                });
        }
    }

    fn request_fdw_inventory(&mut self) {
        let Some(connection_id) = self.connection.lifecycle.active_connection_id().map(str::to_owned) else {
            self.management.fdw.fdw_error = Some("Connect a PostgreSQL database first".into());
            return;
        };
        if !self.active_driver().to_ascii_lowercase().contains("postgres") {
            self.management.fdw.fdw_error = Some("FDW administration is PostgreSQL-only".into());
            return;
        }
        let request_id = self.task_bridge.next_request_id();
        self.dispatch_command(self.management.fdw.list_command(request_id, connection_id));
    }

    fn create_fdw_server_confirmed(&mut self) {
        let Some(connection_id) = self.connection.lifecycle.active_connection_id().map(str::to_owned) else {
            return;
        };
        let request_id = self.task_bridge.next_request_id();
        self.dispatch_command(self.management.fdw.create_command(request_id, connection_id));
    }

    fn drop_fdw_server_confirmed(&mut self, name: &str, cascade: bool) {
        let Some(connection_id) = self.connection.lifecycle.active_connection_id().map(str::to_owned) else {
            return;
        };
        let request_id = self.task_bridge.next_request_id();
        self.dispatch_command(
            self.management
                .fdw
                .drop_command(request_id, connection_id, name.to_owned(), cascade),
        );
    }
}
