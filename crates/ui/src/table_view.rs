use super::*;

impl DbProApp {
    pub(super) fn draw_table_workspace(&mut self, ui: &mut egui::Ui) {
        let Some(table_name) = self.selected_table.clone() else {
            self.activate_welcome_tab();
            return;
        };
        if self.table_view == TableView::Ddl && self.table_ddl.is_none() && self.table_ddl_request.is_none() {
            self.request_table_ddl();
        }
        if (self.table_view == TableView::Data || self.table_view == TableView::Profile)
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
                ui.label(
                    RichText::new(char::from(Icon::ChevronRight).to_string())
                        .font(egui::FontId::new(12.0, egui::FontFamily::Name("lucide".into())))
                        .color(self.theme.text_muted),
                );
                ui.label(
                    RichText::new(&schema)
                        .font(font_caption())
                        .color(self.theme.text_secondary),
                );
                ui.label(
                    RichText::new(char::from(Icon::ChevronRight).to_string())
                        .font(egui::FontId::new(12.0, egui::FontFamily::Name("lucide".into())))
                        .color(self.theme.text_muted),
                );
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
                    if Button::new(self.theme)
                        .icon(Icon::Bot)
                        .text("Ask Agent")
                        .variant(ButtonVariant::Ghost)
                        .size(ButtonSize::Sm)
                        .tooltip("Open AI Assistant with table context")
                        .show(ui)
                        .clicked()
                    {
                        self.open_agent_prompt(
                            format!("Explain the `{schema}.{table_name}` table and suggest queries"),
                            ui.ctx(),
                        );
                    }
                    if Button::new(self.theme)
                        .icon(Icon::FileCode2)
                        .text("New Query")
                        .variant(ButtonVariant::Ghost)
                        .size(ButtonSize::Sm)
                        .tooltip("Open SQL Editor for this table")
                        .show(ui)
                        .clicked()
                    {
                        self.set_active_query_text(format!("SELECT *\nFROM {schema}.{table_name}\nLIMIT 100;"));
                        self.active_tab = WorkspaceTab::Query;
                    }
                    if Button::new(self.theme)
                        .icon(Icon::RotateCcw)
                        .text("Refresh")
                        .variant(ButtonVariant::Ghost)
                        .size(ButtonSize::Sm)
                        .tooltip("Reload table metadata and rows")
                        .show(ui)
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
                            (TableView::Profile, Icon::ChartColumn, "Profile"),
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
            TableView::Profile => self.draw_column_profile_pane(ui, self.table_data_result.as_ref()),
            TableView::Structure => {
                egui::ScrollArea::vertical()
                    .id_salt("table-structure-scroll")
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        self.draw_table_structure_view(ui);
                    });
            }
            TableView::Indexes => {
                egui::ScrollArea::vertical()
                    .id_salt("table-indexes-scroll")
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        self.draw_table_indexes_view(ui);
                    });
            }
            TableView::Relations => {
                egui::ScrollArea::vertical()
                    .id_salt("table-relations-scroll")
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        self.draw_table_relations_view(ui);
                    });
            }
            TableView::Constraints => {
                egui::ScrollArea::vertical()
                    .id_salt("table-constraints-scroll")
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        self.draw_table_constraints_view(ui);
                    });
            }
            TableView::Dependencies => {
                egui::ScrollArea::vertical()
                    .id_salt("table-dependencies-scroll")
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        self.draw_table_dependencies_view(ui);
                    });
            }
            TableView::Ddl => {
                egui::ScrollArea::vertical()
                    .id_salt("table-ddl-scroll")
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        self.draw_table_ddl_view(ui, &table_name);
                    });
            }
        }
    }

    /// Bounded column profiling over the currently loaded page of rows (#227).
    pub(super) fn draw_column_profile_pane(&self, ui: &mut egui::Ui, result: Option<&UiQueryResult>) {
        let Some(result) = result else {
            empty_state(
                ui,
                Icon::ChartColumn,
                "No rows loaded",
                "Open the Data tab or wait for the current page to load, then return to Profile.",
                self.theme,
            );
            return;
        };
        if result.columns.is_empty() {
            empty_state(
                ui,
                Icon::ChartColumn,
                "No columns",
                "This result has no columns to profile.",
                self.theme,
            );
            return;
        }

        let profiles = Self::profile_result_columns(result);
        ui.label(
            RichText::new(format!(
                "Profiling {} loaded row(s) · not a full-table scan",
                result.rows.len()
            ))
            .small()
            .color(self.theme.text_muted),
        );
        ui.add_space(8.0);
        egui::ScrollArea::vertical().show(ui, |ui| {
            egui::Grid::new("column_profile_grid")
                .num_columns(6)
                .striped(true)
                .show(ui, |ui| {
                    ui.label(RichText::new("Column").strong());
                    ui.label(RichText::new("Type").strong());
                    ui.label(RichText::new("Nulls").strong());
                    ui.label(RichText::new("Distinct").strong());
                    ui.label(RichText::new("Min").strong());
                    ui.label(RichText::new("Max").strong());
                    ui.end_row();
                    for profile in &profiles {
                        ui.label(&profile.name);
                        ui.label(&profile.data_type);
                        ui.label(format!("{} ({:.0}%)", profile.null_count, profile.null_rate * 100.0));
                        ui.label(format!(
                            "{} ({:.0}%)",
                            profile.distinct_count,
                            profile.distinct_rate * 100.0
                        ));
                        ui.label(profile.min.as_deref().unwrap_or("—"));
                        ui.label(profile.max.as_deref().unwrap_or("—"));
                        ui.end_row();
                    }
                });
        });
    }

    pub(crate) fn profile_result_columns(result: &UiQueryResult) -> Vec<ColumnProfile> {
        let row_count = result.rows.len().max(1) as f64;
        result
            .columns
            .iter()
            .enumerate()
            .map(|(col_idx, column)| {
                let mut null_count = 0usize;
                let mut values = Vec::new();
                for row in &result.rows {
                    match row.get(col_idx) {
                        None | Some(UiCell::Null) => null_count += 1,
                        Some(cell) => values.push(Self::cell_profile_text(cell)),
                    }
                }
                let distinct = values.iter().cloned().collect::<std::collections::BTreeSet<_>>();
                let min = values.iter().min().cloned();
                let max = values.iter().max().cloned();
                ColumnProfile {
                    name: column.name.clone(),
                    data_type: column.data_type.clone(),
                    null_count,
                    null_rate: null_count as f64 / row_count,
                    distinct_count: distinct.len(),
                    distinct_rate: distinct.len() as f64 / row_count,
                    min,
                    max,
                }
            })
            .collect()
    }

    fn cell_profile_text(cell: &UiCell) -> String {
        match cell {
            UiCell::Null => String::new(),
            UiCell::Boolean(b) => b.to_string(),
            UiCell::Number(n) | UiCell::Text(n) | UiCell::Json(n) | UiCell::Bytes(n) => n.clone(),
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
