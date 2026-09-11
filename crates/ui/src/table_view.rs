use super::*;

impl DbProApp {
    pub(super) fn draw_welcome(&mut self, ui: &mut egui::Ui) {
        let modifier = Self::primary_modifier_label();
        let mut open_query = false;
        ui.vertical_centered(|ui| {
            ui.add_space(56.0);
            card_frame(self.theme).show(ui, |ui| {
                ui.set_max_width(680.0);
                section_label(ui, "DATABASE WORKSPACE", self.theme);
                ui.add_space(12.0);
                ui.label(RichText::new("A calmer way to work with databases").size(26.0).strong());
                ui.add_space(8.0);
                ui.label(
                    RichText::new("Connect, explore, and query with confidence.").color(self.theme.text_secondary),
                );
                ui.add_space(18.0);
                let prompt_response = input_full_width(
                    ui,
                    &mut self.welcome_prompt,
                    "Ask your database or paste SQL to get started…",
                    self.theme,
                );
                ui.add_space(8.0);
                ui.horizontal(|ui| {
                    if primary_button_with_icon(ui, Icon::ArrowUp, "Open in Query", self.theme).clicked()
                        || (prompt_response.has_focus() && ui.input(|input| input.key_pressed(egui::Key::Enter)))
                    {
                        open_query = true;
                    }
                    if ghost_button_with_icon(ui, Icon::Search, "Quick Open", self.theme).clicked() {
                        self.open_palette(PaletteMode::QuickOpen);
                    }
                });
                ui.add_space(18.0);
                ui.horizontal(|ui| {
                    if primary_button_with_icon(ui, Icon::Plus, "New query", self.theme).clicked() {
                        self.new_query_document();
                    }
                    if secondary_button_with_icon(ui, Icon::Database, "New connection", self.theme).clicked() {
                        self.open_new_connection();
                    }
                });
                ui.add_space(18.0);
                ui.separator();
                ui.add_space(10.0);
                ui.horizontal(|ui| {
                    ui.label(icon_text(Icon::Command, "", self.theme.accent));
                    ui.label(
                        RichText::new(format!("{modifier} P to open anything"))
                            .small()
                            .color(self.theme.text_muted),
                    );
                    ui.separator();
                    ui.label(icon_text(Icon::PanelLeft, "", self.theme.accent));
                    ui.label(
                        RichText::new(format!("{modifier} B to toggle explorer"))
                            .small()
                            .color(self.theme.text_muted),
                    );
                });
            });
        });
        if open_query && !self.welcome_prompt.trim().is_empty() {
            self.query_text = self.welcome_prompt.trim().to_owned();
            self.active_tab = WorkspaceTab::Query;
            self.runtime_message = "Opened prompt in Query".to_owned();
        }
    }

    pub(super) fn draw_table_workspace(&mut self, ui: &mut egui::Ui) {
        let Some(table_name) = self.selected_table.clone() else {
            self.active_tab = WorkspaceTab::Welcome;
            return;
        };
        if self.table_view == TableView::Ddl && self.table_ddl.is_none() && self.table_ddl_request.is_none() {
            self.request_table_ddl();
        }
        if self.table_view == TableView::Data
            && self.table_info.is_some()
            && self.table_data_result.is_none()
            && self.table_data_request.is_none()
            && self.table_data_error.is_none()
        {
            self.request_table_data();
        }

        toolbar_frame(self.theme).show(ui, |ui| {
            ui.set_min_width((ui.available_width() - 24.0).max(0.0));
            ui.horizontal_wrapped(|ui| {
                ui.label(icon_text(Icon::Table2, "Table", self.theme.text_primary));
                ui.label(icon_text(Icon::ChevronRight, "", self.theme.text_muted));
                ui.label(
                    RichText::new(format!("{}.{table_name}", self.active_schema()))
                        .strong()
                        .color(self.theme.accent),
                );
                ui.separator();
                for (view, icon, label) in [
                    (TableView::Structure, Icon::Columns3, "Structure"),
                    (TableView::Data, Icon::Table2, "Data"),
                    (TableView::Ddl, Icon::Code2, "DDL"),
                ] {
                    let selected = self.table_view == view;
                    let tab = tab_frame(self.theme, selected).show(ui, |ui| {
                        ui.selectable_label(selected, icon_text(icon, label, self.theme.text_primary))
                    });
                    if tab.inner.clicked() {
                        self.table_view = view;
                    }
                }
                if secondary_button_with_icon(ui, Icon::FileCode2, "Open in Query", self.theme).clicked() {
                    self.active_tab = WorkspaceTab::Query;
                }
            });
        });
        ui.add_space(12.0);

        match self.table_view {
            TableView::Structure => self.draw_table_structure(ui),
            TableView::Data => self.draw_table_data(ui, &table_name),
            TableView::Ddl => self.draw_table_ddl(ui, &table_name),
        }
    }

    fn draw_table_structure(&self, ui: &mut egui::Ui) {
        let Some(info) = self.table_info.clone() else {
            card_frame(self.theme).show(ui, |ui| {
                ui.vertical_centered(|ui| {
                    ui.add_space(28.0);
                    let failed = self.table_info_error.as_deref();
                    ui.label(icon_text(
                        if failed.is_some() {
                            Icon::TriangleAlert
                        } else {
                            Icon::LoaderCircle
                        },
                        "",
                        if failed.is_some() {
                            self.theme.warning
                        } else {
                            self.theme.accent
                        },
                    ));
                    ui.add_space(8.0);
                    ui.label(
                        RichText::new(if failed.is_some() {
                            "Table structure could not be loaded"
                        } else {
                            "Loading table structure…"
                        })
                        .strong()
                        .color(self.theme.text_primary),
                    );
                    ui.label(
                        RichText::new(failed.unwrap_or("Columns, keys and indexes will appear here."))
                            .small()
                            .color(self.theme.text_secondary),
                    );
                    ui.add_space(28.0);
                });
            });
            return;
        };

        toolbar_frame(self.theme).show(ui, |ui| {
            ui.horizontal_wrapped(|ui| {
                badge(
                    ui,
                    &format!("{} columns", info.columns.len()),
                    self.theme.accent_soft,
                    self.theme.accent,
                );
                badge(
                    ui,
                    &format!("{} indexes", info.indexes.len()),
                    self.theme.surface_active,
                    self.theme.text_secondary,
                );
                badge(
                    ui,
                    &format!("{} foreign keys", info.foreign_keys.len()),
                    self.theme.surface_active,
                    self.theme.text_secondary,
                );
                if let Some(row_count) = info.row_count {
                    badge(
                        ui,
                        &format!("{row_count} rows"),
                        self.theme.surface_active,
                        self.theme.text_secondary,
                    );
                }
            });
        });
        ui.add_space(10.0);

        card_frame(self.theme).show(ui, |ui| {
            ui.set_min_width(ui.available_width());
            section_label(ui, "COLUMNS", self.theme);
            ui.add_space(8.0);
            egui::ScrollArea::vertical().max_height(300.0).show(ui, |ui| {
                egui::Grid::new("table-structure-columns")
                    .num_columns(5)
                    .spacing([18.0, 8.0])
                    .striped(true)
                    .show(ui, |ui| {
                        for header in ["Column", "Type", "Nullable", "Key", "Default"] {
                            ui.label(RichText::new(header).small().strong().color(self.theme.text_secondary));
                        }
                        ui.end_row();
                        for column in &info.columns {
                            ui.label(RichText::new(&column.name).color(self.theme.text_primary));
                            ui.label(
                                RichText::new(&column.data_type)
                                    .monospace()
                                    .color(self.theme.code_keyword),
                            );
                            ui.label(
                                RichText::new(if column.nullable { "yes" } else { "no" })
                                    .small()
                                    .color(self.theme.text_secondary),
                            );
                            if column.is_primary_key {
                                ui.label(icon_text(Icon::KeyRound, "PK", self.theme.warning));
                            } else {
                                ui.label(RichText::new("—").small().color(self.theme.text_muted));
                            }
                            ui.label(
                                RichText::new(column.default.as_deref().unwrap_or("—"))
                                    .small()
                                    .color(self.theme.text_secondary),
                            );
                            ui.end_row();
                        }
                    });
            });
        });
        ui.add_space(10.0);

        ui.columns(2, |columns| {
            card_frame(self.theme).show(&mut columns[0], |ui| {
                section_label(ui, "INDEXES", self.theme);
                ui.add_space(6.0);
                if info.indexes.is_empty() {
                    ui.label(RichText::new("No indexes").small().color(self.theme.text_muted));
                }
                for index in &info.indexes {
                    ui.horizontal_wrapped(|ui| {
                        ui.label(icon_text(
                            if index.unique { Icon::BadgeCheck } else { Icon::List },
                            "",
                            self.theme.accent,
                        ));
                        ui.label(RichText::new(&index.name).color(self.theme.text_primary));
                        ui.label(
                            RichText::new(index.columns.join(", "))
                                .small()
                                .color(self.theme.text_secondary),
                        );
                    });
                }
            });
            card_frame(self.theme).show(&mut columns[1], |ui| {
                section_label(ui, "FOREIGN KEYS", self.theme);
                ui.add_space(6.0);
                if info.foreign_keys.is_empty() {
                    ui.label(RichText::new("No foreign keys").small().color(self.theme.text_muted));
                }
                for foreign_key in &info.foreign_keys {
                    ui.horizontal_wrapped(|ui| {
                        ui.label(icon_text(Icon::ArrowRightLeft, "", self.theme.accent));
                        ui.label(RichText::new(&foreign_key.name).color(self.theme.text_primary));
                        ui.label(
                            RichText::new(format!(
                                "{} → {}.{}",
                                foreign_key.from_columns.join(", "),
                                foreign_key.to_schema,
                                foreign_key.to_table,
                            ))
                            .small()
                            .color(self.theme.text_secondary),
                        );
                    });
                }
            });
        });
    }
}
