use super::*;
use egui::Sense;

impl DbProApp {
    pub(super) fn draw_topbar(&mut self, ctx: &egui::Context) {
        let connection_name = self.active_connection_name().to_owned();
        let driver = self.active_driver().to_owned();
        let has_connection = self.connection.lifecycle.active_connection_id().is_some();
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
                    let toggle_tooltip = if self.workspace.sidebar_open {
                        format!("Collapse Sidebar ({}B)", modifier)
                    } else {
                        format!("Expand Sidebar ({}B)", modifier)
                    };
                    let toggle_icon = if self.workspace.sidebar_open {
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
                        self.workspace.sidebar_open = !self.workspace.sidebar_open;
                    }
                    ui.add_space(2.0);

                    // 1. History Navigation (Back / Forward)
                    let can_go_back = self.query.session.active_document_index > 0;
                    let can_go_forward =
                        self.query.session.active_document_index + 1 < self.query.session.documents.len();
                    if Button::new(self.theme)
                        .icon(Icon::ArrowLeft)
                        .variant(ButtonVariant::Ghost)
                        .size(ButtonSize::IconSm)
                        .enabled(can_go_back)
                        .tooltip("Previous Document")
                        .show(ui)
                        .clicked()
                        && self.query.session.active_document_index > 0
                    {
                        self.switch_query_document(self.query.session.active_document_index - 1);
                    }
                    if Button::new(self.theme)
                        .icon(Icon::ArrowRight)
                        .variant(ButtonVariant::Ghost)
                        .size(ButtonSize::IconSm)
                        .enabled(can_go_forward)
                        .tooltip("Next Document")
                        .show(ui)
                        .clicked()
                        && self.query.session.active_document_index + 1 < self.query.session.documents.len()
                    {
                        self.switch_query_document(self.query.session.active_document_index + 1);
                    }

                    ui.add_space(SPACE_SM);
                    ui.label(RichText::new("│").font(font_caption()).color(self.theme.border_subtle));
                    ui.add_space(SPACE_SM);

                    // 2. Active Context Breadcrumb
                    if has_connection {
                        ui.horizontal(|ui| {
                            ui.label(icon_text(connection_icon, "", connection_color));
                            let display_name = crate::components::truncate_ellipsis(&connection_name, 22);
                            let resp = ui.label(
                                RichText::new(display_name)
                                    .font(font_caption())
                                    .strong()
                                    .color(self.theme.text_primary),
                            );
                            if connection_name.len() > 22 {
                                resp.on_hover_text(&connection_name);
                            }
                            let driver_tag = if driver.to_ascii_lowercase().contains("sqlite") {
                                "SQLite"
                            } else {
                                "PostgreSQL"
                            };
                            Badge::new(driver_tag, self.theme)
                                .variant(BadgeVariant::Secondary)
                                .compact(true)
                                .show(ui);
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
                            self.palette.open(PaletteMode::Commands);
                        }
                        if Button::new(self.theme)
                            .icon(Icon::Palette)
                            .variant(ButtonVariant::Ghost)
                            .size(ButtonSize::IconSm)
                            .tooltip("Component Gallery (UI Design System)")
                            .show(ui)
                            .clicked()
                        {
                            self.workspace.active_tab = WorkspaceTab::ComponentGallery;
                        }
                        // No shortcut is bound to the agent panel, so the tooltip must
                        // not advertise one (it previously claimed a hardcoded ⌘I that
                        // existed on no platform and in no handler).
                        let agent_tooltip = if self.workspace.agent_open {
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
                            self.set_agent_open(!self.workspace.agent_open, ctx);
                        }
                        let theme_icon = if self.preferences.dark_mode {
                            Icon::Sun
                        } else {
                            Icon::Moon
                        };
                        let theme_tooltip = if self.preferences.dark_mode {
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
                            self.preferences.dark_mode = !self.preferences.dark_mode;
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
                            self.palette.open(PaletteMode::QuickOpen);
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
                    if self.connection.lifecycle.is_connected() {
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
                    if let Some(result) = self
                        .query
                        .session
                        .active_result()
                        .or(self.table.data_query.result.as_ref())
                    {
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
                        if Button::new(self.theme)
                            .icon(Icon::PanelBottom)
                            .variant(ButtonVariant::Ghost)
                            .size(ButtonSize::IconSm)
                            .tooltip("Toggle output panel")
                            .show(ui)
                            .clicked()
                        {
                            self.workspace.bottom_panel_open = !self.workspace.bottom_panel_open;
                        }
                        if self.shows_editor_status() {
                            ui.label(RichText::new("UTF-8").font(font_mono_sm()).color(self.theme.text_muted));
                            ui.label(
                                RichText::new(format!(
                                    "Ln {}, Col {}",
                                    self.query.editor.query_cursor_line, self.query.editor.query_cursor_column
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
        if !self.workspace.bottom_panel_open {
            return;
        }
        let height = self.workspace.bottom_panel_height;
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
                        if tab_frame(self.theme, self.query.output.active_tab == tab)
                            .show(ui, |ui| ui.selectable_label(self.query.output.active_tab == tab, label))
                            .inner
                            .clicked()
                        {
                            self.query.output.active_tab = tab;
                        }
                    }
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        if compact_icon_button(ui, Icon::X, self.theme)
                            .on_hover_text("Close output")
                            .clicked()
                        {
                            self.workspace.bottom_panel_open = false;
                        }
                    });
                });
                ui.separator();
                match self.query.output.active_tab {
                    OutputTab::Results => {
                        let result = self
                            .query
                            .session
                            .active_result()
                            .or(self.table.data_query.result.as_ref());
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
                        for message in self.query.session.active_messages().iter().rev().take(8) {
                            ui.label(RichText::new(message).small().color(self.theme.text_secondary));
                        }
                    }
                    OutputTab::Explain => {
                        if let Some(plan) = self.query.session.active_explain_plan() {
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
                        for query in self.query.editor.query_history.iter().rev().take(8) {
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
        self.workspace.set_bottom_panel_height(response.response.rect.height());
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
                .schema_explorer
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
                        .schema_explorer
                        .schema
                        .table_details
                        .first()
                        .map(|t| t.columns.iter().map(|c| c.name.clone()).collect())
                        .unwrap_or_default();
                    self.management.masking.masking_columns_csv =
                        db_pro_core::domain::masking::suggest_sensitive_columns(&names).join(",");
                }
                if secondary_button(ui, "Preview sample", self.theme).clicked() {
                    self.preview_masking_sample();
                }
                if secondary_button(ui, "Masked CSV export", self.theme).clicked() {
                    self.run_masked_csv_export_harness();
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
    pub(super) fn draw_diagram_sidebar(&mut self, ui: &mut egui::Ui) {
        ui.vertical(|ui| {
            section_label(ui, "SCHEMA MAP", self.theme);
            ui.add_space(8.0);
            ui.label(RichText::new("Tables and foreign-key relationships").color(self.theme.text_primary));
            ui.add_space(4.0);
            ui.label(
                RichText::new(format!(
                    "{} · {}",
                    plural_count(self.schema_explorer.schema.table_details.len(), "table", "tables"),
                    plural_count(
                        self.schema_explorer
                            .schema
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
                self.workspace.activity = Activity::Explorer;
                self.workspace.sidebar_open = true;
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
            let connection_name = self.active_connection_name().to_owned();
            self.schema_compare
                .take_snapshot(&self.schema_explorer.schema, &connection_name, &mut self.feedback);
        }
        if compact_button_with_icon(ui, Icon::GitCompare, "Diff vs snapshot", self.theme).clicked() {
            self.schema_compare
                .diff_against_snapshot(&self.schema_explorer.schema, &mut self.feedback);
            self.workspace.active_tab = WorkspaceTab::SchemaCompare;
        }
        ui.add_space(8.0);
        if let Some(snap) = &self.schema_compare.schema_snapshot {
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
                    self.schema_compare
                        .diff_against_snapshot(&self.schema_explorer.schema, &mut self.feedback);
                }
                if secondary_button_with_icon(ui, Icon::Camera, "Snapshot", self.theme).clicked() {
                    let connection_name = self.active_connection_name().to_owned();
                    self.schema_compare.take_snapshot(
                        &self.schema_explorer.schema,
                        &connection_name,
                        &mut self.feedback,
                    );
                }
            });
        });
        ui.add_space(SPACE_MD);
        let Some(diff) = self.schema_compare.schema_diff.clone() else {
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
                self.schema_compare.plan_migration(&driver, &mut self.feedback);
            }
            if let Some(plan) = &self.schema_compare.migration_plan {
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
            if !self.schema_compare.migration_preview_sql.is_empty() {
                ui.label(
                    RichText::new(&self.schema_compare.migration_preview_sql)
                        .small()
                        .monospace()
                        .color(self.theme.text_primary),
                );
                if self
                    .schema_compare
                    .migration_plan
                    .as_ref()
                    .is_some_and(|p| p.has_destructive)
                {
                    ui.checkbox(
                        &mut self.schema_compare.migration_confirm_destructive,
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
                &mut self.schema_compare.data_diff_target_id,
                "target connection id",
                self.theme,
            );
            input_full_width(ui, &mut self.schema_compare.data_diff_schema, "schema", self.theme);
            input_full_width(ui, &mut self.schema_compare.data_diff_table, "table", self.theme);
            input_full_width(
                ui,
                &mut self.schema_compare.data_diff_keys,
                "key columns (comma)",
                self.theme,
            );
            if primary_button_with_icon(ui, Icon::GitCompare, "Compare rows", self.theme).clicked() {
                self.request_data_diff_keyed();
            }
            if let Some(diff) = &self.schema_compare.data_diff_result {
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
                            .selectable_label(self.schema_compare.data_diff_filter == label, label)
                            .clicked()
                        {
                            self.schema_compare.data_diff_filter = label.to_owned();
                        }
                    }
                });
                for row in &diff.row_diffs {
                    let include = match self.schema_compare.data_diff_filter.as_str() {
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
        match self.schema_compare.build_data_diff_request(request_id, source_id) {
            Ok(command) => {
                self.dispatch_command(command);
                self.feedback.runtime_message = "Running key-aware data compare…".into();
            }
            Err(error) => self.feedback.runtime_message = error,
        }
    }
}
