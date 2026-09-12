use super::*;

impl DbProApp {
    pub(super) fn draw_welcome(&mut self, ui: &mut egui::Ui) {
        let modifier = Self::primary_modifier_label();
        let mut open_query = false;

        let available_width = ui.available_width();
        let content_width = available_width.min(760.0);
        let content_height = ui.available_height();
        ui.horizontal(|ui| {
            ui.add_space(((available_width - content_width) / 2.0).max(0.0));
            ui.allocate_ui_with_layout(
                egui::vec2(content_width, content_height),
                Layout::top_down(Align::Min),
                |ui| {
                    ui.add_space(SPACE_2XL);
                    self.draw_welcome_header(ui, modifier);
                    ui.add_space(SPACE_XL);
                    open_query = self.draw_welcome_prompt_card(ui);
                    ui.add_space(SPACE_LG);
                    self.draw_welcome_footer(ui);
                },
            );
        });
        if open_query && !self.welcome_prompt.trim().is_empty() {
            self.query_text = self.welcome_prompt.trim().to_owned();
            self.active_tab = WorkspaceTab::Query;
            self.runtime_message = "Opened prompt in Query".to_owned();
        }
    }

    /// Title block plus the command-palette hint pinned to the right edge.
    fn draw_welcome_header(&self, ui: &mut egui::Ui, modifier: &str) {
        ui.horizontal(|ui| {
            ui.label(icon_text(Icon::Sparkles, "WELCOME", self.theme.accent));
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                ui.label(
                    RichText::new(format!("{modifier}K command palette"))
                        .font(font_caption())
                        .color(self.theme.text_muted),
                );
            });
        });
        ui.add_space(SPACE_MD);
        ui.label(
            RichText::new("A focused workspace for your data")
                .font(font_page_title())
                .strong()
                .color(self.theme.text_primary),
        );
        ui.add_space(SPACE_XS);
        ui.label(
            RichText::new("Connect a database, open a query, and keep the useful context close.")
                .font(font_body())
                .color(self.theme.text_secondary),
        );
    }

    /// Prompt card. Returns true when the user asked to open the prompt in Query.
    fn draw_welcome_prompt_card(&mut self, ui: &mut egui::Ui) -> bool {
        let mut open_query = false;
        egui::Frame {
            fill: self.theme.surface_elevated,
            inner_margin: egui::Margin::same(CARD_INNER_PAD),
            rounding: egui::Rounding::same(RADIUS_CARD),
            stroke: egui::Stroke::new(STROKE_THIN, self.theme.border_subtle),
            ..Default::default()
        }
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                egui::Frame {
                    fill: self.theme.accent_soft,
                    inner_margin: egui::Margin::same(SPACE_SM),
                    rounding: egui::Rounding::same(RADIUS_MD),
                    stroke: egui::Stroke::NONE,
                    ..Default::default()
                }
                .show(ui, |ui| ui.label(icon_text(Icon::Database, "", self.theme.accent)));
                ui.vertical(|ui| {
                    ui.label(
                        RichText::new("Start with a connection")
                            .font(font_subheading())
                            .strong()
                            .color(self.theme.text_primary),
                    );
                    ui.label(
                        RichText::new("Your schema and query tools will appear here.")
                            .font(font_caption())
                            .color(self.theme.text_muted),
                    );
                });
            });
            ui.add_space(SPACE_MD);
            let prompt_response = input_full_width(
                ui,
                &mut self.welcome_prompt,
                "Paste SQL or describe what you want to inspect…",
                self.theme,
            );
            ui.add_space(SPACE_SM);
            ui.horizontal(|ui| {
                if primary_button_with_icon(ui, Icon::ArrowUp, "Open in Query", self.theme).clicked()
                    || (prompt_response.has_focus() && ui.input(|input| input.key_pressed(egui::Key::Enter)))
                {
                    open_query = true;
                }
                if secondary_button_with_icon(ui, Icon::Database, "New connection", self.theme).clicked() {
                    self.open_new_connection();
                }
                if compact_icon_button(ui, Icon::Search, self.theme)
                    .on_hover_text("Quick Open")
                    .clicked()
                {
                    self.open_palette(PaletteMode::QuickOpen);
                }
            });
        });
        open_query
    }

    /// Secondary "New query" row shown under the prompt card.
    fn draw_welcome_footer(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            if compact_button_with_icon(ui, Icon::FilePlus2, "New query", self.theme).clicked() {
                self.new_query_document();
            }
            if compact_button_with_icon(ui, Icon::Palette, "Component Gallery", self.theme).clicked() {
                self.active_tab = WorkspaceTab::ComponentGallery;
            }
            ui.label(
                RichText::new("or use the activity rail to open Queries and History")
                    .font(font_caption())
                    .color(self.theme.text_muted),
            );
        });
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
            && self.table_data_result.is_none()
            && self.table_data_request.is_none()
            && self.table_data_error.is_none()
        {
            self.request_table_data();
        }

        let schema = self.active_schema().to_owned();
        let connection_name = self
            .active_connection()
            .map(|c| c.name.clone())
            .unwrap_or_else(|| "Connected".to_owned());

        // Top workspace header
        toolbar_frame(self.theme).show(ui, |ui| {
            ui.horizontal(|ui| {
                // Breadcrumb
                ui.label(
                    RichText::new(char::from(Icon::Table2).to_string())
                        .font(egui::FontId::new(14.0, egui::FontFamily::Name("lucide".into())))
                        .color(self.theme.accent),
                );
                ui.label(
                    RichText::new(connection_name)
                        .font(font_caption())
                        .color(self.theme.text_muted),
                );
                ui.label(RichText::new("›").font(font_caption()).color(self.theme.text_muted));
                ui.label(
                    RichText::new(&schema)
                        .font(font_caption())
                        .color(self.theme.text_secondary),
                );
                ui.label(RichText::new("›").font(font_caption()).color(self.theme.text_muted));
                ui.label(
                    RichText::new(&table_name)
                        .font(font_subheading())
                        .strong()
                        .color(self.theme.text_primary),
                );

                if let Some(info) = self.table_info.as_ref() {
                    if let Some(rows) = info.row_count {
                        badge(
                            ui,
                            &format!("{rows} rows"),
                            self.theme.surface_active,
                            self.theme.text_secondary,
                        );
                    }
                }

                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    if compact_button_with_icon(ui, Icon::Bot, "Ask Agent", self.theme)
                        .on_hover_text("Open AI Assistant with table context")
                        .clicked()
                    {
                        self.open_agent_prompt(
                            format!("Explain the `{schema}.{table_name}` table and suggest queries"),
                            ui.ctx(),
                        );
                    }
                    if compact_button_with_icon(ui, Icon::FileCode2, "New Query", self.theme)
                        .on_hover_text("Open SQL Editor for this table")
                        .clicked()
                    {
                        self.query_text = format!("SELECT *\nFROM {schema}.{table_name}\nLIMIT 100;");
                        self.active_tab = WorkspaceTab::Query;
                    }
                    if compact_button_with_icon(ui, Icon::RotateCcw, "Refresh", self.theme)
                        .on_hover_text("Reload table metadata and rows")
                        .clicked()
                    {
                        self.request_table_info();
                        if self.table_view == TableView::Data {
                            self.reset_table_data_page();
                            self.request_table_data();
                        } else if self.table_view == TableView::Ddl {
                            self.table_ddl = None;
                            self.request_table_ddl();
                        }
                    }
                });
            });
        });

        ui.add_space(6.0);

        // Navigation tab bar
        toolbar_frame(self.theme).show(ui, |ui| {
            egui::ScrollArea::horizontal()
                .id_salt("table-workspace-tabs")
                .auto_shrink([false, true])
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        for (view, icon, label) in [
                            (TableView::Data, Icon::Table2, "Data"),
                            (TableView::Structure, Icon::Columns3, "Structure"),
                            (TableView::Indexes, Icon::List, "Indexes"),
                            (TableView::Relations, Icon::ArrowRightLeft, "Foreign Keys"),
                            (TableView::Constraints, Icon::ShieldCheck, "Constraints"),
                            (TableView::Dependencies, Icon::GitBranch, "Dependencies"),
                            (TableView::Ddl, Icon::Code2, "DDL"),
                        ] {
                            let selected = self.table_view == view;
                            let tab = tab_frame(self.theme, selected).show(ui, |ui| {
                                ui.selectable_label(
                                    selected,
                                    icon_text(
                                        icon,
                                        label,
                                        if selected {
                                            self.theme.accent
                                        } else {
                                            self.theme.text_secondary
                                        },
                                    ),
                                )
                            });
                            if tab.inner.clicked() {
                                self.table_view = view;
                            }
                        }
                    });
                });
        });
        ui.add_space(8.0);

        match self.table_view {
            TableView::Structure => self.draw_table_structure(ui),
            TableView::Data => self.draw_table_data(ui, &table_name),
            TableView::Indexes => self.draw_table_metadata_view(ui, TableView::Indexes),
            TableView::Relations => self.draw_table_metadata_view(ui, TableView::Relations),
            TableView::Constraints => self.draw_table_metadata_view(ui, TableView::Constraints),
            TableView::Dependencies => self.draw_table_metadata_view(ui, TableView::Dependencies),
            TableView::Ddl => self.draw_table_ddl(ui, &table_name),
        }
    }

    fn draw_table_metadata_view(&self, ui: &mut egui::Ui, view: TableView) {
        let Some(info) = self.table_info.as_ref() else {
            ui.label(RichText::new("Table structure is still loading").color(self.theme.text_muted));
            return;
        };
        let Some((title, empty, icon)) = Self::table_metadata_section(view) else {
            return;
        };
        let card_width = ui.available_width();
        card_frame(self.theme).show(ui, |ui| {
            ui.set_min_width((card_width - 28.0).max(0.0));
            section_label(ui, title, self.theme);
            ui.add_space(8.0);
            match view {
                TableView::Indexes => self.draw_index_metadata_list(ui, info, icon, empty),
                TableView::Relations | TableView::Dependencies => {
                    self.draw_relation_metadata_list(ui, info, icon, empty)
                }
                TableView::Constraints => self.draw_constraint_metadata_list(ui, info, icon, empty),
                _ => {}
            }
        });
    }

    /// Title, empty-state copy and icon for the read-only metadata tabs.
    fn table_metadata_section(view: TableView) -> Option<(&'static str, &'static str, Icon)> {
        match view {
            TableView::Indexes => Some(("INDEXES", "No indexes", Icon::List)),
            TableView::Relations => Some(("FOREIGN KEYS", "No foreign keys", Icon::ArrowRightLeft)),
            TableView::Constraints => Some((
                "CONSTRAINTS",
                "No explicit constraints in the current metadata",
                Icon::ShieldCheck,
            )),
            TableView::Dependencies => Some((
                "DEPENDENCIES",
                "No dependency edges in the current metadata",
                Icon::GitBranch,
            )),
            _ => None,
        }
    }

    fn draw_index_metadata_list(&self, ui: &mut egui::Ui, info: &UiTableInfo, icon: Icon, empty: &str) {
        if info.indexes.is_empty() {
            empty_state(ui, icon, empty, "This table has no index metadata yet.", self.theme);
            return;
        }
        let cols = [
            crate::components::table::TableColumn::new("Index Name").width(240.0),
            crate::components::table::TableColumn::new("Columns"),
            crate::components::table::TableColumn::fixed("Type", 120.0),
        ];
        crate::components::table::Table::new(&cols, self.theme)
            .row_height(34.0)
            .show(
                ui,
                info.indexes.len(),
                |_| false,
                |_| {},
                |_| {},
                |_| {},
                |ui, row_idx, col_idx| {
                    let index = &info.indexes[row_idx];
                    match col_idx {
                        0 => {
                            ui.horizontal(|ui| {
                                ui.label(icon_text(
                                    if index.unique { Icon::BadgeCheck } else { Icon::List },
                                    "",
                                    self.theme.accent,
                                ));
                                ui.label(RichText::new(&index.name).strong().color(self.theme.text_primary));
                            });
                        }
                        1 => {
                            ui.label(
                                RichText::new(index.columns.join(", "))
                                    .monospace()
                                    .color(self.theme.text_secondary),
                            );
                        }
                        2 => {
                            if index.unique {
                                crate::components::badge::Badge::new("UNIQUE", self.theme)
                                    .variant(crate::components::badge::BadgeVariant::Default)
                                    .compact(true)
                                    .show(ui);
                            } else {
                                crate::components::badge::Badge::new("INDEX", self.theme)
                                    .variant(crate::components::badge::BadgeVariant::Secondary)
                                    .compact(true)
                                    .show(ui);
                            }
                        }
                        _ => {}
                    }
                },
            );
    }

    fn draw_relation_metadata_list(&self, ui: &mut egui::Ui, info: &UiTableInfo, icon: Icon, empty: &str) {
        if info.foreign_keys.is_empty() {
            empty_state(
                ui,
                icon,
                empty,
                "Relationships will appear here when they are defined.",
                self.theme,
            );
            return;
        }
        let cols = [
            crate::components::table::TableColumn::new("Constraint Name").width(220.0),
            crate::components::table::TableColumn::new("From Columns").width(180.0),
            crate::components::table::TableColumn::new("Target Table").width(200.0),
            crate::components::table::TableColumn::new("Target Columns"),
        ];
        crate::components::table::Table::new(&cols, self.theme)
            .row_height(34.0)
            .show(
                ui,
                info.foreign_keys.len(),
                |_| false,
                |_| {},
                |_| {},
                |_| {},
                |ui, row_idx, col_idx| {
                    let relation = &info.foreign_keys[row_idx];
                    match col_idx {
                        0 => {
                            ui.horizontal(|ui| {
                                ui.label(icon_text(Icon::ArrowRightLeft, "", self.theme.accent));
                                ui.label(RichText::new(&relation.name).strong().color(self.theme.text_primary));
                            });
                        }
                        1 => {
                            ui.label(
                                RichText::new(relation.from_columns.join(", "))
                                    .monospace()
                                    .color(self.theme.text_secondary),
                            );
                        }
                        2 => {
                            ui.label(
                                RichText::new(format!("{}.{}", relation.to_schema, relation.to_table))
                                    .strong()
                                    .color(self.theme.text_primary),
                            );
                        }
                        3 => {
                            ui.label(
                                RichText::new(relation.to_columns.join(", "))
                                    .monospace()
                                    .color(self.theme.text_secondary),
                            );
                        }
                        _ => {}
                    }
                },
            );
    }

    fn draw_constraint_metadata_list(&self, ui: &mut egui::Ui, info: &UiTableInfo, icon: Icon, empty: &str) {
        let mut rules: Vec<(&'static str, String, Icon, Color32)> = Vec::new();
        if let Some(primary_key) = &info.primary_key {
            rules.push((
                "PRIMARY KEY",
                primary_key.join(", "),
                Icon::KeyRound,
                self.theme.warning,
            ));
        }
        for column in &info.columns {
            if !column.nullable {
                rules.push(("NOT NULL", column.name.clone(), Icon::ShieldCheck, self.theme.success));
            }
        }
        if rules.is_empty() {
            empty_state(
                ui,
                icon,
                empty,
                "No primary-key or NOT NULL rules were found.",
                self.theme,
            );
            return;
        }

        let cols = [
            crate::components::table::TableColumn::fixed("Constraint Type", 160.0),
            crate::components::table::TableColumn::new("Target Columns / Expression"),
        ];
        crate::components::table::Table::new(&cols, self.theme)
            .row_height(34.0)
            .show(
                ui,
                rules.len(),
                |_| false,
                |_| {},
                |_| {},
                |_| {},
                |ui, row_idx, col_idx| {
                    let rule = &rules[row_idx];
                    match col_idx {
                        0 => {
                            ui.horizontal(|ui| {
                                ui.label(icon_text(rule.2, "", rule.3));
                                ui.label(RichText::new(rule.0).strong().color(self.theme.text_primary));
                            });
                        }
                        1 => {
                            ui.label(RichText::new(&rule.1).monospace().color(self.theme.text_secondary));
                        }
                        _ => {}
                    }
                },
            );
    }

    fn draw_table_structure(&mut self, ui: &mut egui::Ui) {
        let Some(info) = self.table_info.clone() else {
            self.draw_table_structure_placeholder(ui);
            return;
        };

        self.draw_table_structure_summary(ui, &info);
        ui.add_space(8.0);
        self.draw_table_structure_columns(ui, &info);
        ui.add_space(8.0);
        self.draw_table_structure_relations(ui, &info);
    }

    /// Loading / failure placeholder shown while the column metadata is in flight.
    fn draw_table_structure_placeholder(&self, ui: &mut egui::Ui) {
        grid_frame(self.theme).show(ui, |ui| {
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
    }

    /// Count badges and metric chips summarising the table above the column grid.
    fn draw_table_structure_summary(&self, ui: &mut egui::Ui, info: &UiTableInfo) {
        toolbar_frame(self.theme).show(ui, |ui| {
            ui.horizontal_wrapped(|ui| {
                section_label(ui, "METRICS", self.theme);
                crate::components::badge::Badge::new(&format!("{} columns", info.columns.len()), self.theme)
                    .variant(crate::components::badge::BadgeVariant::Default)
                    .show(ui);
                if let Some(pk) = &info.primary_key {
                    crate::components::badge::Badge::new(&format!("PK: {}", pk.join(", ")), self.theme)
                        .variant(crate::components::badge::BadgeVariant::Warning)
                        .show(ui);
                }
                crate::components::badge::Badge::new(&format!("{} indexes", info.indexes.len()), self.theme)
                    .variant(crate::components::badge::BadgeVariant::Secondary)
                    .show(ui);
                crate::components::badge::Badge::new(&format!("{} foreign keys", info.foreign_keys.len()), self.theme)
                    .variant(crate::components::badge::BadgeVariant::Secondary)
                    .show(ui);
                if let Some(row_count) = info.row_count {
                    crate::components::badge::Badge::new(&format!("{row_count} rows"), self.theme)
                        .variant(crate::components::badge::BadgeVariant::Secondary)
                        .show(ui);
                }
            });
        });
    }

    /// Column list using the common Table component with search filter.
    fn draw_table_structure_columns(&mut self, ui: &mut egui::Ui, info: &UiTableInfo) {
        card_frame(self.theme).show(ui, |ui| {
            ui.set_min_width(ui.available_width());
            ui.horizontal(|ui| {
                section_label(ui, "COLUMNS", self.theme);
                ui.add_space(8.0);
                input(
                    ui,
                    &mut self.table_structure_search,
                    "Filter columns…",
                    220.0,
                    self.theme,
                );
                if !self.table_structure_search.is_empty()
                    && compact_icon_button(ui, Icon::X, self.theme)
                        .on_hover_text("Clear filter")
                        .clicked()
                {
                    self.table_structure_search.clear();
                }
            });
            ui.add_space(8.0);

            let filter_lower = self.table_structure_search.trim().to_lowercase();
            let matching_columns: Vec<_> = info
                .columns
                .iter()
                .filter(|c| {
                    if filter_lower.is_empty() {
                        true
                    } else {
                        c.name.to_lowercase().contains(&filter_lower)
                            || c.data_type.to_lowercase().contains(&filter_lower)
                    }
                })
                .collect();

            if matching_columns.is_empty() {
                empty_state(
                    ui,
                    Icon::Columns3,
                    "No columns match filter",
                    "Try typing a different column name or type.",
                    self.theme,
                );
            } else {
                let columns = [
                    crate::components::table::TableColumn::new("Column Name").width(220.0),
                    crate::components::table::TableColumn::new("Data Type").width(180.0),
                    crate::components::table::TableColumn::fixed("Nullable", 110.0),
                    crate::components::table::TableColumn::fixed("Key", 90.0),
                    crate::components::table::TableColumn::new("Default Expression"),
                ];

                crate::components::table::Table::new(&columns, self.theme)
                    .row_height(36.0)
                    .show(
                        ui,
                        matching_columns.len(),
                        |_| false,
                        |_| {},
                        |_| {},
                        |_| {},
                        |ui, row_idx, col_idx| {
                            let column = matching_columns[row_idx];
                            let is_fk = info
                                .foreign_keys
                                .iter()
                                .any(|fk| fk.from_columns.contains(&column.name));
                            match col_idx {
                                0 => {
                                    let (icon, color) = if column.is_primary_key {
                                        (Icon::Key, self.theme.warning)
                                    } else if is_fk {
                                        (Icon::ArrowRightLeft, self.theme.accent)
                                    } else {
                                        (Icon::Columns3, self.theme.text_muted)
                                    };
                                    ui.horizontal(|ui| {
                                        ui.label(
                                            RichText::new(char::from(icon).to_string())
                                                .font(egui::FontId::new(12.0, egui::FontFamily::Name("lucide".into())))
                                                .color(color),
                                        );
                                        ui.add_space(4.0);
                                        ui.label(
                                            RichText::new(&column.name)
                                                .font(font_ui_label())
                                                .strong()
                                                .color(self.theme.text_primary),
                                        );
                                    });
                                }
                                1 => {
                                    ui.label(
                                        RichText::new(&column.data_type)
                                            .monospace()
                                            .color(self.theme.text_secondary),
                                    );
                                }
                                2 => {
                                    if column.nullable {
                                        crate::components::badge::Badge::new("NULL", self.theme)
                                            .variant(crate::components::badge::BadgeVariant::Secondary)
                                            .compact(true)
                                            .show(ui);
                                    } else {
                                        crate::components::badge::Badge::new("NOT NULL", self.theme)
                                            .variant(crate::components::badge::BadgeVariant::Outline)
                                            .compact(true)
                                            .show(ui);
                                    }
                                }
                                3 => {
                                    if column.is_primary_key {
                                        crate::components::badge::Badge::new("PK", self.theme)
                                            .variant(crate::components::badge::BadgeVariant::Warning)
                                            .compact(true)
                                            .show(ui);
                                    } else if is_fk {
                                        crate::components::badge::Badge::new("FK", self.theme)
                                            .variant(crate::components::badge::BadgeVariant::Default)
                                            .compact(true)
                                            .show(ui);
                                    } else {
                                        ui.label(RichText::new("—").font(font_caption()).color(self.theme.text_muted));
                                    }
                                }
                                4 => {
                                    ui.label(
                                        RichText::new(column.default.as_deref().unwrap_or("—"))
                                            .font(font_caption())
                                            .color(self.theme.text_secondary),
                                    );
                                }
                                _ => {}
                            }
                        },
                    );
            }
        });
    }

    /// Side-by-side index and foreign-key cards using common Table components.
    fn draw_table_structure_relations(&self, ui: &mut egui::Ui, info: &UiTableInfo) {
        ui.columns(2, |columns| {
            card_frame(self.theme).show(&mut columns[0], |ui| {
                section_label(ui, "INDEXES", self.theme);
                ui.add_space(6.0);
                if info.indexes.is_empty() {
                    ui.label(RichText::new("No indexes defined").small().color(self.theme.text_muted));
                } else {
                    let cols = [
                        crate::components::table::TableColumn::new("Name").width(140.0),
                        crate::components::table::TableColumn::new("Columns"),
                    ];
                    crate::components::table::Table::new(&cols, self.theme)
                        .row_height(32.0)
                        .show(
                            ui,
                            info.indexes.len(),
                            |_| false,
                            |_| {},
                            |_| {},
                            |_| {},
                            |ui, row_idx, col_idx| {
                                let index = &info.indexes[row_idx];
                                match col_idx {
                                    0 => {
                                        ui.horizontal(|ui| {
                                            ui.label(icon_text(
                                                if index.unique { Icon::BadgeCheck } else { Icon::List },
                                                "",
                                                self.theme.accent,
                                            ));
                                            ui.label(
                                                RichText::new(&index.name).strong().color(self.theme.text_primary),
                                            );
                                        });
                                    }
                                    1 => {
                                        ui.label(
                                            RichText::new(index.columns.join(", "))
                                                .small()
                                                .color(self.theme.text_secondary),
                                        );
                                    }
                                    _ => {}
                                }
                            },
                        );
                }
            });
            card_frame(self.theme).show(&mut columns[1], |ui| {
                section_label(ui, "FOREIGN KEYS", self.theme);
                ui.add_space(6.0);
                if info.foreign_keys.is_empty() {
                    ui.label(
                        RichText::new("No foreign keys defined")
                            .small()
                            .color(self.theme.text_muted),
                    );
                } else {
                    let cols = [
                        crate::components::table::TableColumn::new("Name").width(140.0),
                        crate::components::table::TableColumn::new("Target"),
                    ];
                    crate::components::table::Table::new(&cols, self.theme)
                        .row_height(32.0)
                        .show(
                            ui,
                            info.foreign_keys.len(),
                            |_| false,
                            |_| {},
                            |_| {},
                            |_| {},
                            |ui, row_idx, col_idx| {
                                let fk = &info.foreign_keys[row_idx];
                                match col_idx {
                                    0 => {
                                        ui.horizontal(|ui| {
                                            ui.label(icon_text(Icon::ArrowRightLeft, "", self.theme.accent));
                                            ui.label(RichText::new(&fk.name).strong().color(self.theme.text_primary));
                                        });
                                    }
                                    1 => {
                                        ui.label(
                                            RichText::new(format!(
                                                "{} → {}.{}",
                                                fk.from_columns.join(", "),
                                                fk.to_schema,
                                                fk.to_table,
                                            ))
                                            .small()
                                            .color(self.theme.text_secondary),
                                        );
                                    }
                                    _ => {}
                                }
                            },
                        );
                }
            });
        });
    }
}
