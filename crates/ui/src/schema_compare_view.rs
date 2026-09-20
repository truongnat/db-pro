use super::*;

impl DbProApp {
    pub(super) fn draw_schema_compare_sidebar(&mut self, ui: &mut egui::Ui) {
        section_label(ui, "SCHEMA COMPARE", self.theme);
        ui.add_space(6.0);
        ui.label(
            RichText::new("Snapshot the loaded schema, then re-introspect and diff.")
                .small()
                .color(self.theme.text_muted),
        );
        ui.add_space(8.0);
        if compact_button_with_icon(ui, Icon::Camera, "Take snapshot", self.theme).clicked() {
            let connection_name = self.active_connection_name().to_owned();
            self.schema
                .compare
                .take_snapshot(&self.schema.explorer.schema, &connection_name, &mut self.feedback);
        }
        if compact_button_with_icon(ui, Icon::GitCompare, "Diff vs snapshot", self.theme).clicked() {
            self.schema
                .compare
                .diff_against_snapshot(&self.schema.explorer.schema, &mut self.feedback);
            self.workspace.active_tab = WorkspaceTab::SchemaCompare;
        }
        ui.add_space(8.0);
        if let Some(snap) = &self.schema.compare.schema_snapshot {
            ui.label(
                RichText::new(format!("Snapshot: {}", snap.label))
                    .small()
                    .color(self.theme.text_secondary),
            );
            ui.label(
                RichText::new(format!("{} tables", snap.tables.len()))
                    .small()
                    .color(self.theme.text_muted),
            );
        } else {
            ui.label(RichText::new("No snapshot yet.").small().color(self.theme.text_muted));
        }
    }

    pub(super) fn draw_schema_compare(&mut self, ui: &mut egui::Ui) {
        ui.set_min_width(ui.available_width());
        ui.add_space(SPACE_SM);
        ui.horizontal(|ui| {
            ui.label(
                RichText::new("Schema Compare")
                    .font(font_subheading())
                    .strong()
                    .color(self.theme.text_primary),
            );
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if secondary_button_with_icon(ui, Icon::GitCompare, "Diff now", self.theme).clicked() {
                    self.schema
                        .compare
                        .diff_against_snapshot(&self.schema.explorer.schema, &mut self.feedback);
                }
                if secondary_button_with_icon(ui, Icon::Camera, "Snapshot", self.theme).clicked() {
                    let connection_name = self.active_connection_name().to_owned();
                    self.schema.compare.take_snapshot(
                        &self.schema.explorer.schema,
                        &connection_name,
                        &mut self.feedback,
                    );
                }
            });
        });
        ui.add_space(SPACE_MD);
        let Some(diff) = self.schema.compare.schema_diff.clone() else {
            card_frame(self.theme).show(ui, |ui| {
                ui.set_min_width((ui.available_width() - 8.0).max(0.0));
                empty_state(
                    ui,
                    Icon::GitCompare,
                    "No schema diff yet",
                    "Take a snapshot, change or refresh the schema, then Diff now.",
                    self.theme,
                );
            });
            return;
        };
        egui::ScrollArea::vertical().show(ui, |ui| {
            for (title, items) in [
                ("Tables only in snapshot", &diff.tables_only_in_source),
                ("Tables only in current", &diff.tables_only_in_target),
                ("Views only in snapshot", &diff.views_only_in_source),
                ("Views only in current", &diff.views_only_in_target),
                ("Routines only in snapshot", &diff.functions_only_in_source),
                ("Routines only in current", &diff.functions_only_in_target),
                ("Column / type changes", &diff.column_changes),
            ] {
                ui.label(RichText::new(title).strong());
                if items.is_empty() {
                    ui.label(RichText::new("— none —").small().color(self.theme.text_muted));
                } else {
                    for item in items {
                        ui.label(RichText::new(format!("• {item}")).small());
                    }
                }
                ui.add_space(8.0);
            }
            ui.separator();
            ui.add_space(8.0);
            section_label(ui, "MIGRATION PLAN", self.theme);
            if primary_button_with_icon(ui, Icon::FileCode2, "Generate migration plan", self.theme).clicked() {
                let driver = self.active_driver().to_owned();
                self.schema.compare.plan_migration(&driver, &mut self.feedback);
            }
            if let Some(plan) = &self.schema.compare.migration_plan {
                ui.label(
                    RichText::new(format!(
                        "{} ops · fingerprint {} · destructive={}",
                        plan.operations.len(),
                        plan.fingerprint,
                        plan.has_destructive
                    ))
                    .small()
                    .monospace()
                    .color(self.theme.text_secondary),
                );
                for warning in &plan.warnings {
                    ui.label(RichText::new(format!("⚠ {warning}")).small().color(self.theme.warning));
                }
                for op in &plan.operations {
                    let risk = match op.risk {
                        db_pro_core::domain::migration::MigrationRisk::Destructive => "DESTRUCTIVE",
                        db_pro_core::domain::migration::MigrationRisk::Mutating => "mutating",
                        db_pro_core::domain::migration::MigrationRisk::Safe => "safe",
                    };
                    ui.label(
                        RichText::new(format!("{} · {} · {}", op.id, risk, op.sql))
                            .small()
                            .monospace()
                            .color(self.theme.text_secondary),
                    );
                }
            }
            if !self.schema.compare.migration_preview_sql.is_empty() {
                ui.label(
                    RichText::new(&self.schema.compare.migration_preview_sql)
                        .small()
                        .monospace()
                        .color(self.theme.text_primary),
                );
                if self
                    .schema
                    .compare
                    .migration_plan
                    .as_ref()
                    .is_some_and(|p| p.has_destructive)
                {
                    ui.checkbox(
                        &mut self.schema.compare.migration_confirm_destructive,
                        "Confirm destructive operations (never auto-applied)",
                    );
                }
                if primary_button_with_icon(ui, Icon::Play, "Apply migration SQL", self.theme).clicked() {
                    self.apply_migration_preview();
                }
            }
            ui.separator();
            ui.add_space(8.0);
            section_label(ui, "DATA COMPARE", self.theme);
            ui.label(
                RichText::new("Key-aware sample compare across two active connections. Sync SQL is preview-only.")
                    .small()
                    .color(self.theme.text_muted),
            );
            input_full_width(
                ui,
                &mut self.schema.compare.data_diff_target_id,
                "target connection id",
                self.theme,
            );
            input_full_width(ui, &mut self.schema.compare.data_diff_schema, "schema", self.theme);
            input_full_width(ui, &mut self.schema.compare.data_diff_table, "table", self.theme);
            input_full_width(
                ui,
                &mut self.schema.compare.data_diff_keys,
                "key columns (comma)",
                self.theme,
            );
            if primary_button_with_icon(ui, Icon::GitCompare, "Compare rows", self.theme).clicked() {
                self.request_data_diff_keyed();
            }
            if let Some(diff) = &self.schema.compare.data_diff_result {
                ui.label(
                    RichText::new(format!(
                        "counts src={} tgt={} · +{} -{} ~{} ={} · truncated={}",
                        diff.source_row_count,
                        diff.target_row_count,
                        diff.added,
                        diff.removed,
                        diff.changed,
                        diff.equal,
                        diff.truncated
                    ))
                    .small()
                    .monospace()
                    .color(self.theme.text_secondary),
                );
                ui.horizontal(|ui| {
                    for label in ["all", "added", "removed", "changed"] {
                        if ui
                            .selectable_label(self.schema.compare.data_diff_filter == label, label)
                            .clicked()
                        {
                            self.schema.compare.data_diff_filter = label.to_owned();
                        }
                    }
                });
                for row in &diff.row_diffs {
                    let include = match self.schema.compare.data_diff_filter.as_str() {
                        "added" => row.state == db_pro_core::domain::cross_connection::DataRowState::Added,
                        "removed" => row.state == db_pro_core::domain::cross_connection::DataRowState::Removed,
                        "changed" => row.state == db_pro_core::domain::cross_connection::DataRowState::Changed,
                        _ => true,
                    };
                    if !include {
                        continue;
                    }
                    ui.label(
                        RichText::new(format!("{:?} · {}", row.state, row.key.replace('\u{1f}', "|")))
                            .small()
                            .monospace()
                            .color(self.theme.text_secondary),
                    );
                }
                for sql in &diff.sync_sql_preview {
                    ui.label(RichText::new(sql).small().monospace().color(self.theme.text_muted));
                }
            }
        });
    }

    pub(crate) fn request_data_diff_keyed(&mut self) {
        let Some(source_id) = self.connection.lifecycle.active_connection_id().map(str::to_owned) else {
            self.feedback.runtime_message = "Connect a source database first".into();
            return;
        };
        let request_id = self.task_bridge.next_request_id();
        match self.schema.compare.build_data_diff_request(request_id, source_id) {
            Ok(command) => {
                self.dispatch_command(command);
                self.feedback.runtime_message = "Running key-aware data compare…".into();
            }
            Err(error) => self.feedback.runtime_message = error,
        }
    }
}
