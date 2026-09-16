use super::*;
use egui::Sense;

impl DbProApp {
    pub(super) fn draw_topbar(&mut self, ctx: &egui::Context) {
        let connection_name = self.active_connection_name().to_owned();
        let driver = self.active_driver().to_owned();
        let has_connection = self.active_connection_id.is_some();
        let (connection_icon, connection_color) = self
            .active_connection()
            .map(|connection| self.connection_indicator(connection))
            .unwrap_or((Icon::Circle, self.theme.warning));
        let modifier = Self::primary_modifier_label();
        TopBottomPanel::top("topbar")
            .exact_height(38.0)
            .frame(egui::Frame {
                fill: self.theme.surface_app,
                inner_margin: egui::Margin::symmetric(SPACE_MD, 4.0),
                stroke: egui::Stroke::new(STROKE_THIN, self.theme.border_subtle),
                ..Default::default()
            })
            .show(ctx, |ui| {
                ui.set_min_size(ui.available_size());
                ui.horizontal_centered(|ui| {
                    // 0. Toggle Sidebar Button (Codex style)
                    let toggle_tooltip = if self.sidebar_open {
                        format!("Collapse Sidebar ({}B)", modifier)
                    } else {
                        format!("Expand Sidebar ({}B)", modifier)
                    };
                    let toggle_icon = if self.sidebar_open {
                        Icon::PanelLeftClose
                    } else {
                        Icon::PanelLeft
                    };
                    if Button::new(self.theme)
                        .icon(toggle_icon)
                        .variant(ButtonVariant::Ghost)
                        .size(ButtonSize::IconSm)
                        .tooltip(toggle_tooltip)
                        .show(ui)
                        .clicked()
                    {
                        self.sidebar_open = !self.sidebar_open;
                    }
                    ui.add_space(2.0);

                    // 1. History Navigation (Back / Forward)
                    let can_go_back = self.active_query_document > 0;
                    let can_go_forward = self.active_query_document + 1 < self.query_documents.len();
                    if Button::new(self.theme)
                        .icon(Icon::ArrowLeft)
                        .variant(ButtonVariant::Ghost)
                        .size(ButtonSize::IconSm)
                        .enabled(can_go_back)
                        .tooltip("Previous Document")
                        .show(ui)
                        .clicked()
                        && self.active_query_document > 0
                    {
                        self.switch_query_document(self.active_query_document - 1);
                    }
                    if Button::new(self.theme)
                        .icon(Icon::ArrowRight)
                        .variant(ButtonVariant::Ghost)
                        .size(ButtonSize::IconSm)
                        .enabled(can_go_forward)
                        .tooltip("Next Document")
                        .show(ui)
                        .clicked()
                        && self.active_query_document + 1 < self.query_documents.len()
                    {
                        self.switch_query_document(self.active_query_document + 1);
                    }

                    ui.add_space(SPACE_SM);
                    ui.label(RichText::new("│").font(font_caption()).color(self.theme.border_subtle));
                    ui.add_space(SPACE_SM);

                    // 2. Active Context Breadcrumb
                    if has_connection {
                        ui.horizontal(|ui| {
                            ui.label(icon_text(connection_icon, "", connection_color));
                            ui.label(
                                RichText::new(&connection_name)
                                    .font(font_caption())
                                    .strong()
                                    .color(self.theme.text_primary),
                            );
                            let driver_tag = if driver.to_ascii_lowercase().contains("sqlite") {
                                "SQLite"
                            } else {
                                "PostgreSQL"
                            };
                            egui::Frame {
                                fill: self.theme.surface_panel,
                                inner_margin: egui::Margin::symmetric(5.0, 1.0),
                                rounding: egui::Rounding::same(3.0),
                                stroke: egui::Stroke::new(1.0, self.theme.border_subtle),
                                ..Default::default()
                            }
                            .show(ui, |ui| {
                                ui.label(RichText::new(driver_tag).size(9.5).color(self.theme.text_secondary));
                            });
                        });
                    } else {
                        ui.horizontal(|ui| {
                            ui.label(icon_text(Icon::Database, "", self.theme.accent));
                            ui.label(
                                RichText::new("DB PRO")
                                    .font(font_caption())
                                    .strong()
                                    .color(self.theme.text_primary),
                            );
                        });
                    }

                    // 3. Right actions + Center Search Box
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        if Button::new(self.theme)
                            .icon(Icon::Command)
                            .variant(ButtonVariant::Ghost)
                            .size(ButtonSize::IconSm)
                            .tooltip(format!("Command Palette ({modifier}⇧P)"))
                            .show(ui)
                            .clicked()
                        {
                            self.open_palette(PaletteMode::Commands);
                        }
                        if Button::new(self.theme)
                            .icon(Icon::Palette)
                            .variant(ButtonVariant::Ghost)
                            .size(ButtonSize::IconSm)
                            .tooltip("Component Gallery (UI Design System)")
                            .show(ui)
                            .clicked()
                        {
                            self.active_tab = WorkspaceTab::ComponentGallery;
                        }
                        // No shortcut is bound to the agent panel, so the tooltip must
                        // not advertise one (it previously claimed a hardcoded ⌘I that
                        // existed on no platform and in no handler).
                        let agent_tooltip = if self.agent_open {
                            "Close Copilot Panel"
                        } else {
                            "Open Copilot Assistant"
                        };
                        if Button::new(self.theme)
                            .icon(Icon::Bot)
                            .variant(ButtonVariant::Ghost)
                            .size(ButtonSize::IconSm)
                            .tooltip(agent_tooltip)
                            .show(ui)
                            .clicked()
                        {
                            self.set_agent_open(!self.agent_open, ctx);
                        }
                        let theme_icon = if self.dark_mode { Icon::Sun } else { Icon::Moon };
                        let theme_tooltip = if self.dark_mode {
                            "Switch to Light Theme"
                        } else {
                            "Switch to Dark Theme"
                        };
                        if Button::new(self.theme)
                            .icon(theme_icon)
                            .variant(ButtonVariant::Ghost)
                            .size(ButtonSize::IconSm)
                            .tooltip(theme_tooltip)
                            .show(ui)
                            .clicked()
                        {
                            self.dark_mode = !self.dark_mode;
                        }

                        ui.add_space(SPACE_SM);

                        // 4. Center Command Search Box
                        let search_width = (ui.available_width() - 32.0).clamp(180.0, 360.0);
                        let (rect, resp) = ui.allocate_exact_size(egui::vec2(search_width, 26.0), Sense::click());
                        let hovered = resp.hovered();
                        let bg = if hovered {
                            self.theme.surface_hover
                        } else {
                            self.theme.surface_panel
                        };
                        let border = if hovered {
                            self.theme.border_strong
                        } else {
                            self.theme.border_subtle
                        };
                        ui.painter().rect(
                            rect,
                            egui::Rounding::same(RADIUS_MD),
                            bg,
                            egui::Stroke::new(1.0, border),
                        );

                        let search_icon_pos = egui::pos2(rect.left() + 8.0, rect.center().y);
                        ui.painter().text(
                            search_icon_pos,
                            egui::Align2::LEFT_CENTER,
                            char::from(Icon::Search).to_string(),
                            egui::FontId::new(12.0, egui::FontFamily::Name("lucide".into())),
                            self.theme.text_muted,
                        );

                        let text_pos = egui::pos2(rect.left() + 26.0, rect.center().y);
                        ui.painter().text(
                            text_pos,
                            egui::Align2::LEFT_CENTER,
                            "Search commands, tables, schemas...",
                            egui::FontId::proportional(11.5),
                            self.theme.text_muted,
                        );

                        let kbd_pos = egui::pos2(rect.right() - 8.0, rect.center().y);
                        let kbd_text = format!("{modifier}P");
                        ui.painter().text(
                            kbd_pos,
                            egui::Align2::RIGHT_CENTER,
                            &kbd_text,
                            egui::FontId::monospace(10.0),
                            self.theme.text_muted,
                        );

                        if resp.clicked() {
                            self.open_palette(PaletteMode::QuickOpen);
                        }
                    });
                });
            });
    }

    pub(super) fn draw_statusbar(&mut self, ctx: &egui::Context) {
        let (icon, color, label) = self.statusbar_state();
        let runtime_status = self.runtime_status();
        TopBottomPanel::bottom("statusbar")
            .exact_height(28.0)
            .frame(egui::Frame {
                fill: self.theme.surface_panel,
                inner_margin: egui::Margin::symmetric(SPACE_MD, SPACE_XXS),
                stroke: egui::Stroke::new(STROKE_THIN, self.theme.border_subtle),
                ..Default::default()
            })
            .show(ctx, |ui| {
                ui.set_min_size(ui.available_size());
                ui.horizontal_centered(|ui| {
                    ui.add_space(SPACE_SM);
                    ui.label(icon_text(icon, "", color));
                    ui.label(
                        RichText::new(label)
                            .font(font_caption())
                            .color(self.theme.text_secondary),
                    );
                    ui.separator();
                    if self.connected {
                        ui.label(
                            RichText::new(self.active_connection_name())
                                .font(font_caption())
                                .color(self.theme.text_secondary),
                        );
                    }
                    ui.label(
                        RichText::new(self.active_driver())
                            .font(font_caption())
                            .color(self.theme.text_muted),
                    );
                    if let Some(connection) = self.active_connection() {
                        ui.label(
                            RichText::new(&connection.database)
                                .font(font_caption())
                                .color(self.theme.text_muted),
                        );
                        ui.label(
                            RichText::new(self.active_schema())
                                .font(font_caption())
                                .color(self.theme.text_muted),
                        );
                    }
                    if let Some(result) = self.active_query_result().or(self.table_data_result.as_ref()) {
                        ui.label(
                            RichText::new(format!("{} ms", result.duration_ms))
                                .font(font_mono_sm())
                                .color(self.theme.text_muted),
                        );
                    }
                    if let Some((message, message_color)) = runtime_status {
                        ui.separator();
                        ui.add_sized(
                            [260.0, 18.0],
                            egui::Label::new(RichText::new(message).font(font_caption()).color(message_color)),
                        );
                    }
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        if compact_icon_button(ui, Icon::PanelBottom, self.theme)
                            .on_hover_text("Toggle output panel")
                            .clicked()
                        {
                            self.bottom_panel_open = !self.bottom_panel_open;
                        }
                        if self.shows_editor_status() {
                            ui.label(RichText::new("UTF-8").font(font_mono_sm()).color(self.theme.text_muted));
                            ui.label(
                                RichText::new(format!(
                                    "Ln {}, Col {}",
                                    self.query_cursor_line, self.query_cursor_column
                                ))
                                .font(font_mono_sm())
                                .color(self.theme.text_muted),
                            );
                        } else {
                            ui.label(
                                RichText::new(self.statusbar_context_label())
                                    .font(font_caption())
                                    .color(self.theme.text_muted),
                            );
                        }
                    });
                });
            });
    }

    pub(super) fn draw_output_panel(&mut self, ctx: &egui::Context) {
        if !self.bottom_panel_open {
            return;
        }
        let height = self.bottom_panel_height;
        let response = TopBottomPanel::bottom("output_panel")
            .resizable(true)
            .default_height(height)
            .height_range(OUTPUT_MIN_HEIGHT..=OUTPUT_MAX_HEIGHT)
            .frame(panel_frame(self.theme))
            .show(ctx, |ui| {
                ui.set_min_size(ui.available_size());
                ui.horizontal(|ui| {
                    section_label(ui, "OUTPUT", self.theme);
                    for (tab, label) in [
                        (OutputTab::Results, "Results"),
                        (OutputTab::Chart, "Chart"),
                        (OutputTab::Messages, "Messages"),
                        (OutputTab::Explain, "Explain"),
                        (OutputTab::History, "History"),
                    ] {
                        if tab_frame(self.theme, self.output_tab == tab)
                            .show(ui, |ui| ui.selectable_label(self.output_tab == tab, label))
                            .inner
                            .clicked()
                        {
                            self.output_tab = tab;
                        }
                    }
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        if compact_icon_button(ui, Icon::X, self.theme)
                            .on_hover_text("Close output")
                            .clicked()
                        {
                            self.bottom_panel_open = false;
                        }
                    });
                });
                ui.separator();
                match self.output_tab {
                    OutputTab::Results => {
                        let result = self.active_query_result().or(self.table_data_result.as_ref());
                        ui.label(
                            RichText::new(
                                result
                                    .map(|value| format!("{} rows · {} ms", value.row_count, value.duration_ms))
                                    .unwrap_or_else(|| "No result".to_owned()),
                            )
                            .small()
                            .color(self.theme.text_secondary),
                        );
                    }
                    OutputTab::Chart => {
                        ui.label(
                            RichText::new("Chart view — open the Chart tab for full controls")
                                .small()
                                .color(self.theme.text_muted),
                        );
                    }
                    OutputTab::Messages => {
                        for message in self.active_query_messages().iter().rev().take(8) {
                            ui.label(RichText::new(message).small().color(self.theme.text_secondary));
                        }
                    }
                    OutputTab::Explain => {
                        if let Some(plan) = self.active_explain_plan() {
                            egui::ScrollArea::vertical().show(ui, |ui| {
                                ui.label(RichText::new(plan).monospace().small().color(self.theme.text_secondary));
                            });
                        } else {
                            ui.label(
                                RichText::new("Run Explain to inspect the query plan")
                                    .small()
                                    .color(self.theme.text_muted),
                            );
                        }
                    }
                    OutputTab::History => {
                        for query in self.query_history.iter().rev().take(8) {
                            ui.label(
                                RichText::new(query)
                                    .monospace()
                                    .small()
                                    .color(self.theme.text_secondary),
                            );
                        }
                    }
                }
            });
        self.bottom_panel_height = response
            .response
            .rect
            .height()
            .clamp(OUTPUT_MIN_HEIGHT, OUTPUT_MAX_HEIGHT);
    }

    /// Transfers activity: streaming job list + synthetic harness (#193).
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
                self.activity = Activity::Settings;
                self.settings_section = SettingsSection::Backup;
                self.sidebar_open = true;
            }
        });
        ui.add_space(SPACE_MD);
        ui.horizontal_wrapped(|ui| {
            if primary_button_with_icon(ui, Icon::Play, "Run synthetic harness", self.theme).clicked() {
                self.run_synthetic_transfer_harness(false);
            }
            if secondary_button_with_icon(ui, Icon::Ban, "Run then cancel", self.theme).clicked() {
                self.run_synthetic_transfer_harness(true);
            }
            if secondary_button_with_icon(ui, Icon::FileSpreadsheet, "CSV export harness", self.theme).clicked() {
                self.run_csv_export_harness();
            }
            if secondary_button_with_icon(ui, Icon::Download, "CSV import preview", self.theme).clicked() {
                self.run_csv_import_preview_harness();
            }
            if secondary_button_with_icon(ui, Icon::Braces, "JSONL export harness", self.theme).clicked() {
                self.run_jsonl_export_harness();
            }
            if secondary_button_with_icon(ui, Icon::Sheet, "Excel export harness", self.theme).clicked() {
                self.run_excel_export_harness();
            }
            if secondary_button_with_icon(ui, Icon::DatabaseBackup, "DB→DB harness", self.theme).clicked() {
                self.run_db_to_db_transfer_harness();
            }
            if ghost_button_with_icon(ui, Icon::Trash2, "Clear jobs", self.theme).clicked() {
                self.transfer_jobs.clear();
            }
        });
        ui.add_space(SPACE_MD);
        if self.transfer_jobs.is_empty() {
            empty_state(
                ui,
                Icon::Upload,
                "No transfers yet",
                "Run the synthetic harness to verify streaming progress, or import/export from Query once formats land.",
                self.theme,
            );
            return;
        }
        for job in &self.transfer_jobs {
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

    pub(crate) fn run_synthetic_transfer_harness(&mut self, cancel_midway: bool) {
        use db_pro_core::application::TransferService;
        use db_pro_core::domain::transfer::{TransferCancellation, TransferJob, TransferStatus};

        let id = format!("xfer-{}", self.transfer_jobs.len() + 1);
        let mut job = TransferJob::new_synthetic(id, if cancel_midway { 20_000 } else { 5_000 }, 128);
        let cancel = TransferCancellation::new();
        if cancel_midway {
            // Prove cancel path: run a short first batch then cancel before continuing.
            use db_pro_core::application::{CountingTarget, SyntheticSource, TransferSource, TransferTarget};
            let mut source = SyntheticSource::new(20_000);
            let mut target = CountingTarget::default();
            job.status = TransferStatus::Running;
            if let Ok(Some(batch)) = source.next_batch(job.batch_size) {
                job.progress.rows_read += batch.len() as u64;
                if let Ok(written) = target.write_batch(&batch) {
                    job.progress.rows_written += written;
                    job.progress.bytes_written = target.bytes_written;
                }
            }
            cancel.cancel();
            let _ = TransferService::run(&mut job, &mut source, &mut target, &cancel);
            job.progress.bytes_written = target.bytes_written;
        } else {
            let _ = TransferService::run_synthetic(&mut job, &cancel);
        }
        self.runtime_message = format!(
            "Transfer {} · {:?} · wrote {}",
            job.id, job.status, job.progress.rows_written
        );
        self.transfer_jobs.insert(0, job);
        if self.transfer_jobs.len() > 20 {
            self.transfer_jobs.truncate(20);
        }
    }

    pub(crate) fn run_csv_export_harness(&mut self) {
        use db_pro_core::application::{DelimitedFileTarget, DelimitedFormat, SyntheticSource, TransferService};
        use db_pro_core::domain::transfer::{
            TransferCancellation, TransferJob, TransferSourceKind, TransferStatus, TransferTargetKind,
        };

        let mut path = std::env::temp_dir();
        path.push(format!("dbpro-export-{}.csv", self.transfer_jobs.len() + 1));
        let id = format!("csv-{}", self.transfer_jobs.len() + 1);
        let mut job = TransferJob {
            id,
            label: format!("CSV export → {}", path.display()),
            source: TransferSourceKind::Synthetic { rows: 1_000 },
            target: TransferTargetKind::File {
                path: path.to_string_lossy().into_owned(),
                format: "csv".into(),
            },
            mapping: Default::default(),
            batch_size: 100,
            options: Default::default(),
            status: TransferStatus::Pending,
            progress: Default::default(),
            error: None,
        };
        let mut source = SyntheticSource::new(1_000);
        let target = DelimitedFileTarget::create(&path, DelimitedFormat::Csv, vec!["id".into(), "name".into()]);
        match target {
            Ok(mut target) => {
                let cancel = TransferCancellation::new();
                let _ = TransferService::run(&mut job, &mut source, &mut target, &cancel);
            }
            Err(err) => {
                job.status = TransferStatus::Failed;
                job.error = Some(err.to_string());
            }
        }
        self.runtime_message = format!("CSV transfer {} · {:?} · {}", job.id, job.status, path.display());
        self.transfer_jobs.insert(0, job);
        if self.transfer_jobs.len() > 20 {
            self.transfer_jobs.truncate(20);
        }
    }

    pub(crate) fn run_csv_import_preview_harness(&mut self) {
        use db_pro_core::application::{DelimitedFileSource, DelimitedFileTarget, DelimitedFormat};
        use db_pro_core::domain::transfer::{TransferJob, TransferStatus, TransferTargetKind};

        let mut path = std::env::temp_dir();
        path.push("dbpro-import-sample.csv");
        let _ = DelimitedFileTarget::create(&path, DelimitedFormat::Csv, vec!["id".into(), "name".into()]).and_then(
            |mut target| {
                use db_pro_core::application::TransferTarget;
                use db_pro_core::domain::transfer::TransferRow;
                target.write_batch(&[
                    TransferRow {
                        cells: vec!["1".into(), "alpha".into()],
                    },
                    TransferRow {
                        cells: vec!["2".into(), "beta".into()],
                    },
                ])?;
                target.finish()
            },
        );

        let mut job = TransferJob::new_synthetic(format!("csv-import-{}", self.transfer_jobs.len() + 1), 0, 50);
        job.label = format!("CSV import preview ← {}", path.display());
        job.target = TransferTargetKind::File {
            path: path.to_string_lossy().into_owned(),
            format: "csv".into(),
        };
        match DelimitedFileSource::open(&path, DelimitedFormat::Csv, true) {
            Ok(mut source) => match source.preview_rows(20) {
                Ok(rows) => {
                    job.status = TransferStatus::Succeeded;
                    job.progress.rows_read = rows.len() as u64;
                    job.progress.message = format!("headers={:?}; preview {} row(s)", source.headers(), rows.len());
                }
                Err(err) => {
                    job.status = TransferStatus::Failed;
                    job.error = Some(err.to_string());
                }
            },
            Err(err) => {
                job.status = TransferStatus::Failed;
                job.error = Some(err.to_string());
            }
        }
        self.runtime_message = job.progress.message.clone();
        self.transfer_jobs.insert(0, job);
        if self.transfer_jobs.len() > 20 {
            self.transfer_jobs.truncate(20);
        }
    }

    pub(crate) fn run_jsonl_export_harness(&mut self) {
        use db_pro_core::application::{JsonlFileTarget, SyntheticSource, TransferService};
        use db_pro_core::domain::transfer::{TransferCancellation, TransferJob, TransferStatus, TransferTargetKind};

        let mut path = std::env::temp_dir();
        path.push(format!("dbpro-export-{}.jsonl", self.transfer_jobs.len() + 1));
        let mut job = TransferJob::new_synthetic(format!("jsonl-{}", self.transfer_jobs.len() + 1), 400, 50);
        job.label = format!("JSONL export → {}", path.display());
        job.target = TransferTargetKind::File {
            path: path.to_string_lossy().into_owned(),
            format: "jsonl".into(),
        };
        match JsonlFileTarget::create(&path, vec!["id".into(), "name".into()]) {
            Ok(mut target) => {
                let mut source = SyntheticSource::new(400);
                let cancel = TransferCancellation::new();
                let _ = TransferService::run(&mut job, &mut source, &mut target, &cancel);
            }
            Err(err) => {
                job.status = TransferStatus::Failed;
                job.error = Some(err.to_string());
            }
        }
        self.runtime_message = format!("JSONL {} · {:?}", job.id, job.status);
        self.transfer_jobs.insert(0, job);
        if self.transfer_jobs.len() > 20 {
            self.transfer_jobs.truncate(20);
        }
    }

    pub(crate) fn run_excel_export_harness(&mut self) {
        use db_pro_core::application::{ExcelFileTarget, SyntheticSource, TransferService};
        use db_pro_core::domain::transfer::{TransferCancellation, TransferJob, TransferStatus, TransferTargetKind};

        let mut path = std::env::temp_dir();
        path.push(format!("dbpro-export-{}.xlsx", self.transfer_jobs.len() + 1));
        let mut job = TransferJob::new_synthetic(format!("xlsx-{}", self.transfer_jobs.len() + 1), 80, 20);
        job.label = format!("Excel export → {}", path.display());
        job.target = TransferTargetKind::File {
            path: path.to_string_lossy().into_owned(),
            format: "xlsx".into(),
        };
        match ExcelFileTarget::create(&path, "Sheet1", vec!["id".into(), "name".into()]) {
            Ok(mut target) => {
                let mut source = SyntheticSource::new(80);
                let cancel = TransferCancellation::new();
                let _ = TransferService::run(&mut job, &mut source, &mut target, &cancel);
            }
            Err(err) => {
                job.status = TransferStatus::Failed;
                job.error = Some(err.to_string());
            }
        }
        self.runtime_message = format!("Excel {} · {:?}", job.id, job.status);
        self.transfer_jobs.insert(0, job);
        if self.transfer_jobs.len() > 20 {
            self.transfer_jobs.truncate(20);
        }
    }

    pub(crate) fn run_db_to_db_transfer_harness(&mut self) {
        use db_pro_core::application::{
            assert_endpoint_capabilities, build_conversion_plan, DbColumnSpec, GeneratorTableSource, MemoryTableTarget,
            TransferService,
        };
        use db_pro_core::domain::connection::DriverType;
        use db_pro_core::domain::transfer::{
            DbTableEndpoint, DbTransferOptions, TransferCancellation, TransferConflictPolicy, TransferJob,
            TransferMapping, TransferStatus, TransferTransactionPolicy,
        };

        let source_cols = vec![
            DbColumnSpec {
                name: "id".into(),
                type_name: "uuid".into(),
            },
            DbColumnSpec {
                name: "payload".into(),
                type_name: "jsonb".into(),
            },
        ];
        let target_cols = vec![
            DbColumnSpec {
                name: "id".into(),
                type_name: "text".into(),
            },
            DbColumnSpec {
                name: "payload".into(),
                type_name: "text".into(),
            },
        ];
        let plan = match build_conversion_plan(
            DriverType::Postgres,
            DriverType::SQLite,
            &source_cols,
            &target_cols,
            &TransferMapping::default(),
        ) {
            Ok(plan) => plan,
            Err(err) => {
                self.runtime_message = format!("DB→DB plan failed: {err}");
                return;
            }
        };
        if let Err(err) = plan.ensure_runnable() {
            self.runtime_message = format!("DB→DB blocked: {err}");
            return;
        }
        if let Err(err) = assert_endpoint_capabilities(true, true, false, false) {
            self.runtime_message = format!("DB→DB capability gate: {err}");
            return;
        }

        let mut job = TransferJob::new_db_table_copy(
            format!("dbdb-{}", self.transfer_jobs.len() + 1),
            DbTableEndpoint {
                connection_id: "src-conn".into(),
                schema: "public".into(),
                table: "events".into(),
            },
            DbTableEndpoint {
                connection_id: "dst-conn".into(),
                schema: "main".into(),
                table: "events".into(),
            },
            100,
            DbTransferOptions {
                conflict: TransferConflictPolicy::Skip,
                transaction: TransferTransactionPolicy::PerBatch,
                create_target_if_missing: false,
            },
        );
        job.label = format!(
            "DB→DB PG→SQLite preview ({} cols, {} warnings)",
            plan.columns.len(),
            plan.warnings.len()
        );
        let mut source = GeneratorTableSource::new(1_500);
        let mut target = MemoryTableTarget::new(TransferConflictPolicy::Skip, TransferTransactionPolicy::PerBatch);
        let cancel = TransferCancellation::new();
        let result = TransferService::run(&mut job, &mut source, &mut target, &cancel);
        job.progress.bytes_written = target.bytes_written;
        if result.status == TransferStatus::Succeeded || result.status == TransferStatus::Partial {
            self.runtime_message = format!(
                "DB→DB {} · {:?} · committed_batches={} · warnings={}",
                job.id,
                result.status,
                result.progress.committed_batches,
                plan.warnings.len()
            );
        } else {
            self.runtime_message = format!("DB→DB {} · {:?} · {:?}", job.id, result.status, job.error);
        }
        self.transfer_jobs.insert(0, job);
        if self.transfer_jobs.len() > 20 {
            self.transfer_jobs.truncate(20);
        }
    }

    /// Monitor activity: lightweight connection pulse from live app state.
    pub(super) fn draw_monitor_activity(&mut self, ui: &mut egui::Ui) {
        section_label(ui, "MONITOR", self.theme);
        ui.add_space(SPACE_SM);

        let connected = self.connected && self.active_connection_id.is_some();
        let driver = self.active_driver().to_owned();
        let name = self.active_connection_name().to_owned();

        egui::Frame {
            fill: self.theme.surface_elevated,
            inner_margin: egui::Margin::same(SPACE_MD),
            rounding: egui::Rounding::same(RADIUS_MD),
            stroke: egui::Stroke::new(STROKE_THIN, self.theme.border_subtle),
            ..Default::default()
        }
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                let (dot, label) = if connected {
                    (self.theme.success, "Connected")
                } else {
                    (self.theme.text_muted, "Disconnected")
                };
                status_dot(ui, dot, connected, false, self.theme);
                ui.label(RichText::new(label).strong().color(self.theme.text_primary));
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if connected && secondary_button_with_icon(ui, Icon::RefreshCw, "Refresh", self.theme).clicked() {
                        self.request_monitoring_snapshot();
                    }
                    ui.checkbox(&mut self.monitoring_poll, "Auto-refresh");
                });
            });
            ui.add_space(SPACE_SM);
            if connected {
                ui.label(
                    RichText::new(format!("{name} · {driver}"))
                        .small()
                        .color(self.theme.text_secondary),
                );
            } else {
                ui.label(
                    RichText::new("Connect from Explorer to monitor sessions.")
                        .small()
                        .color(self.theme.text_muted),
                );
            }
        });

        if connected && self.monitoring_poll {
            let due = self
                .monitoring_last_poll
                .map(|t| t.elapsed() >= std::time::Duration::from_secs(5))
                .unwrap_or(true);
            if due {
                self.request_monitoring_snapshot();
            }
        }

        ui.add_space(SPACE_MD);
        if let Some(error) = &self.monitoring_error {
            ui.colored_label(self.theme.warning, error);
            ui.add_space(SPACE_SM);
        }

        if let Some(snapshot) = self.monitoring_snapshot.clone() {
            if let Some(local) = &snapshot.local {
                section_label(ui, "LOCAL STATE", self.theme);
                ui.add_space(SPACE_SM);
                ui.label(RichText::new(&local.note).small().color(self.theme.text_muted));
                ui.label(format!(
                    "journal={} · pages={:?} · page_size={:?} · freelist={:?} · ~bytes={:?}",
                    local.journal_mode.as_deref().unwrap_or("?"),
                    local.page_count,
                    local.page_size,
                    local.freelist_count,
                    local.file_size_bytes
                ));
            }

            section_label(ui, "SESSIONS", self.theme);
            ui.add_space(SPACE_SM);
            ui.checkbox(&mut self.monitoring_filter_active_only, "Active queries only");
            ui.add_space(SPACE_SM);

            let sessions: Vec<_> = if self.monitoring_filter_active_only {
                snapshot.active_queries().into_iter().cloned().collect()
            } else {
                snapshot.sessions.clone()
            };

            if sessions.is_empty() {
                ui.label(
                    RichText::new(if snapshot.sessions.is_empty() {
                        snapshot.message.as_str()
                    } else {
                        "No active queries right now."
                    })
                    .small()
                    .color(self.theme.text_muted),
                );
            } else {
                for session in sessions {
                    card_frame(self.theme).show(ui, |ui| {
                        ui.horizontal(|ui| {
                            ui.label(
                                RichText::new(format!("pid {}", session.backend_id))
                                    .strong()
                                    .monospace()
                                    .color(self.theme.text_primary),
                            );
                            if session.is_current {
                                badge(ui, "current", self.theme.surface_active, self.theme.text_secondary);
                            }
                            ui.label(
                                RichText::new(session.state.clone().unwrap_or_else(|| "—".into()))
                                    .small()
                                    .color(self.theme.text_secondary),
                            );
                            if let Some(ms) = session.query_duration_ms {
                                ui.label(RichText::new(format!("{ms} ms")).small().color(self.theme.text_muted));
                            }
                        });
                        ui.label(
                            RichText::new(format!(
                                "{} · {} · {}",
                                session.username.as_deref().unwrap_or("?"),
                                session.database.as_deref().unwrap_or("?"),
                                session.application_name.as_deref().unwrap_or("-")
                            ))
                            .small()
                            .color(self.theme.text_secondary),
                        );
                        if let Some(query) = &session.query_text {
                            let short = if query.len() > 120 {
                                format!("{}…", &query.chars().take(119).collect::<String>())
                            } else {
                                query.clone()
                            };
                            ui.label(RichText::new(short).monospace().small().color(self.theme.text_primary));
                            ui.horizontal(|ui| {
                                if ghost_button_with_icon(ui, Icon::FileCode2, "Open SQL", self.theme).clicked() {
                                    self.set_active_query_text(query.clone());
                                    self.active_tab = WorkspaceTab::Query;
                                    self.activity = Activity::Explorer;
                                }
                                if !session.is_current
                                    && secondary_button_with_icon(ui, Icon::Ban, "Cancel", self.theme).clicked()
                                {
                                    if let Some(connection_id) = self.active_connection_id.clone() {
                                        let request_id = self.task_bridge.next_request_id();
                                        self.dispatch_command(UiCommand::MonitoringCancelBackend {
                                            request_id,
                                            connection_id,
                                            backend_id: session.backend_id,
                                        });
                                    }
                                }
                                if !session.is_current && danger_button(ui, "Terminate", self.theme).clicked() {
                                    self.monitoring_terminate_confirm = Some(session.backend_id);
                                }
                            });
                        }
                    });
                    ui.add_space(SPACE_SM);
                }
            }

            if let Some(server) = &snapshot.server {
                ui.add_space(SPACE_MD);
                section_label(ui, "SERVER", self.theme);
                ui.add_space(SPACE_SM);
                if let Some(version) = &server.version {
                    ui.label(RichText::new(version).small().color(self.theme.text_secondary));
                }
                ui.label(
                    RichText::new(format!(
                        "db={} · connections={:?}/{:?} · size={}",
                        server.current_database.as_deref().unwrap_or("?"),
                        server.current_connections,
                        server.max_connections,
                        server
                            .database_size_bytes
                            .map(db_pro_core::domain::monitoring::format_bytes_exact)
                            .unwrap_or_else(|| "—".into())
                    ))
                    .small()
                    .color(self.theme.text_muted),
                );
            }

            let blocking = snapshot.blocking_locks();
            if !blocking.is_empty() {
                ui.add_space(SPACE_MD);
                section_label(ui, "LOCKS / BLOCKERS", self.theme);
                ui.add_space(SPACE_SM);
                for lock in blocking.into_iter().take(30) {
                    ui.label(
                        RichText::new(format!(
                            "pid {} {} · blocker={:?} · {} · {}",
                            lock.locked_pid,
                            if lock.granted { "granted" } else { "waiting" },
                            lock.blocker_pid,
                            lock.mode.as_deref().unwrap_or("?"),
                            lock.relation.as_deref().unwrap_or("?")
                        ))
                        .small()
                        .monospace()
                        .color(self.theme.text_secondary),
                    );
                }
            }

            if !snapshot.relation_sizes.is_empty() {
                ui.add_space(SPACE_MD);
                section_label(ui, "SIZE / STATS", self.theme);
                ui.add_space(SPACE_SM);
                for rel in snapshot.relation_sizes.iter().take(20) {
                    ui.label(
                        RichText::new(format!(
                            "{}.{} ({}) · {} · seq={:?} idx={:?} dead={:?}",
                            rel.schema,
                            rel.name,
                            rel.kind,
                            db_pro_core::domain::monitoring::format_bytes_exact(rel.total_bytes),
                            rel.seq_scan,
                            rel.idx_scan,
                            rel.n_dead_tup
                        ))
                        .small()
                        .color(self.theme.text_secondary),
                    );
                }
            }

            ui.add_space(SPACE_MD);
            section_label(ui, "MAINTENANCE", self.theme);
            ui.add_space(SPACE_SM);
            ui.label(
                RichText::new("Actions run through MonitoringService — SQL is never built in the UI.")
                    .small()
                    .color(self.theme.text_muted),
            );
            ui.horizontal_wrapped(|ui| {
                use db_pro_core::domain::monitoring::MaintenanceAction;
                for action in [
                    MaintenanceAction::Analyze,
                    MaintenanceAction::Vacuum,
                    MaintenanceAction::VacuumAnalyze,
                ] {
                    if secondary_button(ui, action.as_label(), self.theme).clicked() {
                        self.monitoring_maintenance_confirm = Some(action);
                    }
                }
            });
        } else if connected {
            ui.label(
                RichText::new("Refresh to load sessions (or wait for auto-refresh).")
                    .small()
                    .color(self.theme.text_muted),
            );
        }

        if let Some(backend_id) = self.monitoring_terminate_confirm {
            egui::Window::new("Terminate session?")
                .collapsible(false)
                .resizable(false)
                .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
                .show(ui.ctx(), |ui| {
                    ui.label(format!(
                        "Terminate PostgreSQL backend pid {backend_id}? This disconnects the client."
                    ));
                    ui.horizontal(|ui| {
                        if danger_button(ui, "Terminate", self.theme).clicked() {
                            if let Some(connection_id) = self.active_connection_id.clone() {
                                let request_id = self.task_bridge.next_request_id();
                                self.dispatch_command(UiCommand::MonitoringTerminateBackend {
                                    request_id,
                                    connection_id,
                                    backend_id,
                                });
                            }
                            self.monitoring_terminate_confirm = None;
                        }
                        if secondary_button_with_icon(ui, Icon::X, "Cancel", self.theme).clicked() {
                            self.monitoring_terminate_confirm = None;
                        }
                    });
                });
        }

        if let Some(action) = self.monitoring_maintenance_confirm {
            egui::Window::new("Run maintenance?")
                .collapsible(false)
                .resizable(false)
                .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
                .show(ui.ctx(), |ui| {
                    ui.label(format!(
                        "Run {} on the active database? Long-running VACUUM can take locks.",
                        action.as_label()
                    ));
                    ui.horizontal(|ui| {
                        if danger_button(ui, action.as_label(), self.theme).clicked() {
                            if let Some(connection_id) = self.active_connection_id.clone() {
                                let request_id = self.task_bridge.next_request_id();
                                self.dispatch_command(UiCommand::MonitoringMaintenance {
                                    request_id,
                                    connection_id,
                                    schema: None,
                                    table: None,
                                    action,
                                    confirmed: true,
                                });
                            }
                            self.monitoring_maintenance_confirm = None;
                        }
                        if secondary_button_with_icon(ui, Icon::X, "Cancel", self.theme).clicked() {
                            self.monitoring_maintenance_confirm = None;
                        }
                    });
                });
        }
    }

    fn request_monitoring_snapshot(&mut self) {
        let Some(connection_id) = self.active_connection_id.clone() else {
            return;
        };
        self.monitoring_last_poll = Some(std::time::Instant::now());
        let request_id = self.task_bridge.next_request_id();
        self.dispatch_command(UiCommand::MonitoringSnapshot {
            request_id,
            connection_id,
        });
    }

    pub(super) fn draw_diagram_sidebar(&mut self, ui: &mut egui::Ui) {
        ui.vertical(|ui| {
            section_label(ui, "SCHEMA MAP", self.theme);
            ui.add_space(8.0);
            ui.label(RichText::new("Tables and foreign-key relationships").color(self.theme.text_primary));
            ui.add_space(4.0);
            ui.label(
                RichText::new(format!(
                    "{} · {}",
                    plural_count(self.schema.table_details.len(), "table", "tables"),
                    plural_count(
                        self.schema
                            .table_details
                            .iter()
                            .map(|table| table.foreign_keys.len())
                            .sum::<usize>(),
                        "relationship",
                        "relationships",
                    )
                ))
                .small()
                .color(self.theme.text_muted),
            );
        });
        ui.add_space(12.0);
        ui.separator();
        ui.add_space(12.0);
        ui.vertical(|ui| {
            section_label(ui, "NAVIGATION", self.theme);
            ui.add_space(8.0);
            ui.label(
                RichText::new(
                    "Drag the canvas to pan. Use the floating controls in the map to zoom or fit the schema.",
                )
                .small()
                .color(self.theme.text_secondary),
            );
            ui.add_space(10.0);
            if secondary_button_with_icon(ui, Icon::Database, "Back to Explorer", self.theme).clicked() {
                self.activity = Activity::Explorer;
                self.sidebar_open = true;
            }
        });
    }

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
            self.take_schema_snapshot();
        }
        if compact_button_with_icon(ui, Icon::GitCompare, "Diff vs snapshot", self.theme).clicked() {
            self.diff_against_schema_snapshot();
            self.active_tab = WorkspaceTab::SchemaCompare;
        }
        ui.add_space(8.0);
        if let Some(snap) = &self.schema_snapshot {
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
                    self.diff_against_schema_snapshot();
                }
                if secondary_button_with_icon(ui, Icon::Camera, "Snapshot", self.theme).clicked() {
                    self.take_schema_snapshot();
                }
            });
        });
        ui.add_space(SPACE_MD);
        let Some(diff) = self.schema_diff.clone() else {
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
        });
    }
}
