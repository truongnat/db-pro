use super::*;

impl DbProApp {
    pub(super) fn draw_transfers_activity(&mut self, ui: &mut egui::Ui) {
        section_label(ui, "TRANSFERS", self.theme);
        ui.add_space(SPACE_SM);
        ui.label(
            RichText::new("Streaming transfer engine · bounded batches · cancellable")
                .small()
                .color(self.theme.text_muted),
        );
        ui.add_space(SPACE_MD);
        card_frame(self.theme).show(ui, |ui| {
            section_label(ui, "BACKUP / RESTORE", self.theme);
            ui.label(
                RichText::new("Provider-aware backup lives in Settings → Backup / Restore (pg_dump/pg_restore or SQLite snapshot).")
                    .small()
                    .color(self.theme.text_secondary),
            );
            ui.add_space(6.0);
            if secondary_button_with_icon(ui, Icon::Archive, "Open Backup settings", self.theme).clicked() {
                self.workspace.activity = Activity::Settings;
                self.preferences.section = SettingsSection::Backup;
                self.workspace.sidebar_open = true;
            }
        });
        ui.add_space(SPACE_MD);
        card_frame(self.theme).show(ui, |ui| {
            section_label(ui, "SYNTHETIC TABLE SEED", self.theme);
            ui.label(
                RichText::new(
                    "Dev/test only · deterministic seed · preview before insert · Production requires confirmation",
                )
                .small()
                .color(self.theme.text_muted),
            );
            ui.add_space(SPACE_XS);
            let tables: Vec<(String, String)> = self
                .schema
                .explorer
                .schema
                .table_details
                .iter()
                .map(|t| (t.schema.clone(), t.name.clone()))
                .collect();
            ui.horizontal(|ui| {
                egui::ComboBox::from_id_salt("synth_table")
                    .selected_text(if self.management.synthetic_data.synthetic_table.is_empty() {
                        "Select table…"
                    } else {
                        &self.management.synthetic_data.synthetic_table
                    })
                    .show_ui(ui, |ui| {
                        for (schema, name) in &tables {
                            let key = if schema.is_empty() {
                                name.clone()
                            } else {
                                format!("{schema}.{name}")
                            };
                            ui.selectable_value(&mut self.management.synthetic_data.synthetic_table, key.clone(), key);
                        }
                    });
                ui.label("rows");
                ui.add(
                    egui::TextEdit::singleline(&mut self.management.synthetic_data.synthetic_row_count)
                        .desired_width(48.0),
                );
                ui.label("seed");
                ui.add(
                    egui::TextEdit::singleline(&mut self.management.synthetic_data.synthetic_seed).desired_width(64.0),
                );
                ui.label("null%");
                ui.add(
                    egui::TextEdit::singleline(&mut self.management.synthetic_data.synthetic_null_pct)
                        .desired_width(36.0),
                );
            });
            let is_production = self
                .active_connection()
                .map(|c| c.environment.eq_ignore_ascii_case("Production"))
                .unwrap_or(false);
            if is_production {
                ui.colored_label(
                    self.theme.warning,
                    "Production connection — confirm before applying INSERT SQL",
                );
                ui.checkbox(
                    &mut self.management.synthetic_data.synthetic_production_confirm,
                    "I confirm seeding this Production database",
                );
            }
            ui.horizontal(|ui| {
                if secondary_button(ui, "Preview", self.theme).clicked() {
                    self.preview_synthetic_seed();
                }
                if secondary_button(ui, "Export SQL → Query", self.theme).clicked() {
                    self.export_synthetic_seed_sql();
                }
                if primary_button(ui, "Apply INSERT (run)", self.theme).clicked() {
                    self.apply_synthetic_seed();
                }
            });
            if let Some(error) = &self.management.synthetic_data.synthetic_error {
                ui.colored_label(self.theme.danger, error);
            }
            if let Some(preview) = &self.management.synthetic_data.synthetic_preview {
                ui.label(RichText::new(&preview.message).small().color(self.theme.text_secondary));
                for (i, row) in preview.rows.iter().take(8).enumerate() {
                    ui.label(RichText::new(format!("#{i}: {}", row.join(" | "))).monospace().small());
                }
            }
        });
        ui.add_space(SPACE_MD);
        card_frame(self.theme).show(ui, |ui| {
            section_label(ui, "MASKING / SAFE SAMPLE", self.theme);
            ui.label(
                RichText::new(
                    "Preview-only by default · no in-place destructive masking · keyed hash for join-preserving IDs",
                )
                .small()
                .color(self.theme.text_muted),
            );
            ui.horizontal(|ui| {
                ui.add(
                    egui::TextEdit::singleline(&mut self.management.masking.masking_columns_csv)
                        .hint_text("cols: email,phone"),
                );
                egui::ComboBox::from_id_salt("mask_rule")
                    .selected_text(format!("{:?}", self.management.masking.masking_rule))
                    .show_ui(ui, |ui| {
                        ui.selectable_value(
                            &mut self.management.masking.masking_rule,
                            db_pro_core::domain::masking::MaskRule::Redact,
                            "Redact",
                        );
                        ui.selectable_value(
                            &mut self.management.masking.masking_rule,
                            db_pro_core::domain::masking::MaskRule::Hash,
                            "Hash",
                        );
                        ui.selectable_value(
                            &mut self.management.masking.masking_rule,
                            db_pro_core::domain::masking::MaskRule::PartialReveal,
                            "Partial",
                        );
                        ui.selectable_value(
                            &mut self.management.masking.masking_rule,
                            db_pro_core::domain::masking::MaskRule::Fixed,
                            "Fixed",
                        );
                        ui.selectable_value(
                            &mut self.management.masking.masking_rule,
                            db_pro_core::domain::masking::MaskRule::Synthetic,
                            "Synthetic",
                        );
                    });
                ui.checkbox(&mut self.management.masking.masking_keyed, "Keyed hash");
                if secondary_button(ui, "Suggest cols", self.theme).clicked() {
                    let names: Vec<String> = self
                        .schema
                        .explorer
                        .schema
                        .table_details
                        .first()
                        .map(|t| t.columns.iter().map(|c| c.name.clone()).collect())
                        .unwrap_or_default();
                    self.management.masking.masking_columns_csv =
                        db_pro_core::domain::masking::suggest_sensitive_columns(&names).join(",");
                }
                if secondary_button(ui, "Preview sample", self.theme).clicked() {
                    self.transfer_harness_context().preview_masking_sample();
                }
                if secondary_button(ui, "Masked CSV export", self.theme).clicked() {
                    self.transfer_harness_context().run_masked_csv_export_harness();
                }
            });
            if let Some(error) = &self.management.masking.masking_error {
                ui.colored_label(self.theme.danger, error);
            }
            if let Some(preview) = &self.management.masking.masking_preview {
                ui.label(RichText::new(&preview.message).small().color(self.theme.text_secondary));
                for (i, (orig, masked)) in preview.original.iter().zip(preview.masked.iter()).take(5).enumerate() {
                    ui.label(RichText::new(format!("#{i} {orig:?} → {masked:?}")).monospace().small());
                }
            }
        });
        ui.add_space(SPACE_MD);
        ui.horizontal_wrapped(|ui| {
            if primary_button_with_icon(ui, Icon::Play, "Run synthetic harness", self.theme).clicked() {
                self.transfer_harness_context().run_synthetic_transfer_harness(false);
            }
            if secondary_button_with_icon(ui, Icon::Ban, "Run then cancel", self.theme).clicked() {
                self.transfer_harness_context().run_synthetic_transfer_harness(true);
            }
            if secondary_button_with_icon(ui, Icon::FileSpreadsheet, "CSV export harness", self.theme).clicked() {
                self.transfer_harness_context().run_csv_export_harness();
            }
            if secondary_button_with_icon(ui, Icon::Download, "CSV import preview", self.theme).clicked() {
                self.transfer_harness_context().run_csv_import_preview_harness();
            }
            if secondary_button_with_icon(ui, Icon::Braces, "JSONL export harness", self.theme).clicked() {
                self.transfer_harness_context().run_jsonl_export_harness();
            }
            if secondary_button_with_icon(ui, Icon::Sheet, "Excel export harness", self.theme).clicked() {
                self.transfer_harness_context().run_excel_export_harness();
            }
            if secondary_button_with_icon(ui, Icon::DatabaseBackup, "DB→DB harness", self.theme).clicked() {
                self.transfer_harness_context().run_db_to_db_transfer_harness();
            }
            if ghost_button_with_icon(ui, Icon::Trash2, "Clear jobs", self.theme).clicked() {
                self.management.transfer.transfer_jobs.clear();
            }
        });
        ui.add_space(SPACE_MD);
        if self.management.transfer.transfer_jobs.is_empty() {
            empty_state(
                ui,
                Icon::Upload,
                "No transfers yet",
                "Run the synthetic harness to verify streaming progress, or import/export from Query once formats land.",
                self.theme,
            );
            return;
        }
        for job in &self.management.transfer.transfer_jobs {
            card_frame(self.theme).show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label(RichText::new(&job.label).strong().color(self.theme.text_primary));
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        badge(
                            ui,
                            match job.status {
                                db_pro_core::domain::transfer::TransferStatus::Pending => "pending",
                                db_pro_core::domain::transfer::TransferStatus::Running => "running",
                                db_pro_core::domain::transfer::TransferStatus::Succeeded => "succeeded",
                                db_pro_core::domain::transfer::TransferStatus::Failed => "failed",
                                db_pro_core::domain::transfer::TransferStatus::Cancelled => "cancelled",
                                db_pro_core::domain::transfer::TransferStatus::Partial => "partial",
                            },
                            self.theme.surface_active,
                            self.theme.text_secondary,
                        );
                    });
                });
                ui.label(
                    RichText::new(format!(
                        "read {} · wrote {} · {} bytes · committed {} · uncommitted {} · err_rows {} · {}",
                        job.progress.rows_read,
                        job.progress.rows_written,
                        job.progress.bytes_written,
                        job.progress.committed_batches,
                        job.progress.uncommitted_rows,
                        job.progress.error_rows,
                        job.progress.message
                    ))
                    .small()
                    .color(self.theme.text_secondary),
                );
                if let Some(error) = &job.error {
                    ui.colored_label(self.theme.warning, error);
                }
            });
            ui.add_space(SPACE_SM);
        }
    }
}

impl DbProApp {
    pub(crate) fn preview_synthetic_seed(&mut self) {
        self.transfer_harness_context().preview_synthetic_seed();
    }

    pub(crate) fn export_synthetic_seed_sql(&mut self) {
        match self.transfer_harness_context().build_synthetic_seed_sql() {
            Ok((sql, count)) => {
                self.set_active_query_text(sql);
                self.workspace.active_tab = WorkspaceTab::Query;
                self.management.synthetic_data.synthetic_error = None;
                self.feedback.runtime_message = format!("Synthetic INSERT SQL ({count} rows) exported to Query editor");
            }
            Err(error) => {
                self.management.synthetic_data.synthetic_error = Some(error);
            }
        }
    }

    pub(crate) fn apply_synthetic_seed(&mut self) {
        let is_production = self
            .active_connection()
            .map(|connection| connection.environment.eq_ignore_ascii_case("Production"))
            .unwrap_or(false);
        if is_production && !self.management.synthetic_data.synthetic_production_confirm {
            self.management.synthetic_data.synthetic_error =
                Some("Production confirmation required before applying seed INSERT".into());
            return;
        }
        self.export_synthetic_seed_sql();
        if self.management.synthetic_data.synthetic_error.is_some() {
            return;
        }
        if self.connection.lifecycle.active_connection_id().is_none() || !self.connection.lifecycle.is_connected() {
            self.management.synthetic_data.synthetic_error = Some("Connect to a database before applying seed".into());
            return;
        }
        self.dispatch_query();
        self.feedback.runtime_message = "Synthetic seed INSERT dispatched via query runtime".into();
    }

    fn transfer_harness_context(&mut self) -> transfer_harness_view::TransferHarnessContext<'_> {
        transfer_harness_view::TransferHarnessContext {
            management: &mut self.management,
            feedback: &mut self.feedback,
            table_details: &self.schema.explorer.schema.table_details,
        }
    }
}
