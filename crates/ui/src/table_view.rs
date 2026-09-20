use super::*;

impl DbProApp {
    pub(super) fn draw_table_workspace(&mut self, ui: &mut egui::Ui) {
        let Some(table_name) = self.schema.explorer.selected_table.clone() else {
            self.activate_welcome_tab();
            return;
        };
        self.ensure_table_workspace_requests();

        let schema = self.active_schema().to_owned();
        let connection_name = self
            .active_connection()
            .map(|c| c.name.clone())
            .unwrap_or_else(|| "Connected".to_owned());

        let surface_context = table_workspace_surface_view::TableWorkspaceSurfaceContext {
            theme: self.theme,
            connection_name: &connection_name,
            schema: &schema,
            table_name: &table_name,
            active_view: self.table.state.table_view,
            row_count: self.table.state.table_info.as_ref().and_then(|info| info.row_count),
        };
        let surface_actions = surface_context.draw(ui);
        for action in surface_actions {
            self.apply_table_workspace_surface_action(action, &schema, &table_name, ui.ctx());
        }
        ui.add_space(8.0);
        self.draw_table_workspace_content(ui, &table_name);
    }

    fn ensure_table_workspace_requests(&mut self) {
        if self.table.state.table_view == TableView::Ddl
            && self.table.state.table_ddl.is_none()
            && self.table.state.table_ddl_request.is_none()
        {
            self.request_table_ddl();
        }
        if (self.table.state.table_view == TableView::Data || self.table.state.table_view == TableView::Profile)
            && self.table.data_query.result.is_none()
            && self.table.data_query.request.is_none()
            && self.table.data_query.error.is_none()
        {
            self.request_table_data();
        }
    }

    fn draw_table_workspace_content(&mut self, ui: &mut egui::Ui, table_name: &str) {
        match self.table.state.table_view {
            TableView::Data => self.draw_table_data(ui, table_name),
            TableView::Profile => self.draw_column_profile_pane(ui, self.table.data_query.result.as_ref()),
            view => self.draw_table_metadata_view(ui, table_name, view),
        }
    }

    fn draw_table_metadata_view(&mut self, ui: &mut egui::Ui, table_name: &str, view: TableView) {
        match view {
            TableView::Structure => self.draw_scrollable_table_pane(ui, "table-structure-scroll", |app, ui| {
                app.draw_table_structure_view(ui);
            }),
            TableView::Indexes => self.draw_scrollable_table_pane(ui, "table-indexes-scroll", |app, ui| {
                app.draw_table_indexes_view(ui);
            }),
            TableView::Relations => self.draw_scrollable_table_pane(ui, "table-relations-scroll", |app, ui| {
                app.draw_table_relations_view(ui);
            }),
            TableView::Constraints => self.draw_scrollable_table_pane(ui, "table-constraints-scroll", |app, ui| {
                app.draw_table_constraints_view(ui);
            }),
            TableView::Dependencies => self.draw_scrollable_table_pane(ui, "table-dependencies-scroll", |app, ui| {
                app.draw_table_dependencies_view(ui);
            }),
            TableView::Ddl => self.draw_scrollable_table_pane(ui, "table-ddl-scroll", |app, ui| {
                app.draw_table_ddl_view(ui, table_name);
            }),
            TableView::Data | TableView::Profile => {
                unreachable!("data and profile are rendered by the workspace router")
            }
        }
    }

    fn draw_scrollable_table_pane(
        &mut self,
        ui: &mut egui::Ui,
        id: &'static str,
        draw: impl FnOnce(&mut Self, &mut egui::Ui),
    ) {
        egui::ScrollArea::vertical()
            .id_salt(id)
            .auto_shrink([false, false])
            .show(ui, |ui| draw(self, ui));
    }

    fn apply_table_workspace_surface_action(
        &mut self,
        action: table_workspace_surface_view::TableWorkspaceSurfaceAction,
        schema: &str,
        table_name: &str,
        context: &egui::Context,
    ) {
        use table_workspace_surface_view::TableWorkspaceSurfaceAction as Action;

        match action {
            Action::AskAgent => self.open_agent_prompt(
                format!("Explain the `{schema}.{table_name}` table and suggest queries"),
                context,
            ),
            Action::NewQuery => {
                self.set_active_query_text(format!("SELECT *\nFROM {schema}.{table_name}\nLIMIT 100;"));
                self.workspace.active_tab = WorkspaceTab::Query;
            }
            Action::Refresh => {
                self.request_table_info();
                match self.table.state.table_view {
                    TableView::Data => {
                        self.reset_table_data_page();
                        self.request_table_data();
                    }
                    TableView::Ddl => {
                        self.table.state.table_ddl = None;
                        self.request_table_ddl();
                    }
                    _ => {}
                }
            }
            Action::SelectView(view) => self.table.state.table_view = view,
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
        self.draw_profile_grid(ui, &profiles);
    }

    fn draw_profile_grid(&self, ui: &mut egui::Ui, profiles: &[ColumnProfile]) {
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
                    for profile in profiles {
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
                let failed = self.table.state.table_info_error.as_deref();
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
