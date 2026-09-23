use super::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum TransferActivityAction {
    OpenBackupSettings,
    PreviewSyntheticSeed,
    ExportSyntheticSeedSql,
    ApplySyntheticSeed,
    PreviewMasking,
    RunMaskedCsvExport,
    RunSyntheticTransfer { cancel_midway: bool },
    RunCsvExport,
    RunCsvImportPreview,
    RunJsonlExport,
    RunExcelExport,
    RunDbToDbTransfer,
    ClearJobs,
}

pub(super) struct TransferActivityContext<'a> {
    pub(super) theme: DbProTheme,
    pub(super) management: &'a mut DatabaseManagementState,
    pub(super) table_details: &'a [UiTableSummary],
    pub(super) is_production: bool,
}

impl TransferActivityContext<'_> {
    pub(super) fn draw(mut self, ui: &mut egui::Ui) -> Option<TransferActivityAction> {
        let mut action = None;
        self.draw_intro(ui);
        self.draw_backup_card(ui, &mut action);
        self.draw_synthetic_card(ui, &mut action);
        self.draw_masking_card(ui, &mut action);
        self.draw_harness_toolbar(ui, &mut action);
        self.draw_jobs(ui);
        action
    }

    fn draw_intro(&self, ui: &mut egui::Ui) {
        section_label(ui, "TRANSFERS", self.theme);
        ui.add_space(SPACE_SM);
        ui.label(
            RichText::new("Streaming transfer engine · bounded batches · cancellable")
                .small()
                .color(self.theme.text_muted),
        );
        ui.add_space(SPACE_MD);
    }

    fn draw_backup_card(&self, ui: &mut egui::Ui, action: &mut Option<TransferActivityAction>) {
        card_frame(self.theme).show(ui, |ui| {
            section_label(ui, "BACKUP / RESTORE", self.theme);
            ui.label(
                RichText::new(
                    "Provider-aware backup lives in Settings → Backup / Restore (pg_dump/pg_restore or SQLite snapshot).",
                )
                .small()
                .color(self.theme.text_secondary),
            );
            ui.add_space(6.0);
            if secondary_button_with_icon(ui, Icon::Archive, "Open Backup settings", self.theme).clicked() {
                *action = Some(TransferActivityAction::OpenBackupSettings);
            }
        });
        ui.add_space(SPACE_MD);
    }

    fn draw_synthetic_card(&mut self, ui: &mut egui::Ui, action: &mut Option<TransferActivityAction>) {
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
            self.draw_synthetic_inputs(ui);
            if self.is_production {
                ui.colored_label(
                    self.theme.warning,
                    "Production connection — confirm before applying INSERT SQL",
                );
                ui.checkbox(
                    &mut self.management.synthetic_data.synthetic_production_confirm,
                    "I confirm seeding this Production database",
                );
            }
            self.draw_synthetic_actions(ui, action);
            self.draw_synthetic_result(ui);
        });
        ui.add_space(SPACE_MD);
    }

    fn draw_synthetic_inputs(&mut self, ui: &mut egui::Ui) {
        let tables: Vec<(String, String)> = self
            .table_details
            .iter()
            .map(|table| (table.schema.clone(), table.name.clone()))
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
                egui::TextEdit::singleline(&mut self.management.synthetic_data.synthetic_row_count).desired_width(48.0),
            );
            ui.label("seed");
            ui.add(egui::TextEdit::singleline(&mut self.management.synthetic_data.synthetic_seed).desired_width(64.0));
            ui.label("null%");
            ui.add(
                egui::TextEdit::singleline(&mut self.management.synthetic_data.synthetic_null_pct).desired_width(36.0),
            );
        });
    }

    fn draw_synthetic_actions(&self, ui: &mut egui::Ui, action: &mut Option<TransferActivityAction>) {
        ui.horizontal(|ui| {
            if secondary_button(ui, "Preview", self.theme).clicked() {
                *action = Some(TransferActivityAction::PreviewSyntheticSeed);
            }
            if secondary_button(ui, "Export SQL → Query", self.theme).clicked() {
                *action = Some(TransferActivityAction::ExportSyntheticSeedSql);
            }
            if primary_button(ui, "Apply INSERT (run)", self.theme).clicked() {
                *action = Some(TransferActivityAction::ApplySyntheticSeed);
            }
        });
    }

    fn draw_synthetic_result(&self, ui: &mut egui::Ui) {
        if let Some(error) = &self.management.synthetic_data.synthetic_error {
            ui.colored_label(self.theme.danger, error);
        }
        if let Some(preview) = &self.management.synthetic_data.synthetic_preview {
            ui.label(RichText::new(&preview.message).small().color(self.theme.text_secondary));
            for (index, row) in preview.rows.iter().take(8).enumerate() {
                ui.label(
                    RichText::new(format!("#{index}: {}", row.join(" | ")))
                        .monospace()
                        .small(),
                );
            }
        }
    }

    fn draw_masking_card(&mut self, ui: &mut egui::Ui, action: &mut Option<TransferActivityAction>) {
        card_frame(self.theme).show(ui, |ui| {
            section_label(ui, "MASKING / SAFE SAMPLE", self.theme);
            ui.label(
                RichText::new(
                    "Preview-only by default · no in-place destructive masking · keyed hash for join-preserving IDs",
                )
                .small()
                .color(self.theme.text_muted),
            );
            self.draw_masking_controls(ui, action);
            self.draw_masking_result(ui);
        });
        ui.add_space(SPACE_MD);
    }

    fn draw_masking_controls(&mut self, ui: &mut egui::Ui, action: &mut Option<TransferActivityAction>) {
        ui.horizontal(|ui| {
            ui.add(
                egui::TextEdit::singleline(&mut self.management.masking.masking_columns_csv)
                    .hint_text("cols: email,phone"),
            );
            egui::ComboBox::from_id_salt("mask_rule")
                .selected_text(format!("{:?}", self.management.masking.masking_rule))
                .show_ui(ui, |ui| {
                    for (rule, label) in [
                        (db_pro_core::domain::masking::MaskRule::Redact, "Redact"),
                        (db_pro_core::domain::masking::MaskRule::Hash, "Hash"),
                        (db_pro_core::domain::masking::MaskRule::PartialReveal, "Partial"),
                        (db_pro_core::domain::masking::MaskRule::Fixed, "Fixed"),
                        (db_pro_core::domain::masking::MaskRule::Synthetic, "Synthetic"),
                    ] {
                        ui.selectable_value(&mut self.management.masking.masking_rule, rule, label);
                    }
                });
            ui.checkbox(&mut self.management.masking.masking_keyed, "Keyed hash");
            if secondary_button(ui, "Suggest cols", self.theme).clicked() {
                let names: Vec<String> = self
                    .table_details
                    .first()
                    .map(|table| table.columns.iter().map(|column| column.name.clone()).collect())
                    .unwrap_or_default();
                self.management.masking.masking_columns_csv =
                    db_pro_core::domain::masking::suggest_sensitive_columns(&names).join(",");
            }
            if secondary_button(ui, "Preview sample", self.theme).clicked() {
                *action = Some(TransferActivityAction::PreviewMasking);
            }
            if secondary_button(ui, "Masked CSV export", self.theme).clicked() {
                *action = Some(TransferActivityAction::RunMaskedCsvExport);
            }
        });
    }

    fn draw_masking_result(&self, ui: &mut egui::Ui) {
        if let Some(error) = &self.management.masking.masking_error {
            ui.colored_label(self.theme.danger, error);
        }
        if let Some(preview) = &self.management.masking.masking_preview {
            ui.label(RichText::new(&preview.message).small().color(self.theme.text_secondary));
            for (index, (original, masked)) in preview.original.iter().zip(preview.masked.iter()).take(5).enumerate() {
                ui.label(
                    RichText::new(format!("#{index} {original:?} → {masked:?}"))
                        .monospace()
                        .small(),
                );
            }
        }
    }

    fn draw_harness_toolbar(&self, ui: &mut egui::Ui, action: &mut Option<TransferActivityAction>) {
        ui.horizontal_wrapped(|ui| {
            if primary_button_with_icon(ui, Icon::Play, "Run synthetic harness", self.theme).clicked() {
                *action = Some(TransferActivityAction::RunSyntheticTransfer { cancel_midway: false });
            }
            if secondary_button_with_icon(ui, Icon::Ban, "Run then cancel", self.theme).clicked() {
                *action = Some(TransferActivityAction::RunSyntheticTransfer { cancel_midway: true });
            }
            if secondary_button_with_icon(ui, Icon::FileSpreadsheet, "CSV export harness", self.theme).clicked() {
                *action = Some(TransferActivityAction::RunCsvExport);
            }
            if secondary_button_with_icon(ui, Icon::Download, "CSV import preview", self.theme).clicked() {
                *action = Some(TransferActivityAction::RunCsvImportPreview);
            }
            if secondary_button_with_icon(ui, Icon::Braces, "JSONL export harness", self.theme).clicked() {
                *action = Some(TransferActivityAction::RunJsonlExport);
            }
            if secondary_button_with_icon(ui, Icon::Sheet, "Excel export harness", self.theme).clicked() {
                *action = Some(TransferActivityAction::RunExcelExport);
            }
            if secondary_button_with_icon(ui, Icon::DatabaseBackup, "DB→DB harness", self.theme).clicked() {
                *action = Some(TransferActivityAction::RunDbToDbTransfer);
            }
            if ghost_button_with_icon(ui, Icon::Trash2, "Clear jobs", self.theme).clicked() {
                *action = Some(TransferActivityAction::ClearJobs);
            }
        });
        ui.add_space(SPACE_MD);
    }

    fn draw_jobs(&self, ui: &mut egui::Ui) {
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
                            transfer_status_label(job.status),
                            self.theme.surface_active,
                            self.theme.text_secondary,
                        );
                    });
                });
                ui.label(
                    RichText::new(format_transfer_progress(job))
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

fn transfer_status_label(status: db_pro_core::domain::transfer::TransferStatus) -> &'static str {
    match status {
        db_pro_core::domain::transfer::TransferStatus::Pending => "pending",
        db_pro_core::domain::transfer::TransferStatus::Running => "running",
        db_pro_core::domain::transfer::TransferStatus::Succeeded => "succeeded",
        db_pro_core::domain::transfer::TransferStatus::Failed => "failed",
        db_pro_core::domain::transfer::TransferStatus::Cancelled => "cancelled",
        db_pro_core::domain::transfer::TransferStatus::Partial => "partial",
    }
}

fn format_transfer_progress(job: &db_pro_core::domain::transfer::TransferJob) -> String {
    format!(
        "read {} · wrote {} · {} bytes · committed {} · uncommitted {} · err_rows {} · {}",
        job.progress.rows_read,
        job.progress.rows_written,
        job.progress.bytes_written,
        job.progress.committed_batches,
        job.progress.uncommitted_rows,
        job.progress.error_rows,
        job.progress.message
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn action_enum_keeps_transfer_commands_explicit() {
        assert_eq!(
            TransferActivityAction::RunSyntheticTransfer { cancel_midway: true },
            TransferActivityAction::RunSyntheticTransfer { cancel_midway: true }
        );
    }
}
