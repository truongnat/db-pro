use super::*;

impl DbProApp {
    pub(super) fn draw_explorer(&mut self, ui: &mut egui::Ui) {
        self.draw_explorer_connections(ui);
        self.draw_explorer_schema_feedback(ui);
        self.draw_explorer_tables(ui);
        let schema_scope = self.active_schema().to_owned();
        ui.indent(("schema-objects", schema_scope.as_str()), |ui| {
            ui.label(icon_text(Icon::Layers3, &schema_scope, self.theme.text_muted));
            self.draw_explorer_views(ui);
            self.draw_explorer_triggers(ui);
            self.draw_explorer_functions(ui);
        });
        self.draw_explorer_footer(ui);
    }

    fn draw_explorer_connections(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.label(icon_text(Icon::ChevronDown, "", self.theme.text_muted));
            ui.label(
                RichText::new("DATABASE")
                    .size(11.0)
                    .strong()
                    .color(self.theme.text_muted),
            );
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                if compact_icon_button(ui, Icon::Plus, self.theme)
                    .on_hover_text("New connection")
                    .clicked()
                {
                    self.open_new_connection();
                }
                let mut refresh_schema = false;
                let more = compact_icon_button(ui, Icon::MoreHorizontal, self.theme).on_hover_text("Explorer actions");
                more.context_menu(|ui| {
                    if ui.button("Refresh schema").clicked() {
                        refresh_schema = true;
                        ui.close_menu();
                    }
                });
                if refresh_schema {
                    if let Some(connection_id) = self.active_connection_id.clone() {
                        self.request_schema_introspection(connection_id, true);
                    }
                }
            });
        });
        ui.add_space(8.0);
        if self.connections.is_empty() {
            ui.label(RichText::new("No saved connections").color(self.theme.text_muted));
            ui.label(
                RichText::new("Connect a database to load its schema here.")
                    .small()
                    .color(self.theme.text_muted),
            );
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
                    let mut connect_from_menu = false;
                    let mut edit_from_menu = false;
                    let mut delete_from_menu = false;
                    let connection_clicked = connection_button.clicked();
                    connection_button.context_menu(|ui| {
                        if ui.button("Connect").clicked() {
                            connect_from_menu = true;
                            ui.close_menu();
                        }
                        if ui.button("Edit connection").clicked() {
                            edit_from_menu = true;
                            ui.close_menu();
                        }
                        if ui.button("Delete connection").clicked() {
                            delete_from_menu = true;
                            ui.close_menu();
                        }
                    });
                    if connection_clicked || connect_from_menu {
                        self.reset_agent_context();
                        self.active_connection_id = Some(connection.id.clone());
                        self.selected_schema = None;
                        self.schema = UiSchemaSummary::default();
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
                        self.staged_changes.clear();
                        self.staged_apply_request = None;
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
                        self.schema_request = None;
                        self.schema_error = None;
                        let _ = self.task_bridge.send(UiCommand::Connect {
                            request_id,
                            connection_id: connection.id.clone(),
                        });
                        self.runtime_message = format!("Connecting to {}…", connection.name);
                    }
                    if edit_from_menu {
                        self.open_edit_connection(&connection);
                    }
                    if delete_from_menu {
                        self.delete_confirmation_id = Some(connection.id.clone());
                    }
                });
                ui.add_space(1.0);
                ui.label(
                    RichText::new(format!("    {} · {}", connection.driver, connection.database))
                        .size(11.0)
                        .color(self.theme.text_muted),
                );
            }
        }
        ui.add_space(12.0);
    }
    fn draw_explorer_schema_feedback(&mut self, ui: &mut egui::Ui) {
        let schema_error = self.schema_error.clone();
        if let Some(error) = schema_error.as_deref() {
            grid_frame(self.theme).show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label(icon_text(Icon::TriangleAlert, "Schema load failed", self.theme.danger));
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        if let Some(connection_id) = self.active_connection_id.clone() {
                            if secondary_button_with_icon(ui, Icon::RotateCcw, "Refresh schema", self.theme).clicked() {
                                self.request_schema_introspection(connection_id, true);
                            }
                        }
                    });
                });
                ui.add_space(6.0);
                egui::ScrollArea::vertical().max_height(120.0).show(ui, |ui| {
                    ui.label(
                        RichText::new(error)
                            .small()
                            .monospace()
                            .color(self.theme.text_secondary),
                    );
                });
            });
            ui.add_space(8.0);
        } else if self.schema_request.is_some() {
            grid_frame(self.theme).show(ui, |ui| {
                ui.horizontal(|ui| {
                    if self.reduce_motion {
                        ui.label(icon_text(Icon::LoaderCircle, "Loading schema…", self.theme.accent));
                    } else {
                        ui.spinner();
                        ui.label(RichText::new("Loading schema…").color(self.theme.accent));
                    }
                    ui.label(
                        RichText::new("Large databases may take a moment.")
                            .small()
                            .color(self.theme.text_muted),
                    );
                });
            });
            ui.add_space(8.0);
        }
        ui.horizontal(|ui| {
            let clear_width = if self.explorer_search.is_empty() { 0.0 } else { 52.0 };
            let search_width = (ui.available_width() - clear_width).max(120.0);
            input(
                ui,
                &mut self.explorer_search,
                "Search tables…",
                search_width,
                self.theme,
            );
            if !self.explorer_search.is_empty() && compact_button(ui, "Clear", self.theme).clicked() {
                self.explorer_search.clear();
            }
        });
        ui.add_space(8.0);
    }
    fn draw_explorer_schemas(&mut self, ui: &mut egui::Ui) {
        let schemas = self.schema.schemas.clone();
        let database_name = self
            .active_connection()
            .map(|connection| connection.database.clone())
            .filter(|database| !database.is_empty())
            .unwrap_or_else(|| "active database".to_owned());
        ui.horizontal(|ui| {
            ui.label(icon_text(Icon::Database, &database_name, self.theme.text_secondary));
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                ui.label(
                    RichText::new(format!(
                        "{} schema{}",
                        schemas.len(),
                        if schemas.len() == 1 { "" } else { "s" }
                    ))
                    .small()
                    .color(self.theme.text_muted),
                );
            });
        });
        ui.add_space(3.0);
        if schemas.is_empty() {
            ui.indent("empty-schema", |ui| {
                ui.label(
                    RichText::new("Schema metadata unavailable")
                        .small()
                        .color(self.theme.text_muted),
                );
            });
        } else {
            for schema in &schemas {
                let selected = self.active_schema() == schema;
                if sidebar_item(ui, Icon::Layers3, schema, selected, self.theme).clicked() {
                    self.selected_schema = Some(schema.clone());
                    self.selected_table = None;
                    self.selected_schema_object = None;
                    self.table_info = None;
                    self.table_ddl = None;
                    self.table_data_result = None;
                    self.staged_changes.clear();
                    self.active_tab = WorkspaceTab::Welcome;
                }
            }
        }
    }

    fn draw_explorer_tables(&mut self, ui: &mut egui::Ui) {
        self.draw_explorer_schemas(ui);
        self.draw_explorer_table_list(ui);
    }

    fn draw_explorer_table_list(&mut self, ui: &mut egui::Ui) {
        let search_query = self.explorer_search.trim().to_ascii_lowercase();
        let all_tables = self.active_schema_table_names();
        let total_tables = all_tables.len();
        let (matching_table_count, tables) = filtered_explorer_tables(&all_tables, &search_query);
        ui.collapsing(
            icon_text(
                Icon::Table2,
                &format!("Tables ({} / {})", matching_table_count, total_tables),
                self.theme.text_primary,
            ),
            |ui| {
                if tables.is_empty() {
                    ui.label(
                        RichText::new(if total_tables == 0 {
                            "No tables found in the active schema."
                        } else {
                            "No tables match the current search."
                        })
                        .small()
                        .color(self.theme.text_muted),
                    );
                }
                for table in &tables {
                    let is_selected = self.selected_table.as_deref() == Some(table.as_str());
                    let table_response = sidebar_item(ui, Icon::Table2, table, is_selected, self.theme);
                    let mut open_query = false;
                    let mut ask_agent = false;
                    let mut refresh_schema = false;
                    table_response.context_menu(|ui| {
                        if ui.button("Open in Query").clicked() {
                            open_query = true;
                            ui.close_menu();
                        }
                        if ui.button("Ask Agent about table").clicked() {
                            ask_agent = true;
                            ui.close_menu();
                        }
                        if ui.button("Refresh schema").clicked() {
                            refresh_schema = true;
                            ui.close_menu();
                        }
                    });
                    if table_response.clicked() || open_query || ask_agent {
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
                        self.staged_changes.clear();
                        self.staged_apply_request = None;
                        self.selected_cell = None;
                        self.selected_row = None;
                        self.data_editing_cell = None;
                        self.data_edit_value.clear();
                        self.data_delete_confirmation = false;
                        self.query_text = format!("SELECT *\nFROM {table}\nLIMIT 100;");
                        self.request_table_info();
                        self.active_tab = WorkspaceTab::Table;
                    }
                    if open_query {
                        self.query_text = format!("SELECT *\nFROM {table}\nLIMIT 100;");
                        self.active_tab = WorkspaceTab::Query;
                    }
                    if ask_agent {
                        self.open_agent_prompt(
                            format!("Explain the `{table}` table and suggest useful read-only queries"),
                            ui.ctx(),
                        );
                    }
                    if refresh_schema {
                        if let Some(connection_id) = self.active_connection_id.clone() {
                            self.request_schema_introspection(connection_id, true);
                        }
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
        let schema_name = self.active_schema().to_owned();
        let views: Vec<_> = if self.schema.schemas.is_empty() {
            self.schema.views.clone()
        } else {
            self.schema
                .views
                .iter()
                .filter(|view| view.schema == schema_name)
                .cloned()
                .collect()
        };
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
                    let view_response = sidebar_item(ui, Icon::Eye, &view.name, is_selected, self.theme);
                    let mut open_query = false;
                    view_response.context_menu(|ui| {
                        if ui.button("Open in Query").clicked() {
                            open_query = true;
                            ui.close_menu();
                        }
                    });
                    if view_response.clicked() {
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
                    if open_query {
                        self.query_text = format!("SELECT *\nFROM {}\nLIMIT 100;", view.name);
                        self.active_tab = WorkspaceTab::Query;
                    }
                }
            },
        );
    }
    fn draw_explorer_triggers(&mut self, ui: &mut egui::Ui) {
        let schema_name = self.active_schema().to_owned();
        let triggers: Vec<_> = if self.schema.schemas.is_empty() {
            self.schema.triggers.clone()
        } else {
            self.schema
                .triggers
                .iter()
                .filter(|trigger| trigger.schema == schema_name)
                .cloned()
                .collect()
        };
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
        let supports_functions = self
            .active_capabilities()
            .is_some_and(|capabilities| capabilities.schema.functions);
        let schema_name = self.active_schema().to_owned();
        let functions: Vec<_> = if self.schema.schemas.is_empty() {
            self.schema.functions.clone()
        } else {
            self.schema
                .functions
                .iter()
                .filter(|function| function.schema == schema_name)
                .cloned()
                .collect()
        };
        ui.collapsing(
            icon_text(
                Icon::Code2,
                &format!("Functions / Procedures ({})", functions.len()),
                self.theme.text_primary,
            ),
            |ui| {
                if !supports_functions {
                    ui.label(
                        RichText::new(format!("{} does not expose routines", self.active_driver()))
                            .small()
                            .color(self.theme.text_muted),
                    );
                    return;
                }
                if functions.is_empty() {
                    ui.label(RichText::new("No functions").small().color(self.theme.text_muted));
                }
                for function in functions.iter().take(100) {
                    let is_selected = matches!(
                        self.selected_schema_object.as_ref(),
                        Some(SchemaObjectSelection::Function(selected)) if selected == &function.name
                    );
                    let label = format!("{} · {}", function.name, function.routine_type);
                    let icon = if function.routine_type.eq_ignore_ascii_case("procedure") {
                        Icon::GitBranch
                    } else {
                        Icon::Code2
                    };
                    let function_response = sidebar_item(ui, icon, &label, is_selected, self.theme);
                    let mut open_query = false;
                    function_response.context_menu(|ui| {
                        if ui.button("Open call in Query").clicked() {
                            open_query = true;
                            ui.close_menu();
                        }
                    });
                    if function_response.clicked() {
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
                    if open_query {
                        self.query_text = format!("SELECT *\nFROM {}.{}();", function.schema, function.name);
                        self.active_tab = WorkspaceTab::Query;
                    }
                }
            },
        );
    }
    fn draw_explorer_footer(&mut self, ui: &mut egui::Ui) {
        if self.table_info.is_none() {
            let columns = self.active_schema_column_names();
            ui.collapsing(
                icon_text(
                    Icon::Columns3,
                    &format!("Columns ({})", columns.len()),
                    self.theme.text_primary,
                ),
                |ui| {
                    for column in columns.iter().take(100) {
                        ui.label(
                            RichText::new(format!("  {column}"))
                                .small()
                                .color(self.theme.text_muted),
                        );
                    }
                },
            );
        }
        ui.add_space(8.0);
        ui.label(
            RichText::new("Right-click a connection for actions")
                .small()
                .color(self.theme.text_muted),
        );
    }
}
