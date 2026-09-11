use super::*;

impl DbProApp {
    pub(super) fn draw_topbar(&mut self, ctx: &egui::Context) {
        let connection_name = self.active_connection_name().to_owned();
        let driver = self.active_driver().to_owned();
        let has_connection = self.active_connection_id.is_some();
        let modifier = Self::primary_modifier_label();
        TopBottomPanel::top("topbar")
            .exact_height(48.0)
            .frame(panel_frame(self.theme))
            .show(ctx, |ui| {
                ui.horizontal_centered(|ui| {
                    ui.add_space(12.0);
                    ui.label(RichText::new("DB").strong().color(self.theme.accent));
                    ui.label(RichText::new("PRO").strong().color(self.theme.text_primary));
                    ui.separator();
                    if has_connection {
                        ui.label(icon_text(Icon::Database, &connection_name, self.theme.text_secondary));
                        ui.label(RichText::new(driver).small().color(self.theme.text_muted));
                    } else {
                        ui.label(RichText::new("No connection").color(self.theme.text_muted));
                    }
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        if ghost_button_with_icon(ui, Icon::Bot, "Agent", self.theme).clicked() {
                            self.set_agent_open(!self.agent_open, ctx);
                        }
                        if ghost_button_with_icon(ui, Icon::Search, "Quick Open", self.theme).clicked() {
                            self.open_palette(PaletteMode::QuickOpen);
                        }
                        if compact_icon_button(ui, Icon::Command, self.theme)
                            .on_hover_text(format!("Command Palette ({modifier}⇧P)"))
                            .clicked()
                        {
                            self.open_palette(PaletteMode::Commands);
                        }
                        ui.label(
                            RichText::new("v0.1 native preview")
                                .small()
                                .color(self.theme.text_muted),
                        );
                    });
                });
            });
    }

    pub(super) fn draw_statusbar(&self, ctx: &egui::Context) {
        let (icon, color, label) = self.statusbar_state();
        let show_runtime_message = self.has_runtime_error();
        TopBottomPanel::bottom("statusbar")
            .exact_height(26.0)
            .frame(panel_frame(self.theme))
            .show(ctx, |ui| {
                ui.horizontal_centered(|ui| {
                    ui.add_space(12.0);
                    ui.label(icon_text(icon, "", color));
                    ui.label(RichText::new(label).small().color(self.theme.text_secondary));
                    ui.separator();
                    ui.label(RichText::new(self.active_driver()).small().color(self.theme.text_muted));
                    if show_runtime_message {
                        ui.separator();
                        ui.add_sized(
                            [260.0, 18.0],
                            egui::Label::new(
                                RichText::new(self.runtime_message.as_str())
                                    .small()
                                    .color(self.theme.danger),
                            ),
                        );
                    }
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        ui.label(RichText::new("UTF-8").small().color(self.theme.text_muted));
                        ui.label(RichText::new("Ln 1, Col 1").small().color(self.theme.text_muted));
                    });
                });
            });
    }

    pub(super) fn draw_activity_bar(&mut self, ctx: &egui::Context) {
        egui::SidePanel::left("activity_bar")
            .resizable(false)
            .exact_width(54.0)
            .frame(activity_bar_frame(self.theme))
            .show(ctx, |ui| {
                ui.vertical_centered(|ui| {
                    ui.add_space(16.0);
                    for (activity, icon, hint) in [
                        (Activity::Explorer, Icon::Database, "Explorer"),
                        (Activity::History, Icon::History, "History"),
                        (Activity::Settings, Icon::Settings2, "Settings"),
                        (Activity::Diagram, Icon::ArrowRightLeft, "ER diagram"),
                    ] {
                        let active = self.activity == activity;
                        let response = icon_button(ui, icon, active, self.theme);
                        if response.on_hover_text(hint).clicked() {
                            self.activity = activity;
                            self.sidebar_open = true;
                            if activity == Activity::Diagram {
                                self.active_tab = WorkspaceTab::Diagram;
                            }
                        }
                        ui.add_space(4.0);
                    }
                });
            });
    }

    pub(super) fn draw_sidebar(&mut self, ctx: &egui::Context) {
        egui::SidePanel::left("sidebar")
            .default_width(260.0)
            .min_width(220.0)
            .max_width(380.0)
            .frame(sidebar_frame(self.theme))
            .show(ctx, |ui| {
                ui.add_space(12.0);
                ui.horizontal(|ui| {
                    section_label(
                        ui,
                        match self.activity {
                            Activity::Explorer => "EXPLORER",
                            Activity::History => "QUERY HISTORY",
                            Activity::Settings => "SETTINGS",
                            Activity::Diagram => "ER DIAGRAM",
                        },
                        self.theme,
                    );
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        if compact_icon_button(ui, Icon::PanelLeftClose, self.theme)
                            .on_hover_text(format!("Hide sidebar ({}B)", Self::primary_modifier_label()))
                            .clicked()
                        {
                            self.sidebar_open = false;
                        }
                    });
                });
                ui.add_space(12.0);

                match self.activity {
                    Activity::Explorer => self.draw_explorer(ui),
                    Activity::History => self.draw_history(ui),
                    Activity::Settings => self.draw_settings(ui),
                    Activity::Diagram => self.draw_diagram_sidebar(ui),
                }
            });
    }

    fn draw_diagram_sidebar(&mut self, ui: &mut egui::Ui) {
        card_frame(self.theme).show(ui, |ui| {
            section_label(ui, "SCHEMA MAP", self.theme);
            ui.add_space(8.0);
            ui.label(RichText::new("Tables and foreign-key relationships").color(self.theme.text_primary));
            ui.add_space(4.0);
            ui.label(
                RichText::new(format!(
                    "{} tables · {} relationships",
                    self.schema.table_details.len(),
                    self.schema
                        .table_details
                        .iter()
                        .map(|table| table.foreign_keys.len())
                        .sum::<usize>()
                ))
                .small()
                .color(self.theme.text_muted),
            );
        });
        ui.add_space(10.0);
        card_frame(self.theme).show(ui, |ui| {
            section_label(ui, "NAVIGATION", self.theme);
            ui.add_space(8.0);
            ui.label(
                RichText::new("Drag the canvas to pan. Use the controls above the map to zoom or fit the schema.")
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

    fn draw_explorer(&mut self, ui: &mut egui::Ui) {
        self.draw_explorer_connections(ui);
        self.draw_explorer_schema_feedback(ui);
        self.draw_explorer_tables(ui);
        self.draw_explorer_views(ui);
        self.draw_explorer_triggers(ui);
        self.draw_explorer_functions(ui);
        self.draw_explorer_footer(ui);
    }

    fn draw_explorer_connections(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.label(icon_text(Icon::ChevronDown, "", self.theme.text_muted));
            ui.label(RichText::new(self.active_connection_name()).strong());
            ui.label(RichText::new(self.active_driver()).small().color(self.theme.accent));
        });
        ui.add_space(8.0);
        if self.connections.is_empty() {
            ui.label(RichText::new("No saved connections").color(self.theme.text_muted));
            ui.label(
                RichText::new("Connect a database to load its schema here.")
                    .small()
                    .color(self.theme.text_muted),
            );
            ui.add_space(8.0);
            if secondary_button_with_icon(ui, Icon::Plus, "New connection", self.theme).clicked() {
                self.open_new_connection();
            }
        } else {
            for connection in self.connections.clone() {
                let is_active = self.active_connection_id.as_deref() == Some(connection.id.as_str());
                ui.horizontal(|ui| {
                    let (connection_state_icon, connection_state_color) = self.connection_indicator(&connection);
                    ui.label(icon_text(connection_state_icon, "", connection_state_color));
                    let connection_button = ui.add(
                        egui::Button::new(
                            RichText::new(connection.name.as_str())
                                .strong()
                                .color(self.theme.text_primary),
                        )
                        .fill(if is_active {
                            self.theme.accent_soft
                        } else {
                            Color32::TRANSPARENT
                        })
                        .stroke(egui::Stroke::NONE)
                        .rounding(egui::Rounding::same(5.0)),
                    );
                    if connection_button.clicked() {
                        self.active_connection_id = Some(connection.id.clone());
                        self.selected_table = None;
                        self.selected_schema_object = None;
                        self.table_info = None;
                        self.table_ddl = None;
                        self.table_info_error = None;
                        self.table_ddl_error = None;
                        self.ddl_execute_confirmation = false;
                        self.ddl_execution_request = None;
                        self.table_data_result = None;
                        self.table_data_total_rows = None;
                        self.table_data_offset = 0;
                        self.table_data_filter_column.clear();
                        self.table_data_filter_value.clear();
                        self.table_data_sort_column = None;
                        self.table_data_sort_desc = false;
                        self.table_data_error = None;
                        self.table_info_request = None;
                        self.table_ddl_request = None;
                        self.table_data_request = None;
                        self.table_mutation_request = None;
                        self.selected_cell = None;
                        self.selected_row = None;
                        self.data_editing_cell = None;
                        self.data_edit_value.clear();
                        self.data_delete_confirmation = false;
                        self.table_view = TableView::Structure;
                        self.explorer_search.clear();
                        let request_id = self.task_bridge.next_request_id();
                        self.connected = false;
                        self.pending_connection_request = Some(request_id);
                        self.schema_request = Some(request_id);
                        self.schema_error = None;
                        let _ = self.task_bridge.send(UiCommand::Connect {
                            request_id,
                            connection_id: connection.id.clone(),
                        });
                        self.runtime_message = format!("Connecting to {}…", connection.name);
                    }
                    ui.label(
                        RichText::new(connection.driver.as_str())
                            .small()
                            .color(self.theme.accent),
                    );
                    if is_active && compact_button(ui, "Edit", self.theme).clicked() {
                        self.open_edit_connection(&connection);
                    }
                    if is_active
                        && compact_icon_button(ui, Icon::X, self.theme)
                            .on_hover_text("Delete connection")
                            .clicked()
                    {
                        self.delete_confirmation_id = Some(connection.id.clone());
                    }
                });
            }
        }
        ui.add_space(12.0);
    }
    fn draw_explorer_schema_feedback(&mut self, ui: &mut egui::Ui) {
        let schema_error = self.schema_error.clone();
        if let Some(error) = schema_error.as_deref() {
            card_frame(self.theme).show(ui, |ui| {
                ui.horizontal_wrapped(|ui| {
                    ui.label(icon_text(Icon::TriangleAlert, "Schema load failed", self.theme.danger));
                    ui.label(RichText::new(error).small().color(self.theme.text_secondary));
                    if let Some(connection_id) = self.active_connection_id.clone() {
                        if secondary_button_with_icon(ui, Icon::RotateCcw, "Refresh schema", self.theme).clicked() {
                            self.request_schema_introspection(connection_id, true);
                        }
                    }
                });
            });
            ui.add_space(8.0);
        }
        ui.horizontal(|ui| {
            input(ui, &mut self.explorer_search, "Search tables…", 184.0, self.theme);
            if !self.explorer_search.is_empty() && compact_button(ui, "Clear", self.theme).clicked() {
                self.explorer_search.clear();
            }
        });
        ui.add_space(8.0);
    }
    fn draw_explorer_tables(&mut self, ui: &mut egui::Ui) {
        let schema_name = self.active_schema().to_owned();
        ui.collapsing(icon_text(Icon::Layers, "Schemas", self.theme.text_primary), |ui| {
            let _ = sidebar_item(ui, Icon::Database, &schema_name, false, self.theme);
        });
        let search_query = self.explorer_search.trim().to_ascii_lowercase();
        let total_tables = self.schema.tables.len();
        let (matching_table_count, tables) = filtered_explorer_tables(&self.schema.tables, &search_query);
        ui.collapsing(
            icon_text(
                Icon::Table2,
                &format!("Tables ({} / {})", matching_table_count, total_tables),
                self.theme.text_primary,
            ),
            |ui| {
                for table in &tables {
                    let is_selected = self.selected_table.as_deref() == Some(table.as_str());
                    if sidebar_item(ui, Icon::Table2, table, is_selected, self.theme).clicked() {
                        self.selected_table = Some(table.clone());
                        self.selected_schema_object = None;
                        self.schema_object_view = SchemaObjectView::Definition;
                        self.table_info = None;
                        self.table_ddl = None;
                        self.table_info_error = None;
                        self.table_ddl_error = None;
                        self.ddl_execute_confirmation = false;
                        self.ddl_execution_request = None;
                        self.table_data_result = None;
                        self.table_data_total_rows = None;
                        self.table_data_offset = 0;
                        self.table_data_filter_column.clear();
                        self.table_data_filter_value.clear();
                        self.table_data_sort_column = None;
                        self.table_data_sort_desc = false;
                        self.table_data_error = None;
                        self.table_info_request = None;
                        self.table_ddl_request = None;
                        self.table_data_request = None;
                        self.table_view = TableView::Structure;
                        self.table_mutation_request = None;
                        self.selected_cell = None;
                        self.selected_row = None;
                        self.data_editing_cell = None;
                        self.data_edit_value.clear();
                        self.data_delete_confirmation = false;
                        self.query_text = format!("SELECT *\nFROM {table}\nLIMIT 100;");
                        self.request_table_info();
                        self.active_tab = WorkspaceTab::Table;
                    }
                    if is_selected {
                        if let Some(info) = self.table_info.clone() {
                            ui.indent(("table-sidebar-details", table.as_str()), |ui| {
                                ui.collapsing(
                                    icon_text(
                                        Icon::Columns3,
                                        &format!("Columns ({})", info.columns.len()),
                                        self.theme.text_secondary,
                                    ),
                                    |ui| {
                                        for column in &info.columns {
                                            ui.label(
                                                RichText::new(format!("{} · {}", column.name, column.data_type))
                                                    .small()
                                                    .color(self.theme.text_muted),
                                            );
                                        }
                                    },
                                );
                                ui.collapsing(
                                    icon_text(
                                        Icon::List,
                                        &format!("Indexes ({})", info.indexes.len()),
                                        self.theme.text_secondary,
                                    ),
                                    |ui| {
                                        if info.indexes.is_empty() {
                                            ui.label(RichText::new("No indexes").small().color(self.theme.text_muted));
                                        }
                                        for index in &info.indexes {
                                            ui.label(RichText::new(&index.name).small().color(self.theme.text_muted));
                                        }
                                    },
                                );
                                ui.collapsing(
                                    icon_text(
                                        Icon::ArrowRightLeft,
                                        &format!("Foreign keys ({})", info.foreign_keys.len()),
                                        self.theme.text_secondary,
                                    ),
                                    |ui| {
                                        if info.foreign_keys.is_empty() {
                                            ui.label(
                                                RichText::new("No foreign keys").small().color(self.theme.text_muted),
                                            );
                                        }
                                        for foreign_key in &info.foreign_keys {
                                            ui.label(
                                                RichText::new(&foreign_key.name).small().color(self.theme.text_muted),
                                            );
                                        }
                                    },
                                );
                            });
                        }
                    }
                }
                if matching_table_count > tables.len() {
                    ui.label(
                        RichText::new(format!(
                            "Showing first {} matches; refine the search to see more.",
                            tables.len()
                        ))
                        .small()
                        .color(self.theme.text_muted),
                    );
                }
            },
        );
    }
    fn draw_explorer_views(&mut self, ui: &mut egui::Ui) {
        let views = self.schema.views.clone();
        ui.collapsing(
            icon_text(Icon::Eye, &format!("Views ({})", views.len()), self.theme.text_primary),
            |ui| {
                if views.is_empty() {
                    ui.label(RichText::new("No views").small().color(self.theme.text_muted));
                }
                for view in views.iter().take(100) {
                    let is_selected = matches!(
                        self.selected_schema_object.as_ref(),
                        Some(SchemaObjectSelection::View(selected)) if selected == &view.name
                    );
                    if sidebar_item(ui, Icon::Eye, &view.name, is_selected, self.theme).clicked() {
                        self.selected_schema_object = Some(SchemaObjectSelection::View(view.name.clone()));
                        self.schema_object_view = SchemaObjectView::Definition;
                        self.selected_table = None;
                        self.table_info = None;
                        self.table_ddl = None;
                        self.table_info_error = None;
                        self.table_ddl_error = None;
                        self.ddl_execute_confirmation = false;
                        self.ddl_execution_request = None;
                        self.table_data_result = None;
                        self.table_data_total_rows = None;
                        self.table_data_request = None;
                        self.table_view = TableView::Ddl;
                        self.active_tab = WorkspaceTab::SchemaObject;
                        self.runtime_message = format!("Opened view {}.{}", view.schema, view.name);
                    }
                }
            },
        );
    }
    fn draw_explorer_triggers(&mut self, ui: &mut egui::Ui) {
        let triggers = self.schema.triggers.clone();
        ui.collapsing(
            icon_text(
                Icon::Zap,
                &format!("Triggers ({})", triggers.len()),
                self.theme.text_primary,
            ),
            |ui| {
                if triggers.is_empty() {
                    ui.label(RichText::new("No triggers").small().color(self.theme.text_muted));
                }
                for trigger in triggers.iter().take(100) {
                    let is_selected = matches!(
                        self.selected_schema_object.as_ref(),
                        Some(SchemaObjectSelection::Trigger(selected)) if selected == &trigger.name
                    );
                    let label = format!("{} · {}", trigger.name, trigger.event);
                    if sidebar_item(ui, Icon::Zap, &label, is_selected, self.theme).clicked() {
                        self.selected_schema_object = Some(SchemaObjectSelection::Trigger(trigger.name.clone()));
                        self.schema_object_view = SchemaObjectView::Definition;
                        self.selected_table = None;
                        self.table_info = None;
                        self.table_ddl = None;
                        self.table_info_error = None;
                        self.table_ddl_error = None;
                        self.ddl_execute_confirmation = false;
                        self.ddl_execution_request = None;
                        self.table_data_result = None;
                        self.table_data_total_rows = None;
                        self.table_data_request = None;
                        self.table_view = TableView::Ddl;
                        self.active_tab = WorkspaceTab::SchemaObject;
                        self.runtime_message = format!("Opened trigger {}", trigger.name);
                    }
                }
            },
        );
    }
    fn draw_explorer_functions(&mut self, ui: &mut egui::Ui) {
        let functions = self.schema.functions.clone();
        ui.collapsing(
            icon_text(
                Icon::Code2,
                &format!("Functions ({})", functions.len()),
                self.theme.text_primary,
            ),
            |ui| {
                if functions.is_empty() {
                    ui.label(RichText::new("No functions").small().color(self.theme.text_muted));
                }
                for function in functions.iter().take(100) {
                    let is_selected = matches!(
                        self.selected_schema_object.as_ref(),
                        Some(SchemaObjectSelection::Function(selected)) if selected == &function.name
                    );
                    let label = format!("{} · {}", function.name, function.routine_type);
                    if sidebar_item(ui, Icon::Code2, &label, is_selected, self.theme).clicked() {
                        self.selected_schema_object = Some(SchemaObjectSelection::Function(function.name.clone()));
                        self.schema_object_view = SchemaObjectView::Definition;
                        self.selected_table = None;
                        self.table_info = None;
                        self.table_ddl = None;
                        self.table_info_error = None;
                        self.table_ddl_error = None;
                        self.table_data_result = None;
                        self.table_data_total_rows = None;
                        self.table_data_request = None;
                        self.table_view = TableView::Ddl;
                        self.active_tab = WorkspaceTab::SchemaObject;
                        self.runtime_message = format!("Opened function {}.{}", function.schema, function.name);
                    }
                }
            },
        );
    }
    fn draw_explorer_footer(&mut self, ui: &mut egui::Ui) {
        if self.table_info.is_none() {
            ui.collapsing(
                icon_text(
                    Icon::Columns3,
                    &format!("Columns ({})", self.schema.columns.len()),
                    self.theme.text_primary,
                ),
                |ui| {
                    for column in self.schema.columns.iter().take(100) {
                        ui.label(
                            RichText::new(format!("  {column}"))
                                .small()
                                .color(self.theme.text_muted),
                        );
                    }
                },
            );
        }
        ui.add_space(16.0);
        if primary_button_with_icon(ui, Icon::Plus, "New connection", self.theme).clicked() {
            self.editing_connection_id = None;
            self.connection_draft = UiConnectionDraft::default();
            self.connection_error.clear();
            self.connection_dialog_open = true;
        }
    }

    fn draw_history(&mut self, ui: &mut egui::Ui) {
        ui.label(
            RichText::new("Saved queries")
                .small()
                .strong()
                .color(self.theme.text_muted),
        );
        if self.saved_queries.is_empty() {
            card_frame(self.theme).show(ui, |ui| {
                ui.vertical_centered(|ui| {
                    ui.label(icon_text(Icon::Bookmark, "", self.theme.accent));
                    ui.add_space(6.0);
                    ui.label(RichText::new("No saved queries yet").strong());
                    ui.label(
                        RichText::new("Save a query to keep it close at hand.")
                            .small()
                            .color(self.theme.text_muted),
                    );
                    ui.add_space(8.0);
                    if compact_button_with_icon(ui, Icon::Plus, "New query", self.theme).clicked() {
                        self.new_query_document();
                    }
                });
            });
        } else {
            let saved = self.saved_queries.clone();
            let mut groups: Vec<(String, Vec<UiSavedQuerySummary>)> = Vec::new();
            for query in saved {
                let folder = query.folder.clone().unwrap_or_else(|| "Unfiled".to_owned());
                if let Some((_, queries)) = groups.iter_mut().find(|(name, _)| name == &folder) {
                    queries.push(query);
                } else {
                    groups.push((folder, vec![query]));
                }
            }

            for (folder, queries) in groups {
                let folder_id = self
                    .query_folders
                    .iter()
                    .find(|item| item.name == folder)
                    .map(|item| item.id.clone());
                let mut delete_requested = false;
                let header = egui::collapsing_header::CollapsingState::load_with_default_open(
                    ui.ctx(),
                    ui.make_persistent_id(("saved-query-folder", folder.as_str())),
                    true,
                )
                .show_header(ui, |ui| {
                    ui.label(icon_text(Icon::FolderOpen, &folder, self.theme.text_primary));
                    ui.label(
                        RichText::new(format!("{} queries", queries.len()))
                            .small()
                            .color(self.theme.text_muted),
                    );
                    if folder_id.is_some() {
                        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                            if compact_icon_button(ui, Icon::X, self.theme)
                                .on_hover_text("Delete folder")
                                .clicked()
                            {
                                delete_requested = true;
                            }
                        });
                    }
                });
                header.body(|ui| {
                    for query in queries {
                        ui.horizontal(|ui| {
                            if ui
                                .selectable_label(
                                    false,
                                    icon_text(Icon::FileCode2, &query.name, self.theme.text_secondary),
                                )
                                .clicked()
                            {
                                self.query_text = query.sql.clone();
                                self.active_tab = WorkspaceTab::Query;
                            }
                            if compact_button(ui, "rename", self.theme).clicked() {
                                let request_id = self.task_bridge.next_request_id();
                                let name = if self.query_folder.trim().is_empty() {
                                    format!("{} (renamed)", query.name)
                                } else {
                                    self.query_folder.trim().to_owned()
                                };
                                let _ = self.task_bridge.send(UiCommand::RenameSavedQuery {
                                    request_id,
                                    id: query.id.clone(),
                                    name,
                                });
                            }
                            if compact_button(ui, "delete", self.theme).clicked() {
                                self.delete_confirmation_id = Some(query.id.clone());
                            }
                        });
                    }
                });
                if delete_requested {
                    self.folder_delete_confirmation = folder_id;
                }
            }
            if let Some(id) = self.delete_confirmation_id.clone() {
                ui.colored_label(self.theme.warning, "Delete this saved query?");
                ui.horizontal(|ui| {
                    if compact_button(ui, "Confirm delete", self.theme).clicked() {
                        let request_id = self.task_bridge.next_request_id();
                        let _ = self.task_bridge.send(UiCommand::DeleteSavedQuery { request_id, id });
                        self.delete_confirmation_id = None;
                    }
                    if compact_button(ui, "Cancel", self.theme).clicked() {
                        self.delete_confirmation_id = None;
                    }
                });
            }
        }
        ui.separator();
        ui.label(
            RichText::new("Local history")
                .small()
                .strong()
                .color(self.theme.text_muted),
        );
        if self.query_history.is_empty() {
            ui.label(RichText::new("No queries run yet").color(self.theme.text_muted));
            return;
        }
        for query in self.query_history.iter().rev() {
            let title = query.lines().next().unwrap_or("query");
            ui.vertical(|ui| {
                ui.label(RichText::new(title).color(self.theme.text_secondary));
                ui.label(RichText::new("local history").small().color(self.theme.text_muted));
            });
            ui.add_space(12.0);
        }
    }

    fn draw_settings(&mut self, ui: &mut egui::Ui) {
        card_frame(self.theme).show(ui, |ui| {
            section_label(ui, "APPEARANCE", self.theme);
            ui.add_space(10.0);
            ui.horizontal_wrapped(|ui| {
                ui.label(icon_text(
                    if self.dark_mode { Icon::Moon } else { Icon::Sun },
                    "",
                    self.theme.accent,
                ));
                ui.selectable_value(&mut self.dark_mode, false, "Light");
                ui.selectable_value(&mut self.dark_mode, true, "Dark");
            });
            ui.label(
                RichText::new(
                    "Quiet surfaces, violet focus states, and high-contrast data. The choice is saved locally.",
                )
                .small()
                .color(self.theme.text_muted),
            );
            ui.checkbox(&mut self.reduce_motion, "Reduce motion");
            ui.label(
                RichText::new("Loading states keep a static status icon instead of a spinner.")
                    .small()
                    .color(self.theme.text_muted),
            );
        });
        ui.add_space(12.0);
        card_frame(self.theme).show(ui, |ui| {
            section_label(ui, "DATABASE FILES", self.theme);
            ui.add_space(10.0);
            ui.label(
                RichText::new("Backup destination")
                    .small()
                    .color(self.theme.text_secondary),
            );
            input_full_width(
                ui,
                &mut self.backup_output_path,
                "Choose a .sql backup path",
                self.theme,
            );
            ui.horizontal_wrapped(|ui| {
                if compact_button_with_icon(ui, Icon::FolderOpen, "Choose path", self.theme).clicked() {
                    let request_id = self.task_bridge.next_request_id();
                    let _ = self.task_bridge.send(UiCommand::PickBackupFile { request_id });
                }
                if secondary_button_with_icon(ui, Icon::Archive, "Create backup", self.theme).clicked() {
                    if let Some(connection) = self.connections.first() {
                        let request_id = self.task_bridge.next_request_id();
                        let _ = self.task_bridge.send(UiCommand::Backup {
                            request_id,
                            connection_id: connection.id.clone(),
                            output_path: self.backup_output_path.clone(),
                            custom_format: false,
                        });
                    }
                }
            });
            ui.add_space(14.0);
            ui.label(
                RichText::new("Restore from backup")
                    .small()
                    .color(self.theme.text_secondary),
            );
            input_full_width(ui, &mut self.restore_input_path, "Choose a backup file", self.theme);
            ui.horizontal_wrapped(|ui| {
                if compact_button_with_icon(ui, Icon::FolderOpen, "Choose file", self.theme).clicked() {
                    let request_id = self.task_bridge.next_request_id();
                    let _ = self.task_bridge.send(UiCommand::PickRestoreFile { request_id });
                }
                if secondary_button_with_icon(ui, Icon::RotateCcw, "Restore database", self.theme).clicked() {
                    self.restore_confirmation = true;
                }
            });
            if self.restore_confirmation {
                ui.add_space(10.0);
                ui.colored_label(self.theme.warning, "Overwrite the active database?");
                ui.horizontal(|ui| {
                    if danger_button(ui, "Confirm restore", self.theme).clicked() {
                        if let Some(connection) = self.connections.first() {
                            let request_id = self.task_bridge.next_request_id();
                            let _ = self.task_bridge.send(UiCommand::Restore {
                                request_id,
                                connection_id: connection.id.clone(),
                                input_path: self.restore_input_path.clone(),
                                custom_format: false,
                            });
                        }
                        self.restore_confirmation = false;
                    }
                    if ghost_button_with_icon(ui, Icon::X, "Cancel", self.theme).clicked() {
                        self.restore_confirmation = false;
                    }
                });
            }
        });
    }
}
