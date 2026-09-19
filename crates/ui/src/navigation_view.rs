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
                    let can_go_back = self.query_session_state.active_document_index > 0;
                    let can_go_forward =
                        self.query_session_state.active_document_index + 1 < self.query_session_state.documents.len();
                    if Button::new(self.theme)
                        .icon(Icon::ArrowLeft)
                        .variant(ButtonVariant::Ghost)
                        .size(ButtonSize::IconSm)
                        .enabled(can_go_back)
                        .tooltip("Previous Document")
                        .show(ui)
                        .clicked()
                        && self.query_session_state.active_document_index > 0
                    {
                        self.switch_query_document(self.query_session_state.active_document_index - 1);
                    }
                    if Button::new(self.theme)
                        .icon(Icon::ArrowRight)
                        .variant(ButtonVariant::Ghost)
                        .size(ButtonSize::IconSm)
                        .enabled(can_go_forward)
                        .tooltip("Next Document")
                        .show(ui)
                        .clicked()
                        && self.query_session_state.active_document_index + 1 < self.query_session_state.documents.len()
                    {
                        self.switch_query_document(self.query_session_state.active_document_index + 1);
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
                        .active_query_result()
                        .or(self.table_state.table_data_result.as_ref())
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
                                    self.query_editor.query_cursor_line, self.query_editor.query_cursor_column
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
                        if tab_frame(self.theme, self.query_output_state.active_tab == tab)
                            .show(ui, |ui| {
                                ui.selectable_label(self.query_output_state.active_tab == tab, label)
                            })
                            .inner
                            .clicked()
                        {
                            self.query_output_state.active_tab = tab;
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
                match self.query_output_state.active_tab {
                    OutputTab::Results => {
                        let result = self
                            .active_query_result()
                            .or(self.table_state.table_data_result.as_ref());
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
                        for query in self.query_editor.query_history.iter().rev().take(8) {
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
                    .selected_text(if self.synthetic_data.synthetic_table.is_empty() {
                        "Select table…"
                    } else {
                        &self.synthetic_data.synthetic_table
                    })
                    .show_ui(ui, |ui| {
                        for (schema, name) in &tables {
                            let key = if schema.is_empty() {
                                name.clone()
                            } else {
                                format!("{schema}.{name}")
                            };
                            ui.selectable_value(&mut self.synthetic_data.synthetic_table, key.clone(), key);
                        }
                    });
                ui.label("rows");
                ui.add(egui::TextEdit::singleline(&mut self.synthetic_data.synthetic_row_count).desired_width(48.0));
                ui.label("seed");
                ui.add(egui::TextEdit::singleline(&mut self.synthetic_data.synthetic_seed).desired_width(64.0));
                ui.label("null%");
                ui.add(egui::TextEdit::singleline(&mut self.synthetic_data.synthetic_null_pct).desired_width(36.0));
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
                    &mut self.synthetic_data.synthetic_production_confirm,
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
            if let Some(error) = &self.synthetic_data.synthetic_error {
                ui.colored_label(self.theme.danger, error);
            }
            if let Some(preview) = &self.synthetic_data.synthetic_preview {
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
                    egui::TextEdit::singleline(&mut self.masking.masking_columns_csv).hint_text("cols: email,phone"),
                );
                egui::ComboBox::from_id_salt("mask_rule")
                    .selected_text(format!("{:?}", self.masking.masking_rule))
                    .show_ui(ui, |ui| {
                        ui.selectable_value(
                            &mut self.masking.masking_rule,
                            db_pro_core::domain::masking::MaskRule::Redact,
                            "Redact",
                        );
                        ui.selectable_value(
                            &mut self.masking.masking_rule,
                            db_pro_core::domain::masking::MaskRule::Hash,
                            "Hash",
                        );
                        ui.selectable_value(
                            &mut self.masking.masking_rule,
                            db_pro_core::domain::masking::MaskRule::PartialReveal,
                            "Partial",
                        );
                        ui.selectable_value(
                            &mut self.masking.masking_rule,
                            db_pro_core::domain::masking::MaskRule::Fixed,
                            "Fixed",
                        );
                        ui.selectable_value(
                            &mut self.masking.masking_rule,
                            db_pro_core::domain::masking::MaskRule::Synthetic,
                            "Synthetic",
                        );
                    });
                ui.checkbox(&mut self.masking.masking_keyed, "Keyed hash");
                if secondary_button(ui, "Suggest cols", self.theme).clicked() {
                    let names: Vec<String> = self
                        .schema_explorer
                        .schema
                        .table_details
                        .first()
                        .map(|t| t.columns.iter().map(|c| c.name.clone()).collect())
                        .unwrap_or_default();
                    self.masking.masking_columns_csv =
                        db_pro_core::domain::masking::suggest_sensitive_columns(&names).join(",");
                }
                if secondary_button(ui, "Preview sample", self.theme).clicked() {
                    self.preview_masking_sample();
                }
                if secondary_button(ui, "Masked CSV export", self.theme).clicked() {
                    self.run_masked_csv_export_harness();
                }
            });
            if let Some(error) = &self.masking.masking_error {
                ui.colored_label(self.theme.danger, error);
            }
            if let Some(preview) = &self.masking.masking_preview {
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
                self.transfer.transfer_jobs.clear();
            }
        });
        ui.add_space(SPACE_MD);
        if self.transfer.transfer_jobs.is_empty() {
            empty_state(
                ui,
                Icon::Upload,
                "No transfers yet",
                "Run the synthetic harness to verify streaming progress, or import/export from Query once formats land.",
                self.theme,
            );
            return;
        }
        for job in &self.transfer.transfer_jobs {
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

    fn build_synthetic_plan(&self) -> Result<db_pro_core::domain::synthetic_data::SyntheticPlan, String> {
        use db_pro_core::domain::synthetic_data::{infer_generator, ColumnSpec, SyntheticPlan};
        if self.synthetic_data.synthetic_table.is_empty() {
            return Err("select a table".into());
        }
        let (schema, name) = if let Some((s, t)) = self.synthetic_data.synthetic_table.split_once('.') {
            (s.to_owned(), t.to_owned())
        } else {
            (String::new(), self.synthetic_data.synthetic_table.clone())
        };
        let detail = self
            .schema_explorer
            .schema
            .table_details
            .iter()
            .find(|t| t.name == name && (schema.is_empty() || t.schema == schema))
            .ok_or_else(|| "table metadata not loaded".to_owned())?;
        let row_count: u64 = self
            .synthetic_data
            .synthetic_row_count
            .parse()
            .map_err(|_| "invalid row count".to_owned())?;
        let seed: u64 = self
            .synthetic_data
            .synthetic_seed
            .parse()
            .map_err(|_| "invalid seed".to_owned())?;
        let null_rate_pct: u8 = self
            .synthetic_data
            .synthetic_null_pct
            .parse()
            .map_err(|_| "invalid null %".to_owned())?;
        let mut columns: Vec<ColumnSpec> = detail
            .columns
            .iter()
            .map(|c| ColumnSpec {
                name: c.name.clone(),
                data_type: c.data_type.clone(),
                nullable: c.nullable,
                is_primary_key: c.is_primary_key,
                generator: infer_generator(&c.data_type),
                fk_values: Vec::new(),
            })
            .collect();
        // FK-aware: cycle deterministic keys 1..=N for each FK column when possible.
        for fk in &detail.foreign_keys {
            for (from_col, _to_col) in fk.from_columns.iter().zip(fk.to_columns.iter()) {
                if let Some(col) = columns.iter_mut().find(|c| c.name == *from_col) {
                    let pool_len = row_count.clamp(1, 50) as usize;
                    let pool: Vec<String> = (1..=pool_len).map(|i| i.to_string()).collect();
                    col.fk_values = pool;
                }
            }
        }
        Ok(SyntheticPlan {
            schema: detail.schema.clone(),
            table: detail.name.clone(),
            columns,
            row_count,
            seed,
            null_rate_pct,
        })
    }

    pub(crate) fn preview_synthetic_seed(&mut self) {
        match self.build_synthetic_plan() {
            Ok(plan) => match db_pro_core::domain::synthetic_data::generate_preview(&plan, 20) {
                Ok(preview) => {
                    self.synthetic_data.synthetic_preview = Some(preview);
                    self.synthetic_data.synthetic_error = None;
                }
                Err(err) => {
                    self.synthetic_data.synthetic_error = Some(err);
                    self.synthetic_data.synthetic_preview = None;
                }
            },
            Err(err) => {
                self.synthetic_data.synthetic_error = Some(err);
                self.synthetic_data.synthetic_preview = None;
            }
        }
    }

    pub(crate) fn export_synthetic_seed_sql(&mut self) {
        let plan = match self.build_synthetic_plan() {
            Ok(p) => p,
            Err(err) => {
                self.synthetic_data.synthetic_error = Some(err);
                return;
            }
        };
        let count = plan.row_count.min(500) as usize;
        let rows = match db_pro_core::domain::synthetic_data::generate_rows(&plan, count) {
            Ok(r) => r,
            Err(err) => {
                self.synthetic_data.synthetic_error = Some(err);
                return;
            }
        };
        match db_pro_core::domain::synthetic_data::render_insert_sql(&plan, &rows) {
            Ok(sql) => {
                self.set_active_query_text(sql);
                self.workspace.active_tab = WorkspaceTab::Query;
                self.synthetic_data.synthetic_error = None;
                self.feedback.runtime_message = format!("Synthetic INSERT SQL ({count} rows) exported to Query editor");
            }
            Err(err) => self.synthetic_data.synthetic_error = Some(err),
        }
    }

    pub(crate) fn apply_synthetic_seed(&mut self) {
        let is_production = self
            .active_connection()
            .map(|c| c.environment.eq_ignore_ascii_case("Production"))
            .unwrap_or(false);
        if is_production && !self.synthetic_data.synthetic_production_confirm {
            self.synthetic_data.synthetic_error =
                Some("Production confirmation required before applying seed INSERT".into());
            return;
        }
        self.export_synthetic_seed_sql();
        if self.synthetic_data.synthetic_error.is_some() {
            return;
        }
        if self.connection.lifecycle.active_connection_id().is_none() || !self.connection.lifecycle.is_connected() {
            self.synthetic_data.synthetic_error = Some("Connect to a database before applying seed".into());
            return;
        }
        self.dispatch_query();
        self.feedback.runtime_message = "Synthetic seed INSERT dispatched via query runtime".into();
    }

    pub(crate) fn preview_masking_sample(&mut self) {
        use db_pro_core::domain::masking::{preview_masking, ColumnMask, MaskingProfile};
        let cols: Vec<String> = self
            .masking
            .masking_columns_csv
            .split(',')
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .map(str::to_owned)
            .collect();
        let headers = if cols.is_empty() {
            vec!["email".into(), "phone".into(), "id".into()]
        } else {
            let mut h = cols.clone();
            if !h.iter().any(|c| c == "id") {
                h.push("id".into());
            }
            h
        };
        let sample = vec![
            headers
                .iter()
                .map(|h| match h.as_str() {
                    "email" => "ada@example.com".into(),
                    "phone" => "1234567890".into(),
                    "id" => "42".into(),
                    _ => format!("val_{h}"),
                })
                .collect::<Vec<_>>(),
            headers
                .iter()
                .map(|h| match h.as_str() {
                    "email" => "grace@example.com".into(),
                    "phone" => "0987654321".into(),
                    "id" => "42".into(),
                    _ => format!("val2_{h}"),
                })
                .collect::<Vec<_>>(),
        ];
        let profile = MaskingProfile {
            name: "preview".into(),
            schema: String::new(),
            table: String::new(),
            columns: cols
                .into_iter()
                .map(|column| ColumnMask {
                    column,
                    rule: self.masking.masking_rule,
                    replacement: "[masked]".into(),
                    keep_prefix: 2,
                    keep_suffix: 2,
                })
                .collect(),
            keyed: self.masking.masking_keyed,
            key_id: "local-dev".into(),
        };
        let key_material = if self.masking.masking_keyed {
            "db-pro-local-masking-key"
        } else {
            ""
        };
        self.masking.masking_preview = Some(preview_masking(&headers, &sample, &profile, key_material));
        self.masking.masking_error = None;
    }

    pub(crate) fn run_masked_csv_export_harness(&mut self) {
        use db_pro_core::application::{DelimitedFileTarget, DelimitedFormat, TransferService};
        use db_pro_core::domain::masking::{mask_transfer_batch, ColumnMask, MaskingProfile};
        use db_pro_core::domain::transfer::{
            TransferCancellation, TransferJob, TransferSourceKind, TransferStatus, TransferTargetKind,
        };

        let path = std::env::temp_dir().join(format!("db-pro-masked-{}.csv", self.transfer.transfer_jobs.len() + 1));
        let headers = vec!["id".into(), "email".into(), "phone".into()];
        let profile = MaskingProfile {
            name: "export".into(),
            schema: String::new(),
            table: String::new(),
            columns: self
                .masking
                .masking_columns_csv
                .split(',')
                .map(str::trim)
                .filter(|s| !s.is_empty())
                .map(|column| ColumnMask {
                    column: column.to_owned(),
                    rule: self.masking.masking_rule,
                    replacement: "[masked]".into(),
                    keep_prefix: 2,
                    keep_suffix: 2,
                })
                .collect(),
            keyed: self.masking.masking_keyed,
            key_id: "local-dev".into(),
        };
        let raw = vec![
            db_pro_core::domain::transfer::TransferRow {
                cells: vec!["1".into(), "ada@example.com".into(), "1234567890".into()],
            },
            db_pro_core::domain::transfer::TransferRow {
                cells: vec!["2".into(), "grace@example.com".into(), "0987654321".into()],
            },
        ];
        let masked = mask_transfer_batch(&headers, &raw, &profile, "db-pro-local-masking-key");
        let mut job = TransferJob {
            id: format!("masked-{}", self.transfer.transfer_jobs.len() + 1),
            label: format!("Masked CSV → {}", path.display()),
            source: TransferSourceKind::Synthetic {
                rows: masked.len() as u64,
            },
            target: TransferTargetKind::File {
                path: path.display().to_string(),
                format: "csv".into(),
            },
            mapping: Default::default(),
            batch_size: 50,
            options: Default::default(),
            status: TransferStatus::Pending,
            progress: Default::default(),
            error: None,
        };
        match DelimitedFileTarget::create(&path, DelimitedFormat::Csv, headers.clone()) {
            Ok(mut target) => {
                let cancel = TransferCancellation::new();
                struct MaskedSource {
                    rows: Vec<db_pro_core::domain::transfer::TransferRow>,
                    idx: usize,
                }
                impl db_pro_core::application::TransferSource for MaskedSource {
                    fn next_batch(
                        &mut self,
                        max_rows: usize,
                    ) -> Result<
                        Option<Vec<db_pro_core::domain::transfer::TransferRow>>,
                        db_pro_core::domain::transfer::TransferError,
                    > {
                        if self.idx >= self.rows.len() {
                            return Ok(None);
                        }
                        let end = (self.idx + max_rows).min(self.rows.len());
                        let batch = self.rows[self.idx..end].to_vec();
                        self.idx = end;
                        Ok(Some(batch))
                    }
                }
                let mut source = MaskedSource { rows: masked, idx: 0 };
                let _ = TransferService::run(&mut job, &mut source, &mut target, &cancel);
                self.feedback.runtime_message = format!("Masked CSV written to {}", path.display());
            }
            Err(err) => {
                job.status = TransferStatus::Failed;
                job.error = Some(err.to_string());
                self.masking.masking_error = Some(err.to_string());
            }
        }
        self.transfer.transfer_jobs.insert(0, job);
        if self.transfer.transfer_jobs.len() > 40 {
            self.transfer.transfer_jobs.truncate(40);
        }
    }

    pub(crate) fn run_synthetic_transfer_harness(&mut self, cancel_midway: bool) {
        use db_pro_core::application::TransferService;
        use db_pro_core::domain::transfer::{TransferCancellation, TransferJob, TransferStatus};

        let id = format!("xfer-{}", self.transfer.transfer_jobs.len() + 1);
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
        self.feedback.runtime_message = format!(
            "Transfer {} · {:?} · wrote {}",
            job.id, job.status, job.progress.rows_written
        );
        self.transfer.transfer_jobs.insert(0, job);
        if self.transfer.transfer_jobs.len() > 20 {
            self.transfer.transfer_jobs.truncate(20);
        }
    }

    pub(crate) fn run_csv_export_harness(&mut self) {
        use db_pro_core::application::{DelimitedFileTarget, DelimitedFormat, SyntheticSource, TransferService};
        use db_pro_core::domain::transfer::{
            TransferCancellation, TransferJob, TransferSourceKind, TransferStatus, TransferTargetKind,
        };

        let mut path = std::env::temp_dir();
        path.push(format!("dbpro-export-{}.csv", self.transfer.transfer_jobs.len() + 1));
        let id = format!("csv-{}", self.transfer.transfer_jobs.len() + 1);
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
        self.feedback.runtime_message = format!("CSV transfer {} · {:?} · {}", job.id, job.status, path.display());
        self.transfer.transfer_jobs.insert(0, job);
        if self.transfer.transfer_jobs.len() > 20 {
            self.transfer.transfer_jobs.truncate(20);
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

        let mut job =
            TransferJob::new_synthetic(format!("csv-import-{}", self.transfer.transfer_jobs.len() + 1), 0, 50);
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
        self.feedback.runtime_message = job.progress.message.clone();
        self.transfer.transfer_jobs.insert(0, job);
        if self.transfer.transfer_jobs.len() > 20 {
            self.transfer.transfer_jobs.truncate(20);
        }
    }

    pub(crate) fn run_jsonl_export_harness(&mut self) {
        use db_pro_core::application::{JsonlFileTarget, SyntheticSource, TransferService};
        use db_pro_core::domain::transfer::{TransferCancellation, TransferJob, TransferStatus, TransferTargetKind};

        let mut path = std::env::temp_dir();
        path.push(format!("dbpro-export-{}.jsonl", self.transfer.transfer_jobs.len() + 1));
        let mut job = TransferJob::new_synthetic(format!("jsonl-{}", self.transfer.transfer_jobs.len() + 1), 400, 50);
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
        self.feedback.runtime_message = format!("JSONL {} · {:?}", job.id, job.status);
        self.transfer.transfer_jobs.insert(0, job);
        if self.transfer.transfer_jobs.len() > 20 {
            self.transfer.transfer_jobs.truncate(20);
        }
    }

    pub(crate) fn run_excel_export_harness(&mut self) {
        use db_pro_core::application::{ExcelFileTarget, SyntheticSource, TransferService};
        use db_pro_core::domain::transfer::{TransferCancellation, TransferJob, TransferStatus, TransferTargetKind};

        let mut path = std::env::temp_dir();
        path.push(format!("dbpro-export-{}.xlsx", self.transfer.transfer_jobs.len() + 1));
        let mut job = TransferJob::new_synthetic(format!("xlsx-{}", self.transfer.transfer_jobs.len() + 1), 80, 20);
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
        self.feedback.runtime_message = format!("Excel {} · {:?}", job.id, job.status);
        self.transfer.transfer_jobs.insert(0, job);
        if self.transfer.transfer_jobs.len() > 20 {
            self.transfer.transfer_jobs.truncate(20);
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
                self.feedback.runtime_message = format!("DB→DB plan failed: {err}");
                return;
            }
        };
        if let Err(err) = plan.ensure_runnable() {
            self.feedback.runtime_message = format!("DB→DB blocked: {err}");
            return;
        }
        if let Err(err) = assert_endpoint_capabilities(true, true, false, false) {
            self.feedback.runtime_message = format!("DB→DB capability gate: {err}");
            return;
        }

        let mut job = TransferJob::new_db_table_copy(
            format!("dbdb-{}", self.transfer.transfer_jobs.len() + 1),
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
            self.feedback.runtime_message = format!(
                "DB→DB {} · {:?} · committed_batches={} · warnings={}",
                job.id,
                result.status,
                result.progress.committed_batches,
                plan.warnings.len()
            );
        } else {
            self.feedback.runtime_message = format!("DB→DB {} · {:?} · {:?}", job.id, result.status, job.error);
        }
        self.transfer.transfer_jobs.insert(0, job);
        if self.transfer.transfer_jobs.len() > 20 {
            self.transfer.transfer_jobs.truncate(20);
        }
    }

    /// Monitor activity: lightweight connection pulse from live app state.
    pub(super) fn draw_monitor_activity(&mut self, ui: &mut egui::Ui) {
        section_label(ui, "MONITOR", self.theme);
        ui.add_space(SPACE_SM);

        let connected =
            self.connection.lifecycle.is_connected() && self.connection.lifecycle.active_connection_id().is_some();
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
                    ui.checkbox(&mut self.monitoring.monitoring_poll, "Auto-refresh");
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

        if connected && self.monitoring.monitoring_poll {
            let due = self
                .monitoring
                .monitoring_last_poll
                .map(|t| t.elapsed() >= std::time::Duration::from_secs(5))
                .unwrap_or(true);
            if due {
                self.request_monitoring_snapshot();
            }
        }

        ui.add_space(SPACE_MD);
        if let Some(error) = &self.monitoring.monitoring_error {
            ui.colored_label(self.theme.warning, error);
            ui.add_space(SPACE_SM);
        }

        if let Some(snapshot) = self.monitoring.monitoring_snapshot.clone() {
            let health = db_pro_core::domain::health_advisor::analyze_health(
                &snapshot,
                snapshot.workload.as_ref(),
                &db_pro_core::domain::health_advisor::HealthAdvisorConfig::default(),
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .map(|d| d.as_millis() as u64)
                    .unwrap_or(0),
            );
            section_label(ui, "HEALTH ADVISOR", self.theme);
            ui.add_space(SPACE_SM);
            ui.label(
                RichText::new(format!(
                    "{} · snapshot @ {} ms",
                    health.message, health.snapshot_fetched_at_ms
                ))
                .small()
                .color(self.theme.text_muted),
            );
            ui.label(
                RichText::new("Deterministic heuristics only — never auto-mutates the database.")
                    .small()
                    .color(self.theme.text_muted),
            );
            ui.add_space(SPACE_SM);
            if health.findings.is_empty() {
                ui.label(
                    RichText::new("No findings for the current snapshot.")
                        .small()
                        .color(self.theme.text_secondary),
                );
            } else {
                for finding in health.findings.iter().take(25) {
                    let color = match finding.severity {
                        db_pro_core::domain::health_advisor::HealthSeverity::Critical => self.theme.danger,
                        db_pro_core::domain::health_advisor::HealthSeverity::Warning => self.theme.warning,
                        db_pro_core::domain::health_advisor::HealthSeverity::Info => self.theme.accent,
                    };
                    card_frame(self.theme).show(ui, |ui| {
                        ui.horizontal(|ui| {
                            badge(ui, finding.severity.as_label(), color, self.theme.text_primary);
                            ui.label(RichText::new(&finding.title).strong().color(self.theme.text_primary));
                        });
                        ui.label(
                            RichText::new(format!("affected: {}", finding.affected))
                                .small()
                                .color(self.theme.text_secondary),
                        );
                        ui.label(
                            RichText::new(format!("evidence: {}", finding.evidence))
                                .small()
                                .monospace()
                                .color(self.theme.text_muted),
                        );
                        ui.label(
                            RichText::new(&finding.explanation)
                                .small()
                                .color(self.theme.text_secondary),
                        );
                        ui.label(
                            RichText::new(format!("suggest: {}", finding.suggested_action))
                                .small()
                                .color(self.theme.text_primary),
                        );
                    });
                    ui.add_space(SPACE_SM);
                }
            }
            ui.add_space(SPACE_MD);

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
            ui.checkbox(
                &mut self.monitoring.monitoring_filter_active_only,
                "Active queries only",
            );
            ui.add_space(SPACE_SM);

            let idle_xacts = snapshot.idle_in_transaction_sessions();
            if !idle_xacts.is_empty() {
                section_label(ui, "IDLE IN TRANSACTION", self.theme);
                ui.add_space(SPACE_SM);
                for session in idle_xacts.into_iter().take(20) {
                    ui.label(
                        RichText::new(format!(
                            "pid {} · xact_age={:?} ms · backend_age={:?} ms · {}",
                            session.backend_id,
                            session.xact_age_ms,
                            session.backend_age_ms,
                            session.username.as_deref().unwrap_or("?")
                        ))
                        .small()
                        .color(self.theme.warning),
                    );
                }
                ui.add_space(SPACE_MD);
            }

            let sessions: Vec<_> = if self.monitoring.monitoring_filter_active_only {
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
                                ui.label(
                                    RichText::new(format!("query {ms} ms"))
                                        .small()
                                        .color(self.theme.text_muted),
                                );
                            }
                            if let Some(ms) = session.xact_age_ms {
                                ui.label(
                                    RichText::new(format!("xact {ms} ms"))
                                        .small()
                                        .color(self.theme.text_muted),
                                );
                            }
                            if session.idle_in_transaction {
                                badge(ui, "idle-in-xact", self.theme.warning, self.theme.text_primary);
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
                                    self.workspace.active_tab = WorkspaceTab::Query;
                                    self.workspace.activity = Activity::Explorer;
                                }
                                if !session.is_current
                                    && secondary_button_with_icon(ui, Icon::Ban, "Cancel", self.theme).clicked()
                                {
                                    if let Some(connection_id) =
                                        self.connection.lifecycle.active_connection_id().map(str::to_owned)
                                    {
                                        let request_id = self.task_bridge.next_request_id();
                                        self.dispatch_command(UiCommand::MonitoringCancelBackend {
                                            request_id,
                                            connection_id,
                                            backend_id: session.backend_id,
                                        });
                                    }
                                }
                                if !session.is_current && danger_button(ui, "Terminate", self.theme).clicked() {
                                    self.monitoring.monitoring_terminate_confirm = Some(session.backend_id);
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

            if let Some(workload) = &snapshot.workload {
                ui.add_space(SPACE_MD);
                section_label(ui, "TOP QUERIES (pg_stat_statements)", self.theme);
                ui.add_space(SPACE_SM);
                if !workload.extension_present {
                    ui.colored_label(self.theme.warning, &workload.message);
                } else {
                    if let Some(ver) = &workload.extension_version {
                        ui.label(
                            RichText::new(format!("extension v{ver} · {}", workload.message))
                                .small()
                                .color(self.theme.text_muted),
                        );
                    }
                    ui.horizontal_wrapped(|ui| {
                        use db_pro_core::domain::monitoring::StatStatementSort;
                        for sort in [
                            StatStatementSort::TotalTime,
                            StatStatementSort::MeanTime,
                            StatStatementSort::Calls,
                            StatStatementSort::Rows,
                        ] {
                            let selected = self.monitoring.monitoring_stat_sort == sort;
                            if ui.selectable_label(selected, sort.as_label()).clicked() {
                                self.monitoring.monitoring_stat_sort = sort;
                                self.request_monitoring_workload();
                            }
                        }
                        if danger_button(ui, "Reset stats…", self.theme).clicked() {
                            self.monitoring.monitoring_reset_stats_confirm = true;
                        }
                    });
                    ui.add_space(SPACE_XS);
                    ui.horizontal(|ui| {
                        ui.label(RichText::new("Filter").small().color(self.theme.text_muted));
                        ui.text_edit_singleline(&mut self.monitoring.monitoring_workload_filter);
                    });
                    let filter = self.monitoring.monitoring_workload_filter.to_ascii_lowercase();
                    let prev_by_id: std::collections::HashMap<Option<i64>, f64> = self
                        .monitoring
                        .monitoring_workload_prev
                        .as_ref()
                        .map(|prev| prev.statements.iter().map(|s| (s.queryid, s.total_time_ms)).collect())
                        .unwrap_or_default();
                    let rows: Vec<_> = workload
                        .statements
                        .iter()
                        .filter(|s| {
                            filter.is_empty()
                                || s.query.to_ascii_lowercase().contains(&filter)
                                || s.database
                                    .as_deref()
                                    .unwrap_or("")
                                    .to_ascii_lowercase()
                                    .contains(&filter)
                                || s.username
                                    .as_deref()
                                    .unwrap_or("")
                                    .to_ascii_lowercase()
                                    .contains(&filter)
                        })
                        .take(40)
                        .collect();
                    if rows.is_empty() {
                        ui.label(
                            RichText::new("No statements match the current filter.")
                                .small()
                                .color(self.theme.text_muted),
                        );
                    }
                    for stmt in rows {
                        card_frame(self.theme).show(ui, |ui| {
                            let delta = prev_by_id
                                .get(&stmt.queryid)
                                .map(|prev| stmt.total_time_ms - prev)
                                .filter(|d| d.abs() > 0.01);
                            ui.horizontal(|ui| {
                                ui.label(
                                    RichText::new(format!(
                                        "calls {} · total {:.1} ms · mean {:.1} ms · rows {}",
                                        stmt.calls, stmt.total_time_ms, stmt.mean_time_ms, stmt.rows
                                    ))
                                    .small()
                                    .strong()
                                    .color(self.theme.text_primary),
                                );
                                if let Some(delta) = delta {
                                    ui.label(
                                        RichText::new(format!("Δ total {delta:+.1} ms"))
                                            .small()
                                            .color(self.theme.accent),
                                    );
                                }
                            });
                            ui.label(
                                RichText::new(format!(
                                    "{} · {} · shared hit/read {}/{} · temp r/w {}/{}",
                                    stmt.username.as_deref().unwrap_or("?"),
                                    stmt.database.as_deref().unwrap_or("?"),
                                    stmt.shared_blks_hit,
                                    stmt.shared_blks_read,
                                    stmt.temp_blks_read,
                                    stmt.temp_blks_written
                                ))
                                .small()
                                .color(self.theme.text_muted),
                            );
                            let short = if stmt.query.len() > 160 {
                                format!("{}…", &stmt.query.chars().take(159).collect::<String>())
                            } else {
                                stmt.query.clone()
                            };
                            ui.label(RichText::new(short).monospace().small().color(self.theme.text_primary));
                            if ghost_button_with_icon(ui, Icon::FileCode2, "Open SQL", self.theme).clicked() {
                                self.set_active_query_text(stmt.query.clone());
                                self.workspace.active_tab = WorkspaceTab::Query;
                                self.workspace.activity = Activity::Explorer;
                            }
                        });
                        ui.add_space(SPACE_SM);
                    }
                }
            }

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
            if let Some(error) = &self.fdw.fdw_error {
                ui.colored_label(self.theme.danger, error);
            }
            if let Some(inv) = self.fdw.fdw_inventory.clone() {
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
                                self.fdw.fdw_ddl_preview =
                                    db_pro_core::domain::fdw::preview_drop_server(&s.name, true).ok();
                            }
                            if danger_button(ui, "Drop…", self.theme).clicked() {
                                self.fdw.fdw_drop_confirm = Some(s.name.clone());
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
                ui.add(egui::TextEdit::singleline(&mut self.fdw.fdw_create_name).hint_text("server name"));
                ui.add(egui::TextEdit::singleline(&mut self.fdw.fdw_create_wrapper).hint_text("fdw"));
            });
            ui.horizontal(|ui| {
                ui.add(egui::TextEdit::singleline(&mut self.fdw.fdw_create_host).hint_text("host"));
                ui.add(egui::TextEdit::singleline(&mut self.fdw.fdw_create_dbname).hint_text("dbname"));
                ui.add(egui::TextEdit::singleline(&mut self.fdw.fdw_create_port).hint_text("port"));
            });
            ui.horizontal(|ui| {
                if ghost_button_with_icon(ui, Icon::FileCode2, "Preview CREATE", self.theme).clicked() {
                    // allow: preview is best-effort — preview generation error (name validation) only hides preview without blocking Create
                    self.fdw.fdw_ddl_preview = db_pro_core::domain::fdw::preview_create_server(
                        &self.fdw.fdw_create_name,
                        &self.fdw.fdw_create_wrapper,
                        &self.fdw.fdw_create_host,
                        &self.fdw.fdw_create_dbname,
                        &self.fdw.fdw_create_port,
                    )
                    .ok();
                }
                if secondary_button(ui, "Create (confirm)", self.theme).clicked() {
                    self.create_fdw_server_confirmed();
                }
            });

            if let Some(preview) = self.fdw.fdw_ddl_preview.clone() {
                egui::Window::new("FDW DDL preview")
                    .collapsible(false)
                    .resizable(true)
                    .default_width(480.0)
                    .show(ui.ctx(), |ui| {
                        ui.label(RichText::new(preview).monospace());
                        if secondary_button(ui, "Close", self.theme).clicked() {
                            self.fdw.fdw_ddl_preview = None;
                        }
                    });
            }
            if let Some(name) = self.fdw.fdw_drop_confirm.clone() {
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
                                self.fdw.fdw_drop_confirm = None;
                            }
                        });
                    });
            }

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
            if let Some(error) = &self.replication.replication_error {
                ui.colored_label(self.theme.danger, error);
            }
            if let Some(inv) = self.replication.replication_inventory.clone() {
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
                                self.replication.replication_ddl_preview =
                                    db_pro_core::domain::replication::preview_drop_publication(&pub_info.name).ok();
                            }
                            if danger_button(ui, "Drop…", self.theme).clicked() {
                                self.replication.replication_drop_publication = Some(pub_info.name.clone());
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
                                self.replication.replication_ddl_preview =
                                    db_pro_core::domain::replication::preview_drop_subscription(&sub.name).ok();
                            }
                            if danger_button(ui, "Drop…", self.theme).clicked() {
                                self.replication.replication_drop_subscription = Some(sub.name.clone());
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
                    egui::TextEdit::singleline(&mut self.replication.replication_create_name)
                        .hint_text("publication name"),
                );
                if ghost_button_with_icon(ui, Icon::FileCode2, "Preview CREATE", self.theme).clicked() {
                    // allow: preview is best-effort — preview generation error (name validation) only hides preview without blocking Create
                    self.replication.replication_ddl_preview =
                        db_pro_core::domain::replication::preview_create_publication_all(
                            &self.replication.replication_create_name,
                        )
                        .ok();
                }
                if secondary_button(ui, "Create (confirm)", self.theme).clicked() {
                    self.create_publication_confirmed();
                }
            });

            if let Some(preview) = self.replication.replication_ddl_preview.clone() {
                egui::Window::new("Replication DDL preview")
                    .collapsible(false)
                    .resizable(true)
                    .default_width(480.0)
                    .show(ui.ctx(), |ui| {
                        ui.label(RichText::new(preview).monospace());
                        if secondary_button(ui, "Close", self.theme).clicked() {
                            self.replication.replication_ddl_preview = None;
                        }
                    });
            }
            if let Some(name) = self.replication.replication_drop_publication.clone() {
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
                                self.replication.replication_drop_publication = None;
                            }
                        });
                    });
            }
            if let Some(name) = self.replication.replication_drop_subscription.clone() {
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
                                self.replication.replication_drop_subscription = None;
                            }
                        });
                    });
            }

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
            if let Some(error) = &self.event_trigger.event_trigger_error {
                ui.colored_label(self.theme.danger, error);
            }
            if let Some(inv) = self.event_trigger.event_trigger_inventory.clone() {
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
                                self.event_trigger.event_trigger_ddl_preview =
                                    db_pro_core::domain::event_trigger::preview_drop_event_trigger(&trig.name).ok();
                            }
                            if ghost_button(ui, "Disable", self.theme).clicked() {
                                self.alter_event_trigger_confirmed(&trig.name, "disable");
                            }
                            if ghost_button(ui, "Enable", self.theme).clicked() {
                                self.alter_event_trigger_confirmed(&trig.name, "enable");
                            }
                            if danger_button(ui, "Drop…", self.theme).clicked() {
                                self.event_trigger.event_trigger_drop_confirm = Some(trig.name.clone());
                            }
                        });
                    });
                    ui.add_space(SPACE_XS);
                }
            }

            ui.add_space(SPACE_SM);
            ui.label(RichText::new("Create event trigger").small().strong());
            ui.horizontal(|ui| {
                ui.add(egui::TextEdit::singleline(&mut self.event_trigger.event_trigger_create_name).hint_text("name"));
                ui.add(
                    egui::TextEdit::singleline(&mut self.event_trigger.event_trigger_create_event).hint_text("event"),
                );
            });
            ui.horizontal(|ui| {
                ui.add(
                    egui::TextEdit::singleline(&mut self.event_trigger.event_trigger_create_function)
                        .hint_text("schema.func()"),
                );
                ui.add(
                    egui::TextEdit::singleline(&mut self.event_trigger.event_trigger_create_tags)
                        .hint_text("tags CSV optional"),
                );
            });
            ui.horizontal(|ui| {
                if ghost_button_with_icon(ui, Icon::FileCode2, "Preview CREATE", self.theme).clicked() {
                    // allow: preview is best-effort — preview generation error (name validation) only hides preview without blocking Create
                    self.event_trigger.event_trigger_ddl_preview =
                        db_pro_core::domain::event_trigger::preview_create_event_trigger(
                            &self.event_trigger.event_trigger_create_name,
                            &self.event_trigger.event_trigger_create_event,
                            &self.event_trigger.event_trigger_create_function,
                            &self.event_trigger.event_trigger_create_tags,
                        )
                        .ok();
                }
                if secondary_button(ui, "Create (confirm)", self.theme).clicked() {
                    self.create_event_trigger_confirmed();
                }
            });

            if let Some(preview) = self.event_trigger.event_trigger_ddl_preview.clone() {
                egui::Window::new("Event trigger DDL preview")
                    .collapsible(false)
                    .resizable(true)
                    .default_width(520.0)
                    .show(ui.ctx(), |ui| {
                        ui.label(RichText::new(preview).monospace());
                        if secondary_button(ui, "Close", self.theme).clicked() {
                            self.event_trigger.event_trigger_ddl_preview = None;
                        }
                    });
            }
            if let Some(name) = self.event_trigger.event_trigger_drop_confirm.clone() {
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
                                self.event_trigger.event_trigger_drop_confirm = None;
                            }
                        });
                    });
            }

            ui.add_space(SPACE_MD);
            section_label(ui, "SERVER SETTINGS (pg_settings)", self.theme);
            ui.add_space(SPACE_SM);
            ui.label(
                RichText::new(
                    "PostgreSQL-only · session SET/RESET for user-context GUCs · ALTER SYSTEM is preview-only",
                )
                .small()
                .color(self.theme.text_muted),
            );
            ui.horizontal(|ui| {
                if secondary_button_with_icon(ui, Icon::RefreshCw, "Load settings", self.theme).clicked() {
                    self.request_pg_settings();
                }
            });
            ui.add_space(SPACE_XS);
            ui.horizontal(|ui| {
                ui.label(RichText::new("Filter").small().color(self.theme.text_muted));
                ui.text_edit_singleline(&mut self.pg_settings.pg_settings_filter);
            });
            if let Some(error) = &self.pg_settings.pg_settings_error {
                ui.colored_label(self.theme.danger, error);
            }
            if let Some(snapshot) = self.pg_settings.pg_settings.clone() {
                ui.label(
                    RichText::new(format!(
                        "{} · fetched @ {} ms",
                        snapshot.message, snapshot.fetched_at_ms
                    ))
                    .small()
                    .color(self.theme.text_muted),
                );
                let filter = self.pg_settings.pg_settings_filter.to_ascii_lowercase();
                let rows: Vec<_> = snapshot
                    .settings
                    .iter()
                    .filter(|s| {
                        filter.is_empty()
                            || s.name.to_ascii_lowercase().contains(&filter)
                            || s.category.to_ascii_lowercase().contains(&filter)
                            || s.source.to_ascii_lowercase().contains(&filter)
                    })
                    .take(60)
                    .collect();
                for setting in rows {
                    card_frame(self.theme).show(ui, |ui| {
                        ui.horizontal(|ui| {
                            ui.label(
                                RichText::new(&setting.name)
                                    .strong()
                                    .monospace()
                                    .color(self.theme.text_primary),
                            );
                            if setting.pending_restart {
                                badge(ui, "pending restart", self.theme.warning, self.theme.text_primary);
                            }
                            if setting.sensitive {
                                badge(ui, "redacted", self.theme.surface_active, self.theme.text_secondary);
                            }
                        });
                        ui.label(
                            RichText::new(format!(
                                "{} · context={} · source={} · {}",
                                setting.category,
                                setting.context,
                                setting.source,
                                setting.display_setting()
                            ))
                            .small()
                            .color(self.theme.text_secondary),
                        );
                        if let Some(desc) = &setting.short_desc {
                            ui.label(RichText::new(desc).small().color(self.theme.text_muted));
                        }
                        ui.label(
                            RichText::new(setting.mutability_reason())
                                .small()
                                .color(self.theme.text_muted),
                        );
                        ui.horizontal(|ui| {
                            if setting.session_mutable() && !setting.sensitive {
                                if ghost_button_with_icon(ui, Icon::Pencil, "Edit session", self.theme).clicked() {
                                    self.pg_settings.pg_settings_edit_name = setting.name.clone();
                                    self.pg_settings.pg_settings_edit_value = setting.setting.clone();
                                }
                                if secondary_button(ui, "RESET", self.theme).clicked() {
                                    self.reset_pg_setting_session(&setting.name);
                                }
                            }
                            if !setting.sensitive
                                && ghost_button_with_icon(ui, Icon::FileCode2, "Preview ALTER SYSTEM", self.theme)
                                    .clicked()
                            {
                                // allow: preview is best-effort — preview generation error (name validation) only hides preview without blocking ALTER SYSTEM
                                self.pg_settings.pg_settings_preview =
                                    db_pro_core::domain::pg_settings::preview_alter_system(
                                        &setting.name,
                                        &setting.setting,
                                    )
                                    .ok();
                            }
                        });
                    });
                    ui.add_space(SPACE_SM);
                }
            }

            if !self.pg_settings.pg_settings_edit_name.is_empty() {
                egui::Window::new(format!("SET SESSION · {}", self.pg_settings.pg_settings_edit_name))
                    .collapsible(false)
                    .resizable(false)
                    .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
                    .show(ui.ctx(), |ui| {
                        ui.text_edit_singleline(&mut self.pg_settings.pg_settings_edit_value);
                        ui.horizontal(|ui| {
                            if secondary_button(ui, "Apply SET", self.theme).clicked() {
                                let name = self.pg_settings.pg_settings_edit_name.clone();
                                let value = self.pg_settings.pg_settings_edit_value.clone();
                                self.set_pg_setting_session(&name, &value);
                                self.pg_settings.pg_settings_edit_name.clear();
                            }
                            if ghost_button_with_icon(ui, Icon::X, "Cancel", self.theme).clicked() {
                                self.pg_settings.pg_settings_edit_name.clear();
                            }
                        });
                    });
            }

            if let Some(preview) = self.pg_settings.pg_settings_preview.clone() {
                egui::Window::new("ALTER SYSTEM preview")
                    .collapsible(false)
                    .resizable(true)
                    .default_width(480.0)
                    .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
                    .show(ui.ctx(), |ui| {
                        ui.label(RichText::new(&preview.note).small().color(self.theme.warning));
                        ui.label(RichText::new(&preview.sql).monospace());
                        if secondary_button(ui, "Close", self.theme).clicked() {
                            self.pg_settings.pg_settings_preview = None;
                        }
                    });
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
                        self.monitoring.monitoring_maintenance_confirm = Some(action);
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

        if let Some(backend_id) = self.monitoring.monitoring_terminate_confirm {
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
                            if let Some(connection_id) =
                                self.connection.lifecycle.active_connection_id().map(str::to_owned)
                            {
                                let request_id = self.task_bridge.next_request_id();
                                self.dispatch_command(UiCommand::MonitoringTerminateBackend {
                                    request_id,
                                    connection_id,
                                    backend_id,
                                });
                            }
                            self.monitoring.monitoring_terminate_confirm = None;
                        }
                        if secondary_button_with_icon(ui, Icon::X, "Cancel", self.theme).clicked() {
                            self.monitoring.monitoring_terminate_confirm = None;
                        }
                    });
                });
        }

        if let Some(action) = self.monitoring.monitoring_maintenance_confirm {
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
                            if let Some(connection_id) =
                                self.connection.lifecycle.active_connection_id().map(str::to_owned)
                            {
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
                            self.monitoring.monitoring_maintenance_confirm = None;
                        }
                        if secondary_button_with_icon(ui, Icon::X, "Cancel", self.theme).clicked() {
                            self.monitoring.monitoring_maintenance_confirm = None;
                        }
                    });
                });
        }

        if self.monitoring.monitoring_reset_stats_confirm {
            egui::Window::new("Reset pg_stat_statements?")
                .collapsible(false)
                .resizable(false)
                .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
                .show(ui.ctx(), |ui| {
                    ui.label(
                        "This clears all accumulated statement statistics on the server. \
                         It is an administrative action and cannot be undone.",
                    );
                    ui.horizontal(|ui| {
                        if danger_button(ui, "Reset statistics", self.theme).clicked() {
                            if let Some(connection_id) =
                                self.connection.lifecycle.active_connection_id().map(str::to_owned)
                            {
                                let request_id = self.task_bridge.next_request_id();
                                self.dispatch_command(UiCommand::MonitoringResetStatStatements {
                                    request_id,
                                    connection_id,
                                    confirmed: true,
                                });
                            }
                            self.monitoring.monitoring_reset_stats_confirm = false;
                        }
                        if secondary_button_with_icon(ui, Icon::X, "Cancel", self.theme).clicked() {
                            self.monitoring.monitoring_reset_stats_confirm = false;
                        }
                    });
                });
        }
    }

    fn request_monitoring_workload(&mut self) {
        let Some(connection_id) = self.connection.lifecycle.active_connection_id().map(str::to_owned) else {
            return;
        };
        let request_id = self.task_bridge.next_request_id();
        self.dispatch_command(UiCommand::MonitoringStatStatements {
            request_id,
            connection_id,
            sort: self.monitoring.monitoring_stat_sort,
            limit: 100,
        });
    }

    fn request_audit_page(&mut self) {
        let Some(connection_id) = self.connection.lifecycle.active_connection_id().map(str::to_owned) else {
            self.audit.audit_error = Some("Connect a database first".into());
            return;
        };
        let request_id = self.task_bridge.next_request_id();
        self.dispatch_command(UiCommand::AuditEventsLoad {
            request_id,
            connection_id,
            filter: db_pro_core::domain::audit::AuditFilter {
                text: self.audit.audit_filter_text.clone(),
                database: self.audit.audit_filter_database.clone(),
                username: self.audit.audit_filter_username.clone(),
                severity: self.audit.audit_filter_severity.clone(),
                command_tag: String::new(),
            },
            limit: Some(100),
        });
    }

    fn export_selected_audit_events(&mut self) {
        let Some(page) = &self.audit.audit_page else {
            self.audit.audit_error = Some("Load an audit page before exporting".into());
            return;
        };
        let selected: Vec<_> = page
            .events
            .iter()
            .filter(|e| self.audit.audit_selected.contains(&e.id) || self.audit.audit_bookmarks.contains(&e.id))
            .cloned()
            .collect();
        if selected.is_empty() {
            self.audit.audit_error = Some("Select or bookmark events to export".into());
            return;
        }
        self.audit.audit_export_preview = Some(db_pro_core::application::AuditService::export_selected(&selected));
        self.audit.audit_error = None;
        self.feedback.runtime_message = format!(
            "Audit export preview · {} row(s) · {}",
            selected.len(),
            page.export_warning
        );
    }

    fn request_pg_settings(&mut self) {
        let Some(connection_id) = self.connection.lifecycle.active_connection_id().map(str::to_owned) else {
            self.pg_settings.pg_settings_error = Some("Connect a PostgreSQL database first".into());
            return;
        };
        let driver = self.active_driver().to_ascii_lowercase();
        if !(driver.contains("postgres")) {
            self.pg_settings.pg_settings_error = Some("pg_settings is PostgreSQL-only".into());
            return;
        }
        let request_id = self.task_bridge.next_request_id();
        self.dispatch_command(UiCommand::ListPgSettings {
            request_id,
            connection_id,
        });
    }

    fn set_pg_setting_session(&mut self, name: &str, value: &str) {
        let Some(connection_id) = self.connection.lifecycle.active_connection_id().map(str::to_owned) else {
            return;
        };
        let request_id = self.task_bridge.next_request_id();
        self.dispatch_command(UiCommand::SetPgSettingSession {
            request_id,
            connection_id,
            name: name.to_owned(),
            value: value.to_owned(),
        });
    }

    fn reset_pg_setting_session(&mut self, name: &str) {
        let Some(connection_id) = self.connection.lifecycle.active_connection_id().map(str::to_owned) else {
            return;
        };
        let request_id = self.task_bridge.next_request_id();
        self.dispatch_command(UiCommand::ResetPgSettingSession {
            request_id,
            connection_id,
            name: name.to_owned(),
        });
    }

    fn request_fdw_inventory(&mut self) {
        let Some(connection_id) = self.connection.lifecycle.active_connection_id().map(str::to_owned) else {
            self.fdw.fdw_error = Some("Connect a PostgreSQL database first".into());
            return;
        };
        if !self.active_driver().to_ascii_lowercase().contains("postgres") {
            self.fdw.fdw_error = Some("FDW administration is PostgreSQL-only".into());
            return;
        }
        let request_id = self.task_bridge.next_request_id();
        self.dispatch_command(UiCommand::ListFdwInventory {
            request_id,
            connection_id,
        });
    }

    fn create_fdw_server_confirmed(&mut self) {
        let Some(connection_id) = self.connection.lifecycle.active_connection_id().map(str::to_owned) else {
            return;
        };
        let request_id = self.task_bridge.next_request_id();
        self.dispatch_command(UiCommand::CreateFdwServer {
            request_id,
            connection_id,
            name: self.fdw.fdw_create_name.clone(),
            fdw: self.fdw.fdw_create_wrapper.clone(),
            host: self.fdw.fdw_create_host.clone(),
            dbname: self.fdw.fdw_create_dbname.clone(),
            port: self.fdw.fdw_create_port.clone(),
            confirmed: true,
        });
    }

    fn drop_fdw_server_confirmed(&mut self, name: &str, cascade: bool) {
        let Some(connection_id) = self.connection.lifecycle.active_connection_id().map(str::to_owned) else {
            return;
        };
        let request_id = self.task_bridge.next_request_id();
        self.dispatch_command(UiCommand::DropFdwServer {
            request_id,
            connection_id,
            name: name.to_owned(),
            cascade,
            confirmed: true,
        });
    }

    fn request_replication_inventory(&mut self) {
        let Some(connection_id) = self.connection.lifecycle.active_connection_id().map(str::to_owned) else {
            self.replication.replication_error = Some("Connect a PostgreSQL database first".into());
            return;
        };
        if !self.active_driver().to_ascii_lowercase().contains("postgres") {
            self.replication.replication_error = Some("Logical replication administration is PostgreSQL-only".into());
            return;
        }
        let request_id = self.task_bridge.next_request_id();
        self.dispatch_command(UiCommand::ListReplicationInventory {
            request_id,
            connection_id,
        });
    }

    fn create_publication_confirmed(&mut self) {
        let Some(connection_id) = self.connection.lifecycle.active_connection_id().map(str::to_owned) else {
            return;
        };
        let request_id = self.task_bridge.next_request_id();
        self.dispatch_command(UiCommand::CreatePublicationAll {
            request_id,
            connection_id,
            name: self.replication.replication_create_name.clone(),
            confirmed: true,
        });
    }

    fn drop_publication_confirmed(&mut self, name: &str) {
        let Some(connection_id) = self.connection.lifecycle.active_connection_id().map(str::to_owned) else {
            return;
        };
        let request_id = self.task_bridge.next_request_id();
        self.dispatch_command(UiCommand::DropPublication {
            request_id,
            connection_id,
            name: name.to_owned(),
            confirmed: true,
        });
    }

    fn drop_subscription_confirmed(&mut self, name: &str) {
        let Some(connection_id) = self.connection.lifecycle.active_connection_id().map(str::to_owned) else {
            return;
        };
        let request_id = self.task_bridge.next_request_id();
        self.dispatch_command(UiCommand::DropSubscription {
            request_id,
            connection_id,
            name: name.to_owned(),
            confirmed: true,
        });
    }

    fn request_event_triggers(&mut self) {
        let Some(connection_id) = self.connection.lifecycle.active_connection_id().map(str::to_owned) else {
            self.event_trigger.event_trigger_error = Some("Connect a PostgreSQL database first".into());
            return;
        };
        if !self.active_driver().to_ascii_lowercase().contains("postgres") {
            self.event_trigger.event_trigger_error = Some("Event triggers are PostgreSQL-only".into());
            return;
        }
        let request_id = self.task_bridge.next_request_id();
        self.dispatch_command(UiCommand::ListEventTriggers {
            request_id,
            connection_id,
        });
    }

    fn create_event_trigger_confirmed(&mut self) {
        let Some(connection_id) = self.connection.lifecycle.active_connection_id().map(str::to_owned) else {
            return;
        };
        let request_id = self.task_bridge.next_request_id();
        self.dispatch_command(UiCommand::CreateEventTrigger {
            request_id,
            connection_id,
            name: self.event_trigger.event_trigger_create_name.clone(),
            event: self.event_trigger.event_trigger_create_event.clone(),
            function_ref: self.event_trigger.event_trigger_create_function.clone(),
            tags_csv: self.event_trigger.event_trigger_create_tags.clone(),
            confirmed: true,
        });
    }

    fn drop_event_trigger_confirmed(&mut self, name: &str) {
        let Some(connection_id) = self.connection.lifecycle.active_connection_id().map(str::to_owned) else {
            return;
        };
        let request_id = self.task_bridge.next_request_id();
        self.dispatch_command(UiCommand::DropEventTrigger {
            request_id,
            connection_id,
            name: name.to_owned(),
            confirmed: true,
        });
    }

    fn alter_event_trigger_confirmed(&mut self, name: &str, mode: &str) {
        let Some(connection_id) = self.connection.lifecycle.active_connection_id().map(str::to_owned) else {
            return;
        };
        let request_id = self.task_bridge.next_request_id();
        self.dispatch_command(UiCommand::AlterEventTrigger {
            request_id,
            connection_id,
            name: name.to_owned(),
            mode: mode.to_owned(),
            confirmed: true,
        });
    }

    fn request_monitoring_snapshot(&mut self) {
        let Some(connection_id) = self.connection.lifecycle.active_connection_id().map(str::to_owned) else {
            return;
        };
        self.monitoring.monitoring_last_poll = Some(std::time::Instant::now());
        let request_id = self.task_bridge.next_request_id();
        self.dispatch_command(UiCommand::MonitoringSnapshot {
            request_id,
            connection_id,
        });
    }

    pub(super) fn draw_security_activity(&mut self, ui: &mut egui::Ui) {
        section_label(ui, "SECURITY", self.theme);
        ui.add_space(SPACE_SM);
        let connected =
            self.connection.lifecycle.is_connected() && self.connection.lifecycle.active_connection_id().is_some();
        let is_pg = self.active_driver().eq_ignore_ascii_case("postgresql")
            || self.active_driver().eq_ignore_ascii_case("postgres");

        if !connected {
            ui.label(
                RichText::new("Connect a PostgreSQL database to manage roles and privileges.")
                    .small()
                    .color(self.theme.text_muted),
            );
            return;
        }
        if !is_pg {
            ui.label(
                RichText::new("User/role management is PostgreSQL-only (capability gated).")
                    .small()
                    .color(self.theme.text_muted),
            );
            return;
        }

        ui.horizontal(|ui| {
            if secondary_button_with_icon(ui, Icon::RefreshCw, "Refresh roles", self.theme).clicked() {
                self.request_security_users();
            }
        });
        if let Some(error) = &self.security.security_error {
            ui.colored_label(self.theme.warning, error);
        }

        ui.add_space(SPACE_MD);
        section_label(ui, "ROLES / USERS", self.theme);
        ui.add_space(SPACE_SM);
        if self.security.security_users.is_empty() {
            ui.label(
                RichText::new("No roles loaded yet — click Refresh.")
                    .small()
                    .color(self.theme.text_muted),
            );
        }
        for user in self.security.security_users.clone() {
            let selected = self.security.security_selected_role.as_deref() == Some(user.name.as_str());
            ui.horizontal(|ui| {
                if ui.selectable_label(selected, &user.name).clicked() {
                    self.security.security_selected_role = Some(user.name.clone());
                    self.request_security_role_details(&user.name);
                }
                if user.can_login {
                    badge(ui, "login", self.theme.surface_active, self.theme.text_secondary);
                }
                if user.is_super {
                    badge(ui, "super", self.theme.warning, self.theme.text_primary);
                }
                if user.can_create_db {
                    badge(ui, "createdb", self.theme.surface_active, self.theme.text_secondary);
                }
                if user.can_create_role {
                    badge(ui, "createrole", self.theme.surface_active, self.theme.text_secondary);
                }
                if danger_button(ui, "Drop", self.theme).clicked() {
                    self.security.security_drop_confirm = Some(user.name.clone());
                }
            });
        }

        ui.add_space(SPACE_MD);
        section_label(ui, "CREATE ROLE", self.theme);
        ui.add_space(SPACE_SM);
        input_full_width(ui, &mut self.security.security_new_role, "role name", self.theme);
        ui.checkbox(&mut self.security.security_new_role_login, "LOGIN");
        if primary_button_with_icon(ui, Icon::Plus, "Create role", self.theme).clicked()
            && !self.security.security_new_role.trim().is_empty()
        {
            if let Some(connection_id) = self.connection.lifecycle.active_connection_id().map(str::to_owned) {
                let request_id = self.task_bridge.next_request_id();
                self.dispatch_command(UiCommand::CreateRole {
                    request_id,
                    connection_id,
                    name: self.security.security_new_role.trim().to_owned(),
                    login: self.security.security_new_role_login,
                });
                self.security.security_new_role.clear();
            }
        }

        if let Some(role) = self.security.security_selected_role.clone() {
            ui.add_space(SPACE_MD);
            section_label(ui, format!("ATTRIBUTES · {role}"), self.theme);
            ui.add_space(SPACE_SM);
            ui.horizontal(|ui| {
                if secondary_button_with_icon(ui, Icon::Check, "LOGIN", self.theme).clicked() {
                    self.dispatch_alter_role(
                        &role,
                        db_pro_core::domain::user::RoleAttributes {
                            login: Some(true),
                            ..Default::default()
                        },
                    );
                }
                if secondary_button_with_icon(ui, Icon::X, "NOLOGIN", self.theme).clicked() {
                    self.dispatch_alter_role(
                        &role,
                        db_pro_core::domain::user::RoleAttributes {
                            login: Some(false),
                            ..Default::default()
                        },
                    );
                }
                if secondary_button_with_icon(ui, Icon::Database, "CREATEDB", self.theme).clicked() {
                    self.dispatch_alter_role(
                        &role,
                        db_pro_core::domain::user::RoleAttributes {
                            createdb: Some(true),
                            ..Default::default()
                        },
                    );
                }
                if secondary_button_with_icon(ui, Icon::Users, "CREATEROLE", self.theme).clicked() {
                    self.dispatch_alter_role(
                        &role,
                        db_pro_core::domain::user::RoleAttributes {
                            createrole: Some(true),
                            ..Default::default()
                        },
                    );
                }
            });

            ui.add_space(SPACE_MD);
            section_label(ui, format!("PASSWORD · {role}"), self.theme);
            ui.add_space(SPACE_SM);
            ui.label(
                RichText::new("Password is never logged or shown in runtime events.")
                    .small()
                    .color(self.theme.text_muted),
            );
            ui.add(
                egui::TextEdit::singleline(&mut self.security.security_password)
                    .password(true)
                    .hint_text("new password")
                    .desired_width(f32::INFINITY),
            );
            if primary_button_with_icon(ui, Icon::Key, "Update password", self.theme).clicked()
                && !self.security.security_password.is_empty()
            {
                if let Some(connection_id) = self.connection.lifecycle.active_connection_id().map(str::to_owned) {
                    let password = std::mem::take(&mut self.security.security_password);
                    let request_id = self.task_bridge.next_request_id();
                    self.dispatch_command(UiCommand::UpdateRolePassword {
                        request_id,
                        connection_id,
                        name: role.clone(),
                        password,
                    });
                }
            }

            ui.add_space(SPACE_MD);
            section_label(ui, format!("MEMBERSHIPS · {role}"), self.theme);
            ui.add_space(SPACE_SM);
            if self.security.security_memberships.is_empty() {
                ui.label(
                    RichText::new("No role memberships.")
                        .small()
                        .color(self.theme.text_muted),
                );
            } else {
                for membership in self.security.security_memberships.clone() {
                    ui.horizontal(|ui| {
                        ui.label(
                            RichText::new(format!("member of {}", membership.role))
                                .small()
                                .monospace()
                                .color(self.theme.text_secondary),
                        );
                        if danger_button(ui, "Revoke", self.theme).clicked() {
                            if let Some(connection_id) =
                                self.connection.lifecycle.active_connection_id().map(str::to_owned)
                            {
                                let request_id = self.task_bridge.next_request_id();
                                self.dispatch_command(UiCommand::RevokeMembership {
                                    request_id,
                                    connection_id,
                                    role: membership.role,
                                    member: role.clone(),
                                });
                            }
                        }
                    });
                }
            }
            input_full_width(
                ui,
                &mut self.security.security_membership_role,
                "grant role name",
                self.theme,
            );
            if secondary_button_with_icon(ui, Icon::Plus, "Grant membership", self.theme).clicked()
                && !self.security.security_membership_role.trim().is_empty()
            {
                if let Some(connection_id) = self.connection.lifecycle.active_connection_id().map(str::to_owned) {
                    let request_id = self.task_bridge.next_request_id();
                    self.dispatch_command(UiCommand::GrantMembership {
                        request_id,
                        connection_id,
                        role: self.security.security_membership_role.trim().to_owned(),
                        member: role.clone(),
                    });
                    self.security.security_membership_role.clear();
                }
            }

            ui.add_space(SPACE_MD);
            section_label(ui, format!("PRIVILEGES · {role}"), self.theme);
            ui.add_space(SPACE_SM);
            if self.security.security_privileges.is_empty() {
                ui.label(
                    RichText::new("No privileges listed for this role.")
                        .small()
                        .color(self.theme.text_muted),
                );
            } else {
                for privs in self.security.security_privileges.clone() {
                    ui.horizontal(|ui| {
                        let target = match privs.object_kind {
                            db_pro_core::domain::user::PrivilegeObjectKind::Database => privs.object_name.clone(),
                            db_pro_core::domain::user::PrivilegeObjectKind::Schema => privs.object_name.clone(),
                            _ => format!("{}.{}", privs.schema, privs.object_name),
                        };
                        ui.label(
                            RichText::new(format!(
                                "{} · {} · {}",
                                privs.object_kind.as_label(),
                                target,
                                privs.privilege_type
                            ))
                            .small()
                            .monospace()
                            .color(self.theme.text_secondary),
                        );
                        if danger_button(ui, "Revoke", self.theme).clicked() {
                            if let Some(connection_id) =
                                self.connection.lifecycle.active_connection_id().map(str::to_owned)
                            {
                                let request_id = self.task_bridge.next_request_id();
                                self.dispatch_command(UiCommand::RevokePrivilege {
                                    request_id,
                                    connection_id,
                                    role_name: role.clone(),
                                    object_kind: privs.object_kind,
                                    schema: privs.schema,
                                    object_name: privs.object_name,
                                    privilege: privs.privilege_type,
                                });
                            }
                        }
                    });
                }
            }

            ui.add_space(SPACE_SM);
            section_label(ui, "GRANT PRIVILEGE", self.theme);
            ui.horizontal(|ui| {
                for (label, kind) in [
                    ("table", db_pro_core::domain::user::PrivilegeObjectKind::Table),
                    ("schema", db_pro_core::domain::user::PrivilegeObjectKind::Schema),
                    ("database", db_pro_core::domain::user::PrivilegeObjectKind::Database),
                    ("sequence", db_pro_core::domain::user::PrivilegeObjectKind::Sequence),
                ] {
                    if ui
                        .selectable_label(self.security.security_grant_kind == kind, label)
                        .clicked()
                    {
                        self.security.security_grant_kind = kind;
                    }
                }
            });
            if !matches!(
                self.security.security_grant_kind,
                db_pro_core::domain::user::PrivilegeObjectKind::Database
                    | db_pro_core::domain::user::PrivilegeObjectKind::Schema
            ) {
                input_full_width(ui, &mut self.security.security_grant_schema, "schema", self.theme);
            }
            let object_hint = match self.security.security_grant_kind {
                db_pro_core::domain::user::PrivilegeObjectKind::Table => "table",
                db_pro_core::domain::user::PrivilegeObjectKind::Schema => "schema",
                db_pro_core::domain::user::PrivilegeObjectKind::Database => "database",
                db_pro_core::domain::user::PrivilegeObjectKind::Sequence => "sequence",
            };
            input_full_width(ui, &mut self.security.security_grant_object, object_hint, self.theme);
            input_full_width(
                ui,
                &mut self.security.security_grant_privilege,
                "privilege (SELECT/USAGE/CONNECT/…)",
                self.theme,
            );
            if primary_button_with_icon(ui, Icon::Plus, "Grant privilege", self.theme).clicked()
                && !self.security.security_grant_object.trim().is_empty()
                && !self.security.security_grant_privilege.trim().is_empty()
            {
                if let Some(connection_id) = self.connection.lifecycle.active_connection_id().map(str::to_owned) {
                    let request_id = self.task_bridge.next_request_id();
                    self.dispatch_command(UiCommand::GrantPrivilege {
                        request_id,
                        connection_id,
                        role_name: role,
                        object_kind: self.security.security_grant_kind,
                        schema: self.security.security_grant_schema.trim().to_owned(),
                        object_name: self.security.security_grant_object.trim().to_owned(),
                        privilege: self.security.security_grant_privilege.trim().to_owned(),
                    });
                }
            }
        }

        if let Some(name) = self.security.security_drop_confirm.clone() {
            egui::Window::new("Drop role?")
                .collapsible(false)
                .resizable(false)
                .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
                .show(ui.ctx(), |ui| {
                    ui.label(format!("Drop role `{name}`? This cannot be undone."));
                    ui.horizontal(|ui| {
                        if danger_button(ui, "Drop role", self.theme).clicked() {
                            if let Some(connection_id) =
                                self.connection.lifecycle.active_connection_id().map(str::to_owned)
                            {
                                let request_id = self.task_bridge.next_request_id();
                                self.dispatch_command(UiCommand::DropRole {
                                    request_id,
                                    connection_id,
                                    name,
                                });
                            }
                            self.security.security_drop_confirm = None;
                        }
                        if secondary_button_with_icon(ui, Icon::X, "Cancel", self.theme).clicked() {
                            self.security.security_drop_confirm = None;
                        }
                    });
                });
        }

        ui.add_space(SPACE_MD);
        section_label(ui, "ROW-LEVEL SECURITY", self.theme);
        ui.add_space(SPACE_SM);
        ui.label(
            RichText::new("Policy changes are administrative — preview SQL, then confirm apply.")
                .small()
                .color(self.theme.text_muted),
        );
        input_full_width(ui, &mut self.security.security_rls_schema, "schema", self.theme);
        input_full_width(ui, &mut self.security.security_rls_table, "table", self.theme);
        ui.horizontal(|ui| {
            if secondary_button_with_icon(ui, Icon::RefreshCw, "Inspect RLS", self.theme).clicked() {
                self.request_security_rls();
            }
            if secondary_button_with_icon(ui, Icon::Shield, "Enable RLS", self.theme).clicked() {
                self.preview_table_rls(false, true);
            }
            if secondary_button_with_icon(ui, Icon::ShieldOff, "Disable RLS", self.theme).clicked() {
                self.preview_table_rls(false, false);
            }
            if secondary_button_with_icon(ui, Icon::Lock, "Force RLS", self.theme).clicked() {
                self.preview_table_rls(true, true);
            }
            if secondary_button_with_icon(ui, Icon::Unlock, "No Force", self.theme).clicked() {
                self.preview_table_rls(true, false);
            }
        });
        if let Some(state) = self.security.security_rls_state.clone() {
            ui.label(
                RichText::new(format!(
                    "{}.{} · enabled={} · forced={} · {} policy(ies)",
                    state.schema,
                    state.table,
                    state.rls_enabled,
                    state.rls_forced,
                    state.policies.len()
                ))
                .small()
                .monospace()
                .color(self.theme.text_secondary),
            );
            for policy in state.policies {
                let roles = if policy.roles.is_empty() {
                    "PUBLIC".to_owned()
                } else {
                    policy.roles.join(", ")
                };
                ui.label(
                    RichText::new(format!(
                        "{} · {} · {} · roles[{}] · USING({}) · CHECK({})",
                        policy.name,
                        policy.command,
                        if policy.permissive { "PERMISSIVE" } else { "RESTRICTIVE" },
                        roles,
                        policy.using_expr.as_deref().unwrap_or("—"),
                        policy.with_check_expr.as_deref().unwrap_or("—"),
                    ))
                    .small()
                    .monospace()
                    .color(self.theme.text_secondary),
                );
                ui.horizontal(|ui| {
                    if danger_button(ui, "Drop policy", self.theme).clicked() {
                        self.preview_drop_rls_policy(&policy.name);
                    }
                    if secondary_button_with_icon(ui, Icon::Pencil, "Load for edit", self.theme).clicked() {
                        self.security.security_rls_policy_name = policy.name.clone();
                        self.security.security_rls_command = policy.command.clone();
                        self.security.security_rls_roles = policy.roles.join(", ");
                        self.security.security_rls_using = policy.using_expr.clone().unwrap_or_default();
                        self.security.security_rls_with_check = policy.with_check_expr.clone().unwrap_or_default();
                    }
                });
            }
        }
        ui.add_space(SPACE_SM);
        section_label(ui, "CREATE / ALTER POLICY", self.theme);
        input_full_width(
            ui,
            &mut self.security.security_rls_policy_name,
            "policy name",
            self.theme,
        );
        input_full_width(
            ui,
            &mut self.security.security_rls_command,
            "command (ALL/SELECT/INSERT/UPDATE/DELETE)",
            self.theme,
        );
        input_full_width(
            ui,
            &mut self.security.security_rls_roles,
            "roles (comma; empty=PUBLIC)",
            self.theme,
        );
        input_full_width(
            ui,
            &mut self.security.security_rls_using,
            "USING expression",
            self.theme,
        );
        input_full_width(
            ui,
            &mut self.security.security_rls_with_check,
            "WITH CHECK expression",
            self.theme,
        );
        ui.horizontal(|ui| {
            if primary_button_with_icon(ui, Icon::Eye, "Preview CREATE", self.theme).clicked() {
                self.preview_rls_policy(db_pro_core::domain::object_mutation::ObjectAction::Create);
            }
            if secondary_button_with_icon(ui, Icon::Pencil, "Preview ALTER", self.theme).clicked() {
                self.preview_rls_policy(db_pro_core::domain::object_mutation::ObjectAction::Alter);
            }
        });
        if !self.security.security_rls_preview_sql.is_empty() {
            ui.label(
                RichText::new(&self.security.security_rls_preview_sql)
                    .small()
                    .monospace()
                    .color(self.theme.text_primary),
            );
            ui.checkbox(
                &mut self.security.security_rls_confirm_apply,
                "I understand this changes data visibility immediately",
            );
            if primary_button_with_icon(ui, Icon::Play, "Apply preview SQL", self.theme).clicked() {
                if !self.security.security_rls_confirm_apply {
                    self.feedback.runtime_message = "Confirm RLS apply checkbox first".into();
                } else {
                    self.apply_security_rls_preview();
                }
            }
        }
    }

    pub(crate) fn request_security_users(&mut self) {
        let Some(connection_id) = self.connection.lifecycle.active_connection_id().map(str::to_owned) else {
            return;
        };
        let request_id = self.task_bridge.next_request_id();
        self.dispatch_command(UiCommand::ListUsers {
            request_id,
            connection_id,
        });
    }

    pub(crate) fn request_security_role_details(&mut self, role_name: &str) {
        let Some(connection_id) = self.connection.lifecycle.active_connection_id().map(str::to_owned) else {
            return;
        };
        let request_id = self.task_bridge.next_request_id();
        self.dispatch_command(UiCommand::ListPrivileges {
            request_id,
            connection_id: connection_id.clone(),
            role_name: role_name.to_owned(),
        });
        let request_id = self.task_bridge.next_request_id();
        self.dispatch_command(UiCommand::ListMemberships {
            request_id,
            connection_id,
            member: role_name.to_owned(),
        });
    }

    pub(crate) fn request_security_rls(&mut self) {
        let Some(connection_id) = self.connection.lifecycle.active_connection_id().map(str::to_owned) else {
            return;
        };
        let schema = self.security.security_rls_schema.trim().to_owned();
        let table = self.security.security_rls_table.trim().to_owned();
        if schema.is_empty() || table.is_empty() {
            self.feedback.runtime_message = "Schema and table are required for RLS inspect".into();
            return;
        }
        let request_id = self.task_bridge.next_request_id();
        self.dispatch_command(UiCommand::ListTableRls {
            request_id,
            connection_id,
            schema,
            table,
        });
    }

    fn preview_table_rls(&mut self, force: bool, enable: bool) {
        use db_pro_core::application::ObjectMutationService;
        use db_pro_core::domain::object_mutation::{
            MutationOptions, ObjectAction, ObjectDefinition, ObjectMutationRequest, TableRlsDefinition,
        };
        use db_pro_core::ports::SqlDialect;

        struct QuoteDialect;
        impl SqlDialect for QuoteDialect {
            fn placeholder(&self, index: usize) -> String {
                format!("${index}")
            }
            fn quote_identifier(&self, name: &str) -> String {
                format!("\"{}\"", name.replace('"', "\"\""))
            }
        }

        let schema = self.security.security_rls_schema.trim().to_owned();
        let table = self.security.security_rls_table.trim().to_owned();
        if schema.is_empty() || table.is_empty() {
            self.feedback.runtime_message = "Schema and table are required".into();
            return;
        }
        let request = ObjectMutationRequest {
            action: if enable {
                ObjectAction::Enable
            } else {
                ObjectAction::Disable
            },
            target: None,
            definition: ObjectDefinition::TableRls(TableRlsDefinition { schema, table, force }),
            options: MutationOptions::default(),
            driver: "postgresql".into(),
        };
        match ObjectMutationService::plan(&request, &QuoteDialect) {
            Ok(preview) => {
                self.security.security_rls_preview_sql = preview.statements.join(";\n");
                if !self.security.security_rls_preview_sql.is_empty() {
                    self.security.security_rls_preview_sql.push(';');
                }
                self.security.security_rls_confirm_apply = false;
            }
            Err(err) => self.feedback.runtime_message = err.to_string(),
        }
    }

    fn preview_rls_policy(&mut self, action: db_pro_core::domain::object_mutation::ObjectAction) {
        use db_pro_core::application::ObjectMutationService;
        use db_pro_core::domain::object_mutation::{
            MutationOptions, ObjectDefinition, ObjectMutationRequest, RlsPolicyDefinition,
        };
        use db_pro_core::ports::SqlDialect;

        struct QuoteDialect;
        impl SqlDialect for QuoteDialect {
            fn placeholder(&self, index: usize) -> String {
                format!("${index}")
            }
            fn quote_identifier(&self, name: &str) -> String {
                format!("\"{}\"", name.replace('"', "\"\""))
            }
        }

        let schema = self.security.security_rls_schema.trim().to_owned();
        let table = self.security.security_rls_table.trim().to_owned();
        let name = self.security.security_rls_policy_name.trim().to_owned();
        if schema.is_empty() || table.is_empty() || name.is_empty() {
            self.feedback.runtime_message = "Schema, table, and policy name are required".into();
            return;
        }
        let roles = self
            .security
            .security_rls_roles
            .split(',')
            .map(|s| s.trim().to_owned())
            .filter(|s| !s.is_empty())
            .collect::<Vec<_>>();
        let request = ObjectMutationRequest {
            action,
            target: None,
            definition: ObjectDefinition::RlsPolicy(RlsPolicyDefinition {
                schema,
                table,
                name,
                permissive: true,
                command: self.security.security_rls_command.clone(),
                roles,
                using_expr: Some(self.security.security_rls_using.clone()).filter(|s| !s.trim().is_empty()),
                with_check_expr: Some(self.security.security_rls_with_check.clone()).filter(|s| !s.trim().is_empty()),
                new_name: None,
            }),
            options: MutationOptions::default(),
            driver: "postgresql".into(),
        };
        match ObjectMutationService::plan(&request, &QuoteDialect) {
            Ok(preview) => {
                if let Some(reason) = preview.unsupported_reason {
                    self.feedback.runtime_message = reason;
                    self.security.security_rls_preview_sql.clear();
                } else {
                    self.security.security_rls_preview_sql = preview.statements.join(";\n");
                    if !self.security.security_rls_preview_sql.is_empty() {
                        self.security.security_rls_preview_sql.push(';');
                    }
                    self.security.security_rls_confirm_apply = false;
                }
            }
            Err(err) => self.feedback.runtime_message = err.to_string(),
        }
    }

    fn preview_drop_rls_policy(&mut self, policy_name: &str) {
        use db_pro_core::application::ObjectMutationService;
        use db_pro_core::domain::object_mutation::{
            MutationOptions, ObjectAction, ObjectDefinition, ObjectMutationRequest, RlsPolicyDefinition,
        };
        use db_pro_core::ports::SqlDialect;

        struct QuoteDialect;
        impl SqlDialect for QuoteDialect {
            fn placeholder(&self, index: usize) -> String {
                format!("${index}")
            }
            fn quote_identifier(&self, name: &str) -> String {
                format!("\"{}\"", name.replace('"', "\"\""))
            }
        }

        let request = ObjectMutationRequest {
            action: ObjectAction::Drop,
            target: None,
            definition: ObjectDefinition::RlsPolicy(RlsPolicyDefinition {
                schema: self.security.security_rls_schema.trim().to_owned(),
                table: self.security.security_rls_table.trim().to_owned(),
                name: policy_name.to_owned(),
                permissive: true,
                command: "ALL".into(),
                roles: vec![],
                using_expr: None,
                with_check_expr: None,
                new_name: None,
            }),
            options: MutationOptions {
                if_exists: true,
                ..MutationOptions::default()
            },
            driver: "postgresql".into(),
        };
        match ObjectMutationService::plan(&request, &QuoteDialect) {
            Ok(preview) => {
                self.security.security_rls_preview_sql = preview.statements.join(";\n");
                if !self.security.security_rls_preview_sql.is_empty() {
                    self.security.security_rls_preview_sql.push(';');
                }
                self.security.security_rls_confirm_apply = false;
            }
            Err(err) => self.feedback.runtime_message = err.to_string(),
        }
    }

    fn apply_security_rls_preview(&mut self) {
        if self.table_state.ddl_execution_request.is_some() {
            return;
        }
        let sql = self.security.security_rls_preview_sql.trim().to_owned();
        if sql.is_empty() {
            return;
        }
        let Some(connection_id) = self.connection.lifecycle.active_connection_id().map(str::to_owned) else {
            return;
        };
        let request_id = self.task_bridge.next_request_id();
        self.dispatch_command(UiCommand::ExecuteDdl {
            request_id,
            connection_id,
            sql,
        });
        self.table_state.ddl_execution_request = Some(request_id);
        self.feedback.runtime_message = "Applying RLS mutation…".into();
    }

    fn dispatch_alter_role(&mut self, name: &str, attributes: db_pro_core::domain::user::RoleAttributes) {
        let Some(connection_id) = self.connection.lifecycle.active_connection_id().map(str::to_owned) else {
            return;
        };
        let request_id = self.task_bridge.next_request_id();
        self.dispatch_command(UiCommand::AlterRole {
            request_id,
            connection_id,
            name: name.to_owned(),
            attributes,
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
            self.take_schema_snapshot();
        }
        if compact_button_with_icon(ui, Icon::GitCompare, "Diff vs snapshot", self.theme).clicked() {
            self.diff_against_schema_snapshot();
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
                    self.diff_against_schema_snapshot();
                }
                if secondary_button_with_icon(ui, Icon::Camera, "Snapshot", self.theme).clicked() {
                    self.take_schema_snapshot();
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
                self.plan_migration_from_schema_diff();
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
        let target_id = self.schema_compare.data_diff_target_id.trim().to_owned();
        if target_id.is_empty() {
            self.feedback.runtime_message = "Target connection id is required".into();
            return;
        }
        let table = self.schema_compare.data_diff_table.trim().to_owned();
        if table.is_empty() {
            self.feedback.runtime_message = "Table is required".into();
            return;
        }
        let key_columns = self
            .schema_compare
            .data_diff_keys
            .split(',')
            .map(|s| s.trim().to_owned())
            .filter(|s| !s.is_empty())
            .collect::<Vec<_>>();
        if key_columns.is_empty() {
            self.feedback.runtime_message = "At least one key column is required".into();
            return;
        }
        let request_id = self.task_bridge.next_request_id();
        self.dispatch_command(UiCommand::DiffTableDataKeyed {
            request_id,
            source_id,
            target_id,
            schema: self.schema_compare.data_diff_schema.trim().to_owned(),
            table,
            key_columns,
            sample_limit: Some(1_000),
        });
        self.feedback.runtime_message = "Running key-aware data compare…".into();
    }
}
