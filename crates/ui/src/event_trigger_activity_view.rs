use super::*;

impl DbProApp {
    pub(super) fn draw_event_trigger_activity(&mut self, ui: &mut egui::Ui) {
        ui.add_space(SPACE_MD);
        section_label(ui, "EVENT TRIGGERS", self.theme);
        ui.add_space(SPACE_SM);
        ui.label(
            RichText::new(
                "PostgreSQL-only · database-level DDL hooks · not table/row triggers · create requires existing function",
            )
            .small()
            .color(self.theme.text_muted),
        );
        if secondary_button_with_icon(ui, Icon::RefreshCw, "Load event triggers", self.theme).clicked() {
            self.request_event_triggers();
        }
        if let Some(error) = &self.management.event_trigger.event_trigger_error {
            ui.colored_label(self.theme.danger, error);
        }
        if let Some(inv) = self.management.event_trigger.event_trigger_inventory.clone() {
            ui.label(RichText::new(&inv.message).small().color(self.theme.text_secondary));
            for trig in inv.triggers.iter().take(50) {
                card_frame(self.theme).show(ui, |ui| {
                    ui.label(
                        RichText::new(format!(
                            "{} · on {} · {} · fn={}",
                            trig.name, trig.event, trig.enabled_label, trig.function_signature
                        ))
                        .strong()
                        .monospace(),
                    );
                    if !trig.tags.is_empty() {
                        ui.label(
                            RichText::new(format!("tags: {}", trig.tags.join(", ")))
                                .small()
                                .color(self.theme.text_muted),
                        );
                    }
                    ui.horizontal(|ui| {
                        if ghost_button_with_icon(ui, Icon::FileCode2, "Preview DROP", self.theme).clicked() {
                            // allow: preview is best-effort — preview generation error (name validation) only hides preview without blocking Drop
                            self.management.event_trigger.event_trigger_ddl_preview =
                                db_pro_core::domain::event_trigger::preview_drop_event_trigger(&trig.name).ok();
                        }
                        if ghost_button(ui, "Disable", self.theme).clicked() {
                            self.alter_event_trigger_confirmed(&trig.name, "disable");
                        }
                        if ghost_button(ui, "Enable", self.theme).clicked() {
                            self.alter_event_trigger_confirmed(&trig.name, "enable");
                        }
                        if danger_button(ui, "Drop…", self.theme).clicked() {
                            self.management.event_trigger.event_trigger_drop_confirm = Some(trig.name.clone());
                        }
                    });
                });
                ui.add_space(SPACE_XS);
            }
        }

        ui.add_space(SPACE_SM);
        ui.label(RichText::new("Create event trigger").small().strong());
        ui.horizontal(|ui| {
            ui.add(
                egui::TextEdit::singleline(&mut self.management.event_trigger.event_trigger_create_name)
                    .hint_text("name"),
            );
            ui.add(
                egui::TextEdit::singleline(&mut self.management.event_trigger.event_trigger_create_event)
                    .hint_text("event"),
            );
        });
        ui.horizontal(|ui| {
            ui.add(
                egui::TextEdit::singleline(&mut self.management.event_trigger.event_trigger_create_function)
                    .hint_text("schema.func()"),
            );
            ui.add(
                egui::TextEdit::singleline(&mut self.management.event_trigger.event_trigger_create_tags)
                    .hint_text("tags CSV optional"),
            );
        });
        ui.horizontal(|ui| {
            if ghost_button_with_icon(ui, Icon::FileCode2, "Preview CREATE", self.theme).clicked() {
                // allow: preview is best-effort — preview generation error (name validation) only hides preview without blocking Create
                self.management.event_trigger.event_trigger_ddl_preview =
                    db_pro_core::domain::event_trigger::preview_create_event_trigger(
                        &self.management.event_trigger.event_trigger_create_name,
                        &self.management.event_trigger.event_trigger_create_event,
                        &self.management.event_trigger.event_trigger_create_function,
                        &self.management.event_trigger.event_trigger_create_tags,
                    )
                    .ok();
            }
            if secondary_button(ui, "Create (confirm)", self.theme).clicked() {
                self.create_event_trigger_confirmed();
            }
        });

        if let Some(preview) = self.management.event_trigger.event_trigger_ddl_preview.clone() {
            egui::Window::new("Event trigger DDL preview")
                .collapsible(false)
                .resizable(true)
                .default_width(520.0)
                .show(ui.ctx(), |ui| {
                    ui.label(RichText::new(preview).monospace());
                    if secondary_button(ui, "Close", self.theme).clicked() {
                        self.management.event_trigger.event_trigger_ddl_preview = None;
                    }
                });
        }
        if let Some(name) = self.management.event_trigger.event_trigger_drop_confirm.clone() {
            egui::Window::new("Drop event trigger?")
                .collapsible(false)
                .resizable(false)
                .show(ui.ctx(), |ui| {
                    ui.label(format!(
                        "Drop event trigger `{name}`? This changes global DDL hook behavior."
                    ));
                    ui.horizontal(|ui| {
                        if danger_button(ui, "Drop", self.theme).clicked() {
                            self.drop_event_trigger_confirmed(&name);
                        }
                        if secondary_button(ui, "Cancel", self.theme).clicked() {
                            self.management.event_trigger.event_trigger_drop_confirm = None;
                        }
                    });
                });
        }
    }

    fn request_event_triggers(&mut self) {
        let Some(connection_id) = self.connection.lifecycle.active_connection_id().map(str::to_owned) else {
            self.management.event_trigger.event_trigger_error = Some("Connect a PostgreSQL database first".into());
            return;
        };
        if !self.active_driver().to_ascii_lowercase().contains("postgres") {
            self.management.event_trigger.event_trigger_error = Some("Event triggers are PostgreSQL-only".into());
            return;
        }
        let request_id = self.task_bridge.next_request_id();
        self.dispatch_command(self.management.event_trigger.list_command(request_id, connection_id));
    }

    fn create_event_trigger_confirmed(&mut self) {
        let Some(connection_id) = self.connection.lifecycle.active_connection_id().map(str::to_owned) else {
            return;
        };
        let request_id = self.task_bridge.next_request_id();
        self.dispatch_command(self.management.event_trigger.create_command(request_id, connection_id));
    }

    fn drop_event_trigger_confirmed(&mut self, name: &str) {
        let Some(connection_id) = self.connection.lifecycle.active_connection_id().map(str::to_owned) else {
            return;
        };
        let request_id = self.task_bridge.next_request_id();
        self.dispatch_command(
            self.management
                .event_trigger
                .drop_command(request_id, connection_id, name.to_owned()),
        );
    }

    fn alter_event_trigger_confirmed(&mut self, name: &str, mode: &str) {
        let Some(connection_id) = self.connection.lifecycle.active_connection_id().map(str::to_owned) else {
            return;
        };
        let request_id = self.task_bridge.next_request_id();
        self.dispatch_command(self.management.event_trigger.alter_command(
            request_id,
            connection_id,
            name.to_owned(),
            mode.to_owned(),
        ));
    }
}
