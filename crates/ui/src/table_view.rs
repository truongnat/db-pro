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
            self.activate_welcome_tab();
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
            TableView::Data => self.draw_table_data(ui, &table_name),
            TableView::Structure => self.draw_table_structure_view(ui),
            TableView::Indexes => self.draw_table_indexes_view(ui),
            TableView::Relations => self.draw_table_relations_view(ui),
            TableView::Constraints => self.draw_table_constraints_view(ui),
            TableView::Dependencies => self.draw_table_dependencies_view(ui),
            TableView::Ddl => self.draw_table_ddl_view(ui, &table_name),
        }
    }

    /// Loading / failure placeholder shown while the column metadata is in flight.
    pub(super) fn draw_table_structure_placeholder(&self, ui: &mut egui::Ui) {
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
}
