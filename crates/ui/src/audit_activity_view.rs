use super::*;

impl DbProApp {
    pub(super) fn draw_audit_activity(&mut self, ui: &mut egui::Ui) {
        ui.add_space(SPACE_MD);
        section_label(ui, "DATABASE AUDIT / ACTIVITY LOG", self.theme);
        ui.add_space(SPACE_SM);
        ui.label(
            RichText::new(
                "Database audit (server CSV log) — distinct from application Diagnostics. \
                 Configure tag audit:csvlog=/path/to/logfile.csv. Logging is never auto-enabled.",
            )
            .small()
            .color(self.theme.text_muted),
        );
        ui.horizontal(|ui| {
            if secondary_button_with_icon(ui, Icon::RefreshCw, "Load audit page", self.theme).clicked() {
                self.request_audit_page();
            }
            if secondary_button_with_icon(ui, Icon::Download, "Export selected", self.theme).clicked() {
                self.export_selected_audit_events();
            }
        });
        ui.horizontal_wrapped(|ui| {
            ui.label(RichText::new("Text").small().color(self.theme.text_muted));
            ui.add(
                egui::TextEdit::singleline(&mut self.audit.audit_filter_text)
                    .desired_width(120.0)
                    .hint_text("message/query"),
            );
            ui.label(RichText::new("DB").small().color(self.theme.text_muted));
            ui.add(egui::TextEdit::singleline(&mut self.audit.audit_filter_database).desired_width(80.0));
            ui.label(RichText::new("User").small().color(self.theme.text_muted));
            ui.add(egui::TextEdit::singleline(&mut self.audit.audit_filter_username).desired_width(80.0));
            ui.label(RichText::new("Severity").small().color(self.theme.text_muted));
            ui.add(egui::TextEdit::singleline(&mut self.audit.audit_filter_severity).desired_width(60.0));
        });
        if let Some(error) = &self.audit.audit_error {
            ui.colored_label(self.theme.danger, error);
        }
        if let Some(page) = self.audit.audit_page.clone() {
            ui.label(
                RichText::new(format!(
                    "{} · scanned {} bytes · truncated={}",
                    page.source.guidance, page.scanned_bytes, page.truncated
                ))
                .small()
                .color(self.theme.text_secondary),
            );
            if let Some(path) = &page.source.path {
                ui.label(RichText::new(format!("source: {path}")).small().monospace());
            }
            if page.source.kind == db_pro_core::domain::audit::AuditSourceKind::Unavailable {
                ui.colored_label(self.theme.warning, &page.source.guidance);
            }
            for event in page.events.iter().take(80) {
                let bookmarked = self.audit.audit_bookmarks.contains(&event.id);
                let selected = self.audit.audit_selected.contains(&event.id);
                card_frame(self.theme).show(ui, |ui| {
                    ui.horizontal(|ui| {
                        let mut sel = selected;
                        if ui.checkbox(&mut sel, "").changed() {
                            if sel {
                                self.audit.audit_selected.insert(event.id.clone());
                            } else {
                                self.audit.audit_selected.remove(&event.id);
                            }
                        }
                        ui.label(
                            RichText::new(format!(
                                "{} · {} · {} · {}",
                                event.timestamp.as_deref().unwrap_or("-"),
                                event.severity.as_deref().unwrap_or("-"),
                                event.username.as_deref().unwrap_or("-"),
                                event.database.as_deref().unwrap_or("-"),
                            ))
                            .small()
                            .strong(),
                        );
                        if event.redacted {
                            ui.colored_label(self.theme.warning, "redacted");
                        }
                        if bookmarked {
                            ui.colored_label(self.theme.accent, "★");
                        }
                    });
                    ui.label(RichText::new(&event.message).small().color(self.theme.text_primary));
                    if let Some(q) = &event.query {
                        let short = if q.len() > 160 {
                            format!("{}…", &q.chars().take(159).collect::<String>())
                        } else {
                            q.clone()
                        };
                        ui.label(RichText::new(short).monospace().small().color(self.theme.text_muted));
                        if ghost_button_with_icon(ui, Icon::FileCode2, "Open SQL", self.theme).clicked() {
                            self.set_active_query_text(q.clone());
                            self.workspace.active_tab = WorkspaceTab::Query;
                            self.workspace.activity = Activity::Explorer;
                        }
                    }
                    ui.horizontal(|ui| {
                        let label = if bookmarked { "Unbookmark" } else { "Bookmark" };
                        if ghost_button_with_icon(ui, Icon::Bookmark, label, self.theme).clicked() {
                            if bookmarked {
                                self.audit.audit_bookmarks.remove(&event.id);
                            } else {
                                self.audit.audit_bookmarks.insert(event.id.clone());
                            }
                        }
                    });
                });
                ui.add_space(SPACE_XS);
            }
            if let Some(preview) = &self.audit.audit_export_preview {
                ui.label(
                    RichText::new(page.export_warning.clone())
                        .small()
                        .color(self.theme.warning),
                );
                ui.label(
                    RichText::new(preview.chars().take(400).collect::<String>())
                        .monospace()
                        .small()
                        .color(self.theme.text_muted),
                );
            }
        }
    }

    fn request_audit_page(&mut self) {
        let Some(connection_id) = self.connection.lifecycle.active_connection_id().map(str::to_owned) else {
            self.audit.audit_error = Some("Connect a database first".into());
            return;
        };
        let request_id = self.task_bridge.next_request_id();
        self.dispatch_command(self.audit.events_load_command(request_id, connection_id));
    }

    fn export_selected_audit_events(&mut self) {
        match self.audit.build_export_preview() {
            Ok((selected_count, export_warning)) => {
                self.feedback.runtime_message =
                    format!("Audit export preview · {selected_count} row(s) · {export_warning}");
            }
            Err(error) => self.audit.audit_error = Some(error),
        }
    }
}
