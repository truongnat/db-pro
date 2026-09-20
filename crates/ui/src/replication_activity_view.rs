use super::*;

impl DbProApp {
    pub(super) fn draw_replication_activity(&mut self, ui: &mut egui::Ui) {
        ui.add_space(SPACE_MD);
        section_label(ui, "LOGICAL REPLICATION", self.theme);
        ui.add_space(SPACE_SM);
        ui.label(
            RichText::new(
                "PostgreSQL-only · subscription conninfo redacted · CREATE SUBSCRIPTION not offered (secrets)",
            )
            .small()
            .color(self.theme.text_muted),
        );
        if secondary_button_with_icon(ui, Icon::RefreshCw, "Load replication inventory", self.theme).clicked() {
            self.request_replication_inventory();
        }
        if let Some(error) = &self.management.replication.replication_error {
            ui.colored_label(self.theme.danger, error);
        }
        if let Some(inv) = self.management.replication.replication_inventory.clone() {
            ui.label(RichText::new(&inv.message).small().color(self.theme.text_secondary));
            for pub_info in inv.publications.iter().take(40) {
                card_frame(self.theme).show(ui, |ui| {
                    ui.label(
                        RichText::new(format!(
                            "publication {} · all_tables={} · owner={:?}",
                            pub_info.name, pub_info.all_tables, pub_info.owner
                        ))
                        .strong()
                        .monospace(),
                    );
                    if !pub_info.tables.is_empty() {
                        ui.label(
                            RichText::new(format!("tables: {}", pub_info.tables.join(", ")))
                                .small()
                                .color(self.theme.text_muted),
                        );
                    }
                    ui.horizontal(|ui| {
                        if ghost_button_with_icon(ui, Icon::FileCode2, "Preview DROP", self.theme).clicked() {
                            // allow: preview is best-effort — preview generation error (name validation) only hides preview without blocking Drop
                            self.management.replication.replication_ddl_preview =
                                db_pro_core::domain::replication::preview_drop_publication(&pub_info.name).ok();
                        }
                        if danger_button(ui, "Drop…", self.theme).clicked() {
                            self.management.replication.replication_drop_publication = Some(pub_info.name.clone());
                        }
                    });
                });
                ui.add_space(SPACE_XS);
            }
            for sub in inv.subscriptions.iter().take(40) {
                card_frame(self.theme).show(ui, |ui| {
                    ui.label(
                        RichText::new(format!(
                            "subscription {} · enabled={} · slot={:?}",
                            sub.name, sub.enabled, sub.slot_name
                        ))
                        .strong()
                        .monospace(),
                    );
                    ui.label(
                        RichText::new(format!(
                            "pubs={} · conninfo={}",
                            sub.publications.join(","),
                            sub.conninfo_redacted
                        ))
                        .small()
                        .color(self.theme.text_muted),
                    );
                    ui.horizontal(|ui| {
                        if ghost_button_with_icon(ui, Icon::FileCode2, "Preview DROP", self.theme).clicked() {
                            // allow: preview is best-effort — preview generation error (name validation) only hides preview without blocking Drop
                            self.management.replication.replication_ddl_preview =
                                db_pro_core::domain::replication::preview_drop_subscription(&sub.name).ok();
                        }
                        if danger_button(ui, "Drop…", self.theme).clicked() {
                            self.management.replication.replication_drop_subscription = Some(sub.name.clone());
                        }
                    });
                });
                ui.add_space(SPACE_XS);
            }
            for slot in inv.slots.iter().take(40) {
                ui.label(
                    RichText::new(format!(
                        "slot {} · type={:?} · active={} · restart={:?}",
                        slot.slot_name, slot.slot_type, slot.active, slot.restart_lsn
                    ))
                    .small()
                    .monospace()
                    .color(self.theme.text_secondary),
                );
            }
        }

        ui.add_space(SPACE_SM);
        ui.label(RichText::new("Create publication (FOR ALL TABLES)").small().strong());
        ui.horizontal(|ui| {
            ui.add(
                egui::TextEdit::singleline(&mut self.management.replication.replication_create_name)
                    .hint_text("publication name"),
            );
            if ghost_button_with_icon(ui, Icon::FileCode2, "Preview CREATE", self.theme).clicked() {
                // allow: preview is best-effort — preview generation error (name validation) only hides preview without blocking Create
                self.management.replication.replication_ddl_preview =
                    db_pro_core::domain::replication::preview_create_publication_all(
                        &self.management.replication.replication_create_name,
                    )
                    .ok();
            }
            if secondary_button(ui, "Create (confirm)", self.theme).clicked() {
                self.create_publication_confirmed();
            }
        });

        if let Some(preview) = self.management.replication.replication_ddl_preview.clone() {
            egui::Window::new("Replication DDL preview")
                .collapsible(false)
                .resizable(true)
                .default_width(480.0)
                .show(ui.ctx(), |ui| {
                    ui.label(RichText::new(preview).monospace());
                    if secondary_button(ui, "Close", self.theme).clicked() {
                        self.management.replication.replication_ddl_preview = None;
                    }
                });
        }
        if let Some(name) = self.management.replication.replication_drop_publication.clone() {
            egui::Window::new("Drop publication?")
                .collapsible(false)
                .resizable(false)
                .show(ui.ctx(), |ui| {
                    ui.label(format!("Drop publication `{name}`?"));
                    ui.horizontal(|ui| {
                        if danger_button(ui, "Drop", self.theme).clicked() {
                            self.drop_publication_confirmed(&name);
                        }
                        if secondary_button(ui, "Cancel", self.theme).clicked() {
                            self.management.replication.replication_drop_publication = None;
                        }
                    });
                });
        }
        if let Some(name) = self.management.replication.replication_drop_subscription.clone() {
            egui::Window::new("Drop subscription?")
                .collapsible(false)
                .resizable(false)
                .show(ui.ctx(), |ui| {
                    ui.label(format!(
                        "Drop subscription `{name}`? Conninfo is never shown or logged."
                    ));
                    ui.horizontal(|ui| {
                        if danger_button(ui, "Drop", self.theme).clicked() {
                            self.drop_subscription_confirmed(&name);
                        }
                        if secondary_button(ui, "Cancel", self.theme).clicked() {
                            self.management.replication.replication_drop_subscription = None;
                        }
                    });
                });
        }
    }

    fn request_replication_inventory(&mut self) {
        let Some(connection_id) = self.connection.lifecycle.active_connection_id().map(str::to_owned) else {
            self.management.replication.replication_error = Some("Connect a PostgreSQL database first".into());
            return;
        };
        if !self.active_driver().to_ascii_lowercase().contains("postgres") {
            self.management.replication.replication_error =
                Some("Logical replication administration is PostgreSQL-only".into());
            return;
        }
        let request_id = self.task_bridge.next_request_id();
        self.dispatch_command(self.management.replication.list_command(request_id, connection_id));
    }

    fn create_publication_confirmed(&mut self) {
        let Some(connection_id) = self.connection.lifecycle.active_connection_id().map(str::to_owned) else {
            return;
        };
        let request_id = self.task_bridge.next_request_id();
        self.dispatch_command(
            self.management
                .replication
                .create_publication_command(request_id, connection_id),
        );
    }

    fn drop_publication_confirmed(&mut self, name: &str) {
        let Some(connection_id) = self.connection.lifecycle.active_connection_id().map(str::to_owned) else {
            return;
        };
        let request_id = self.task_bridge.next_request_id();
        self.dispatch_command(self.management.replication.drop_publication_command(
            request_id,
            connection_id,
            name.to_owned(),
        ));
    }

    fn drop_subscription_confirmed(&mut self, name: &str) {
        let Some(connection_id) = self.connection.lifecycle.active_connection_id().map(str::to_owned) else {
            return;
        };
        let request_id = self.task_bridge.next_request_id();
        self.dispatch_command(self.management.replication.drop_subscription_command(
            request_id,
            connection_id,
            name.to_owned(),
        ));
    }
}
