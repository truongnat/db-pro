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
        // The sidebar chrome can hand children a max_rect slightly wider than
        // the painted card interior; galleys then wrap at the outer edge and
        // clip. Bound this surface to what is actually painted.
        let visible = (ui.available_width().min(ui.clip_rect().width()) - 6.0).max(1.0);
        ui.set_max_width(visible);

        self.draw_intro(ui);
        self.draw_backup_card(ui, &mut action);
        self.draw_synthetic_card(ui, &mut action);
        self.draw_masking_card(ui, &mut action);
        self.draw_copy_choices(ui);
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
                    // cc-scan:allow LINE_TOO_LONG — literal must not wrap
                    "Provider-aware backup lives in Settings → Backup / Restore (pg_dump/pg_restore or SQLite snapshot).",
                )
                .small()
                .color(self.theme.text_secondary),
            );
            ui.add_space(6.0);
            if Button::new(self.theme)
                .icon(Icon::Archive)
                .text("Open Backup settings")
                .variant(ButtonVariant::Secondary)
                .show(ui)
                .clicked()
            {
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
            // ui_rules: child clip = parent ∩ max_rect, so a galley that wraps
            // flush to available_width has its last glyph tips clipped by the
            // card's own edge. Allocate a sub-ui 6pt narrower for wrap-bound
            // text. (scope_builder/set_max_width cannot shrink below the
            // scroll content's minimum rect — only allocate_ui bounds it.)
            let text_w = (ui.available_width() - 6.0).max(40.0);
            let is_production = self.is_production;
            self.draw_synthetic_inputs(ui);
            let management = &mut *self.management;
            let block_h = ui.available_height();
            ui.allocate_ui_with_layout(
                egui::vec2(text_w, block_h),
                egui::Layout::top_down(egui::Align::Min),
                |ui| {
                    ui.add(
                        egui::Label::new(
                            RichText::new(if is_production {
                                "Production target — confirm before applying INSERT"
                            } else {
                                "Live target — confirm before applying INSERT"
                            })
                            .color(self.theme.warning),
                        )
                        .wrap(),
                    );
                    Checkbox::new(
                        &mut management.synthetic_data.synthetic_live_target_confirm,
                        "I confirm running INSERTs on this database",
                        self.theme,
                    )
                    .show(ui);
                    if is_production {
                        Checkbox::new(
                            &mut management.synthetic_data.synthetic_production_confirm,
                            "I confirm seeding this Production database",
                            self.theme,
                        )
                        .show(ui);
                    }
                },
            );

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
        // Narrow sidebar: horizontal_wrapped fused the label+input pairs into
        // an unreadable column ("10 seed 42 null% 0"). Stack each labeled row.
        ui.vertical(|ui| {
            let mut synth_options = vec!["Select table…".to_owned()];
            let synth_keys: Vec<String> = tables
                .iter()
                .map(|(schema, name)| {
                    if schema.is_empty() {
                        name.clone()
                    } else {
                        format!("{schema}.{name}")
                    }
                })
                .collect();
            synth_options.extend(synth_keys.iter().cloned());
            let mut synth_selected = synth_keys
                .iter()
                .position(|key| *key == self.management.synthetic_data.synthetic_table)
                .map(|index| index + 1)
                .unwrap_or(0);
            let synth_previous = synth_selected;
            crate::components::Select::new("synth_table", &mut synth_selected, &synth_options)
                .theme(self.theme)
                .width((ui.available_width() - 8.0).max(120.0))
                .size(crate::components::SelectSize::Sm)
                .variant(crate::components::SelectVariant::Ghost)
                .show(ui);
            if synth_selected != synth_previous {
                self.management.synthetic_data.synthetic_table = if synth_selected == 0 {
                    String::new()
                } else {
                    synth_keys[synth_selected - 1].clone()
                };
            }
            ui.add_space(SPACE_XS);
            // ui.horizontal() lets this fixed-size row overflow the card, which
            // grows the scroll area's max_rect and makes every following card
            // lay out wider than the visible clip. horizontal_wrapped wraps
            // the pair onto a second row instead of expanding.
            ui.horizontal_wrapped(|ui| {
                ui.spacing_mut().item_spacing = egui::vec2(SPACE_SM, SPACE_XS);
                ui.label(RichText::new("rows").small().color(self.theme.text_secondary));
                input(
                    ui,
                    &mut self.management.synthetic_data.synthetic_row_count,
                    "count",
                    48.0,
                    self.theme,
                );
                ui.label(RichText::new("seed").small().color(self.theme.text_secondary));
                input(
                    ui,
                    &mut self.management.synthetic_data.synthetic_seed,
                    "seed",
                    64.0,
                    self.theme,
                );
                ui.label(RichText::new("null%").small().color(self.theme.text_secondary));
                input(
                    ui,
                    &mut self.management.synthetic_data.synthetic_null_pct,
                    "%",
                    36.0,
                    self.theme,
                );
            });
        });
    }

    fn draw_synthetic_actions(&self, ui: &mut egui::Ui, action: &mut Option<TransferActivityAction>) {
        // Let the confirm checkbox own a row before the action cluster — on a
        // narrow sidebar horizontal_wrapped drew "Preview" over its label.
        ui.add_space(SPACE_XS);
        ui.horizontal_wrapped(|ui| {
            if Button::new(self.theme)
                .text("Preview")
                .variant(ButtonVariant::Secondary)
                .show(ui)
                .clicked()
            {
                *action = Some(TransferActivityAction::PreviewSyntheticSeed);
            }
            if Button::new(self.theme)
                .text("Export SQL → Query")
                .variant(ButtonVariant::Secondary)
                .show(ui)
                .clicked()
            {
                *action = Some(TransferActivityAction::ExportSyntheticSeedSql);
            }
            if Button::new(self.theme).text("Apply INSERT (run)").show(ui).clicked() {
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
            if std::mem::take(&mut self.management.transfer.capture_focus_once) {
                ui.scroll_to_cursor(Some(egui::Align::Min));
            }
            // ui_rules: child clip = parent ∩ max_rect, so a galley that wraps
            // flush to available_width has its last glyph tips clipped by the
            // card's own edge. Allocate a sub-ui 6pt narrower for the text.
            // (scope_builder grows max_rect to parent bounds — it cannot shrink.)
            let text_w = (ui.available_width() - 6.0).max(40.0);
            let h = ui.text_style_height(&egui::TextStyle::Body) * 2.0;
            ui.allocate_ui_with_layout(
                egui::vec2(text_w, h.max(ui.spacing().interact_size.y)),
                egui::Layout::top_down(egui::Align::Min),
                |ui| {
                    ui.add(
                        egui::Label::new(
                            RichText::new(
                                "Preview-only by default · no in-place destructive masking · keyed hash for join-preserving IDs",
                            )
                            .small()
                            .color(self.theme.text_muted),
                        )
                        .wrap(),
                    );
                },
            );
            self.draw_masking_controls(ui, action);
            self.draw_masking_result(ui);
        });
        ui.add_space(SPACE_MD);
    }

    fn draw_masking_controls(&mut self, ui: &mut egui::Ui, action: &mut Option<TransferActivityAction>) {
        const MASK_RULES: [(db_pro_core::domain::masking::MaskRule, &str); 5] = [
            (db_pro_core::domain::masking::MaskRule::Redact, "Redact"),
            (db_pro_core::domain::masking::MaskRule::Hash, "Hash"),
            (db_pro_core::domain::masking::MaskRule::PartialReveal, "Partial"),
            (db_pro_core::domain::masking::MaskRule::Fixed, "Fixed"),
            (db_pro_core::domain::masking::MaskRule::Synthetic, "Synthetic"),
        ];
        let mask_labels: Vec<String> = MASK_RULES.iter().map(|(_, label)| (*label).to_owned()).collect();
        let mut mask_selected = MASK_RULES
            .iter()
            .position(|(rule, _)| *rule == self.management.masking.masking_rule)
            .unwrap_or(0);
        let mask_previous = mask_selected;
        ui.horizontal_wrapped(|ui| {
            ui.spacing_mut().item_spacing = egui::vec2(SPACE_SM, SPACE_XS);
            // Bound each group before painting. Select otherwise consumes the
            // remaining row, so wrapping never gets a chance to start.
            let columns_width = f32::min(140.0, ui.max_rect().width());
            ui.allocate_ui_with_layout(
                egui::vec2(columns_width, crate::tokens::component::input::INPUT_HEIGHT_DEFAULT),
                egui::Layout::left_to_right(egui::Align::Center),
                |ui| {
                    input(
                        ui,
                        &mut self.management.masking.masking_columns_csv,
                        "cols: email,phone",
                        columns_width,
                        self.theme,
                    );
                },
            );
            let rule_width = f32::min(100.0, ui.max_rect().width());
            ui.allocate_ui_with_layout(
                egui::vec2(rule_width, crate::tokens::component::input::INPUT_HEIGHT_DEFAULT),
                egui::Layout::left_to_right(egui::Align::Center),
                |ui| {
                    crate::components::Select::new("mask_rule", &mut mask_selected, &mask_labels)
                        .theme(self.theme)
                        .width(rule_width)
                        .size(crate::components::SelectSize::Sm)
                        .variant(crate::components::SelectVariant::Ghost)
                        .show(ui);
                },
            );
            let keyed_width = f32::min(120.0, ui.max_rect().width());
            ui.allocate_ui_with_layout(
                egui::vec2(keyed_width, crate::tokens::component::input::INPUT_HEIGHT_DEFAULT),
                egui::Layout::left_to_right(egui::Align::Center),
                |ui| {
                    Checkbox::new(&mut self.management.masking.masking_keyed, "Keyed hash", self.theme).show(ui);
                },
            );
            if Button::new(self.theme)
                .text("Suggest cols")
                .variant(ButtonVariant::Secondary)
                .show(ui)
                .clicked()
            {
                let names: Vec<String> = self
                    .table_details
                    .first()
                    .map(|table| table.columns.iter().map(|column| column.name.clone()).collect())
                    .unwrap_or_default();
                self.management.masking.masking_columns_csv =
                    db_pro_core::domain::masking::suggest_sensitive_columns(&names).join(",");
            }
            if Button::new(self.theme)
                .text("Preview sample")
                .variant(ButtonVariant::Secondary)
                .show(ui)
                .clicked()
            {
                *action = Some(TransferActivityAction::PreviewMasking);
            }
            if Button::new(self.theme)
                .text("Demo masked CSV export")
                .variant(ButtonVariant::Secondary)
                .show(ui)
                .clicked()
            {
                *action = Some(TransferActivityAction::RunMaskedCsvExport);
            }
        });
        if mask_selected != mask_previous {
            self.management.masking.masking_rule = MASK_RULES[mask_selected].0;
        }
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

    fn draw_copy_choices(&mut self, ui: &mut egui::Ui) {
        ui.label(
            RichText::new("Source, target, tables, csv or json, abort / skip / replace")
                .small()
                .color(self.theme.text_muted),
        );
        let row_width = ui.available_width().min(ui.clip_rect().width()).max(1.0);
        ui.horizontal_wrapped(|ui| {
            ui.spacing_mut().item_spacing = egui::vec2(SPACE_SM, SPACE_XS);
            for (label, value, hint, field_width) in [
                (
                    "Source",
                    &mut self.management.transfer.source_connection_id,
                    "conn id",
                    160.0_f32,
                ),
                (
                    "Target",
                    &mut self.management.transfer.target_connection_id,
                    "conn id",
                    160.0_f32,
                ),
                (
                    "Tables",
                    &mut self.management.transfer.copy_tables,
                    "schema.table…",
                    160.0_f32,
                ),
                (
                    "Format",
                    &mut self.management.transfer.copy_format,
                    "csv or json",
                    120.0_f32,
                ),
                (
                    "Conflict",
                    &mut self.management.transfer.copy_conflict,
                    "abort / skip / replace",
                    140.0_f32,
                ),
            ] {
                // Request the group's real width. Shrinking it to the leftover
                // gap keeps the next field on this row, then the input paints
                // past the sidebar clip.
                let field = field_width.min((row_width - 56.0).max(72.0));
                let group = (field + 56.0).min(row_width);
                ui.allocate_ui_with_layout(
                    egui::vec2(group, crate::tokens::component::input::INPUT_HEIGHT_DEFAULT),
                    egui::Layout::left_to_right(egui::Align::Center),
                    |ui| {
                        ui.set_max_width(group);
                        ui.label(label);
                        Input::new(value, hint, self.theme).width(field).show(ui);
                    },
                );
            }
        });
        ui.add_space(SPACE_SM);
    }

    fn draw_harness_toolbar(&self, ui: &mut egui::Ui, action: &mut Option<TransferActivityAction>) {
        ui.horizontal_wrapped(|ui| {
            if Button::new(self.theme)
                .icon(Icon::Play)
                .text("Run synthetic harness")
                .show(ui)
                .clicked()
            {
                *action = Some(TransferActivityAction::RunSyntheticTransfer { cancel_midway: false });
            }
            if Button::new(self.theme)
                .icon(Icon::Ban)
                .text("Run then cancel")
                .variant(ButtonVariant::Secondary)
                .show(ui)
                .clicked()
            {
                *action = Some(TransferActivityAction::RunSyntheticTransfer { cancel_midway: true });
            }
            if Button::new(self.theme)
                .icon(Icon::FileSpreadsheet)
                .text("CSV export harness")
                .variant(ButtonVariant::Secondary)
                .show(ui)
                .clicked()
            {
                *action = Some(TransferActivityAction::RunCsvExport);
            }
            if Button::new(self.theme)
                .icon(Icon::Download)
                .text("CSV import preview")
                .variant(ButtonVariant::Secondary)
                .show(ui)
                .clicked()
            {
                *action = Some(TransferActivityAction::RunCsvImportPreview);
            }
            if Button::new(self.theme)
                .icon(Icon::Braces)
                .text("JSONL export harness")
                .variant(ButtonVariant::Secondary)
                .show(ui)
                .clicked()
            {
                *action = Some(TransferActivityAction::RunJsonlExport);
            }
            if Button::new(self.theme)
                .icon(Icon::Sheet)
                .text("Excel export harness")
                .variant(ButtonVariant::Secondary)
                .show(ui)
                .clicked()
            {
                *action = Some(TransferActivityAction::RunExcelExport);
            }
            if Button::new(self.theme)
                .icon(Icon::DatabaseBackup)
                .text("Copy tables")
                .variant(ButtonVariant::Secondary)
                .show(ui)
                .clicked()
            {
                *action = Some(TransferActivityAction::RunDbToDbTransfer);
            }
            if Button::new(self.theme)
                .icon(Icon::Trash2)
                .text("Clear jobs")
                .variant(ButtonVariant::Ghost)
                .show(ui)
                .clicked()
            {
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
                // cc-scan:allow LINE_TOO_LONG — literal must not wrap
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
#[path = "tests/transfer_activity_surface_view_tests.rs"]
mod tests;
