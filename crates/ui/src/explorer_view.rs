use super::*;

impl DbProApp {
    /// Entry-point for Database Navigator in DBeaver style:
    /// Unified hierarchical tree where connections are root nodes.
    ///
    /// ```text
    /// [ 🔍 Filter objects...              ] [+] [🔄]
    /// ▼ ● Xe Lạc Hồng (PostgreSQL)    [PG]
    ///   ▼ 🗄️ fullstack_starter
    ///     ▼ 📁 public                (68)
    ///       ▼ 📁 Tables             (68)
    ///         ▶ ⊞ users
    ///         ▶ ⊞ vehicles
    ///         ...
    ///       ▶ 📁 Views               (0)
    ///       ▶ 📁 Functions           (0)
    ///       ▶ 📁 Triggers            (0)
    ///     ▶ 📁 information_schema
    ///     ▶ 📁 pg_catalog
    /// ▶ ○ Local SQLite              [SQLITE]
    /// ```
    pub(super) fn draw_explorer_sub_panes(&mut self, ui: &mut egui::Ui) {
        // ── Toolbar: Filter bar + Action buttons ────────────────────────
        ui.horizontal(|ui| {
            let clear_width = if self.explorer_search.is_empty() { 0.0 } else { 26.0 };
            let search_width = (ui.available_width() - clear_width - 64.0).max(80.0);
            input(
                ui,
                &mut self.explorer_search,
                "Filter objects…",
                search_width,
                self.theme,
            );
            if !self.explorer_search.is_empty()
                && compact_icon_button(ui, Icon::X, self.theme)
                    .on_hover_text("Clear filter")
                    .clicked()
            {
                self.explorer_search.clear();
            }
            if compact_icon_button(ui, Icon::Plus, self.theme)
                .on_hover_text("New connection")
                .clicked()
            {
                self.open_new_connection();
            }
            let mut refresh_schema = false;
            let refresh_btn =
                compact_icon_button(ui, Icon::RotateCcw, self.theme).on_hover_text("Refresh active schema");
            refresh_btn.context_menu(|ui| {
                if ui.button("Refresh Schema").clicked() {
                    refresh_schema = true;
                    ui.close_menu();
                }
            });
            if refresh_btn.clicked() || refresh_schema {
                if let Some(connection_id) = self.active_connection_id.clone() {
                    self.request_schema_introspection(connection_id, true);
                }
            }
        });
        ui.add_space(6.0);

        // ── Unified Database Navigator Tree ─────────────────────────────
        egui::ScrollArea::vertical()
            .id_salt("dbeaver_navigator_scroll")
            .show(ui, |ui| {
                if self.connections.is_empty() {
                    self.draw_dbeaver_empty_state(ui);
                } else {
                    self.draw_dbeaver_connections_tree(ui);
                }
            });
    }

    /// Empty state shown when no connections exist yet.
    fn draw_dbeaver_empty_state(&mut self, ui: &mut egui::Ui) {
        ui.add_space(20.0);
        ui.vertical_centered(|ui| {
            ui.label(icon_text(Icon::Database, "", self.theme.accent));
            ui.add_space(6.0);
            ui.label(RichText::new("No connections").strong().color(self.theme.text_primary));
            ui.add_space(2.0);
            ui.label(
                RichText::new("Create a database connection to begin.")
                    .small()
                    .color(self.theme.text_muted),
            );
            ui.add_space(12.0);
            if compact_button_with_icon(ui, Icon::Plus, "New connection", self.theme).clicked() {
                self.open_new_connection();
            }
        });
    }

    /// Renders the list of connections as expandable root nodes (DBeaver style).
    fn draw_dbeaver_connections_tree(&mut self, ui: &mut egui::Ui) {
        let connections = self.connections.clone();
        for connection in connections {
            let is_active = self.active_connection_id.as_deref() == Some(&connection.id);
            let is_connected = self.connected && is_active;
            let id = ui.make_persistent_id(("dbeaver_conn_node", &connection.id));

            let mut connect_now = false;
            let mut disconnect_now = false;
            let mut refresh_now = false;
            let mut edit_now = false;
            let mut delete_now = false;

            let collapsing =
                egui::collapsing_header::CollapsingState::load_with_default_open(ui.ctx(), id, is_connected);

            collapsing
                .show_header(ui, |ui| {
                    ui.horizontal(|ui| {
                        // Connection status dot
                        let dot_color = if is_connected {
                            Color32::from_rgb(120, 216, 155) // vibrant green
                        } else if self.pending_connection_request.is_some() && is_active {
                            self.theme.accent
                        } else {
                            self.theme.text_muted
                        };
                        ui.label(RichText::new("●").size(9.0).color(dot_color));

                        // Connection name
                        let name_label = ui.add(
                            egui::Label::new(RichText::new(&connection.name).size(12.5).strong().color(
                                if is_connected {
                                    self.theme.text_primary
                                } else {
                                    self.theme.text_secondary
                                },
                            ))
                            .sense(egui::Sense::click()),
                        );

                        if name_label.clicked() && !is_connected {
                            connect_now = true;
                        }

                        // Driver pill badge
                        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                            let badge_text = if connection.driver.eq_ignore_ascii_case("postgresql") {
                                "PG"
                            } else {
                                "SQLITE"
                            };
                            badge(
                                ui,
                                badge_text,
                                if is_connected {
                                    self.theme.accent_soft
                                } else {
                                    self.theme.surface_hover
                                },
                                if is_connected {
                                    self.theme.accent
                                } else {
                                    self.theme.text_muted
                                },
                            );
                        });

                        name_label.context_menu(|ui| {
                            if is_connected {
                                if ui.button("Disconnect").clicked() {
                                    disconnect_now = true;
                                    ui.close_menu();
                                }
                                if ui.button("Refresh Schema").clicked() {
                                    refresh_now = true;
                                    ui.close_menu();
                                }
                            } else if ui.button("Connect").clicked() {
                                connect_now = true;
                                ui.close_menu();
                            }
                            if ui.button("Edit Connection").clicked() {
                                edit_now = true;
                                ui.close_menu();
                            }
                            if ui.button("Delete Connection").clicked() {
                                delete_now = true;
                                ui.close_menu();
                            }
                        });
                    });
                })
                .body(|ui| {
                    if is_connected {
                        self.draw_dbeaver_connected_body(ui, &connection);
                    } else {
                        ui.horizontal(|ui| {
                            ui.label(RichText::new("Disconnected.").small().color(self.theme.text_muted));
                            if compact_button(ui, "Connect", self.theme).clicked() {
                                connect_now = true;
                            }
                        });
                    }
                });

            if connect_now {
                self.connect_to_connection(&connection);
            }
            if disconnect_now {
                self.connected = false;
                self.schema = UiSchemaSummary::default();
                self.selected_table = None;
                self.selected_schema_object = None;
                self.runtime_message = format!("Disconnected from {}", connection.name);
            }
            if refresh_now {
                self.request_schema_introspection(connection.id.clone(), true);
            }
            if edit_now {
                self.open_edit_connection(&connection);
            }
            if delete_now {
                self.delete_confirmation_id = Some(connection.id.clone());
            }
            ui.add_space(2.0);
        }
    }

    /// Body rendered when a connection node is expanded.
    fn draw_dbeaver_connected_body(&mut self, ui: &mut egui::Ui, connection: &UiConnectionSummary) {
        // Schema feedback (progress / error)
        self.draw_explorer_schema_feedback(ui);

        // 🗄️ Database node
        let db_id = ui.make_persistent_id(("dbeaver_db_node", &connection.id, &connection.database));
        let db_name = if connection.database.is_empty() {
            "database".to_owned()
        } else {
            connection.database.clone()
        };

        egui::collapsing_header::CollapsingState::load_with_default_open(ui.ctx(), db_id, true)
            .show_header(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label(icon_text(Icon::Database, &db_name, self.theme.accent));
                });
            })
            .body(|ui| {
                let schemas = self.schema.schemas.clone();
                if schemas.is_empty() {
                    // Flat tables/views (e.g. SQLite)
                    self.draw_dbeaver_schema_objects(ui, "");
                } else {
                    // Nested Schemas (e.g. PostgreSQL: public, information_schema, etc.)
                    for schema in &schemas {
                        self.draw_dbeaver_schema_node(ui, &connection.id, schema);
                    }
                }
            });
    }

    /// A schema folder node inside the Database node (e.g. `📁 public`).
    fn draw_dbeaver_schema_node(&mut self, ui: &mut egui::Ui, connection_id: &str, schema: &str) {
        let is_active_schema = self.active_schema() == schema;
        let schema_id = ui.make_persistent_id(("dbeaver_schema_node", connection_id, schema));
        let table_count = self.schema.table_details.iter().filter(|t| t.schema == schema).count();

        let folder_icon = if is_active_schema {
            Icon::FolderOpen
        } else {
            Icon::Folder
        };
        let folder_color = if is_active_schema {
            self.theme.accent
        } else {
            self.theme.text_secondary
        };

        let mut activate_schema = false;

        egui::collapsing_header::CollapsingState::load_with_default_open(ui.ctx(), schema_id, is_active_schema)
            .show_header(ui, |ui| {
                ui.horizontal(|ui| {
                    let resp = ui.add(
                        egui::Label::new(icon_text(folder_icon, schema, folder_color)).sense(egui::Sense::click()),
                    );
                    if resp.clicked() && !is_active_schema {
                        activate_schema = true;
                    }
                    if table_count > 0 {
                        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                            ui.label(
                                RichText::new(table_count.to_string())
                                    .small()
                                    .color(self.theme.text_muted),
                            );
                        });
                    }
                });
            })
            .body(|ui| {
                if is_active_schema {
                    self.draw_dbeaver_schema_objects(ui, schema);
                } else {
                    ui.horizontal(|ui| {
                        ui.label(RichText::new("Inactive schema.").small().color(self.theme.text_muted));
                        if compact_button(ui, "Activate", self.theme).clicked() {
                            activate_schema = true;
                        }
                    });
                }
            });

        if activate_schema {
            self.selected_schema = Some(schema.to_owned());
            self.selected_table = None;
            self.selected_schema_object = None;
            self.table_info = None;
            self.table_ddl = None;
            self.table_data_result = None;
            self.staged_changes.clear();
            self.active_tab = WorkspaceTab::Welcome;
        }
    }

    /// Renders the folders for a schema: Tables, Views, Functions, Triggers.
    fn draw_dbeaver_schema_objects(&mut self, ui: &mut egui::Ui, schema: &str) {
        let search_query = self.explorer_search.trim().to_ascii_lowercase();
        let all_tables = self.active_schema_table_names();
        let total_tables = all_tables.len();
        let (matching_table_count, tables) = filtered_explorer_tables(&all_tables, &search_query);

        // ── 📁 Tables folder (Expanded by default) ───────────────────────
        let tables_folder_id = ui.make_persistent_id(("dbeaver_tbl_folder", schema));
        let tables_label = if search_query.is_empty() {
            format!("Tables ({total_tables})")
        } else {
            format!("Tables ({matching_table_count} / {total_tables})")
        };

        egui::collapsing_header::CollapsingState::load_with_default_open(ui.ctx(), tables_folder_id, true)
            .show_header(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label(icon_text(Icon::Folder, &tables_label, self.theme.text_primary));
                });
            })
            .body(|ui| {
                if tables.is_empty() {
                    ui.label(
                        RichText::new(if total_tables == 0 {
                            "No tables in schema"
                        } else {
                            "No matching tables"
                        })
                        .small()
                        .color(self.theme.text_muted),
                    );
                } else {
                    for table in &tables {
                        self.draw_dbeaver_table_item(ui, table);
                    }
                }
            });

        // ── 📁 Views folder ──────────────────────────────────────────────
        let schema_name = schema.to_owned();
        let views: Vec<_> = if self.schema.schemas.is_empty() {
            self.schema.views.clone()
        } else {
            self.schema
                .views
                .iter()
                .filter(|v| v.schema == schema_name)
                .cloned()
                .collect()
        };
        self.draw_dbeaver_views_folder(ui, &views);

        // ── 📁 Functions folder (PostgreSQL only) ────────────────────────
        let supports_functions = self.active_capabilities().is_some_and(|c| c.schema.functions);
        if supports_functions {
            let functions: Vec<_> = if self.schema.schemas.is_empty() {
                self.schema.functions.clone()
            } else {
                self.schema
                    .functions
                    .iter()
                    .filter(|f| f.schema == schema_name)
                    .cloned()
                    .collect()
            };
            self.draw_dbeaver_functions_folder(ui, &functions);
        }

        // ── 📁 Triggers folder ───────────────────────────────────────────
        let triggers: Vec<_> = if self.schema.schemas.is_empty() {
            self.schema.triggers.clone()
        } else {
            self.schema
                .triggers
                .iter()
                .filter(|t| t.schema == schema_name)
                .cloned()
                .collect()
        };
        self.draw_dbeaver_triggers_folder(ui, &triggers);
    }

    /// Renders an individual table item in the tree with selection and expandable details.
    fn draw_dbeaver_table_item(&mut self, ui: &mut egui::Ui, table: &str) {
        let is_selected = self.selected_table.as_deref() == Some(table);
        let resp = sidebar_item(ui, Icon::Table2, table, is_selected, self.theme);

        let mut open_query = false;
        let mut ask_agent = false;
        let mut refresh_schema = false;

        resp.context_menu(|ui| {
            if ui.button("Open in Query").clicked() {
                open_query = true;
                ui.close_menu();
            }
            if ui.button("Ask Agent about table").clicked() {
                ask_agent = true;
                ui.close_menu();
            }
            if ui.button("Refresh Schema").clicked() {
                refresh_schema = true;
                ui.close_menu();
            }
        });

        if resp.clicked() || open_query || ask_agent {
            self.selected_table = Some(table.to_owned());
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

        // If table is selected, show nested DBeaver metadata (Columns, Indexes, Foreign keys)
        if is_selected {
            if let Some(info) = self.table_info.clone() {
                ui.indent(("table-tree-details", table), |ui| {
                    // Columns folder
                    let col_id = ui.make_persistent_id(("tbl_col_folder", table));
                    egui::collapsing_header::CollapsingState::load_with_default_open(ui.ctx(), col_id, false)
                        .show_header(ui, |ui| {
                            ui.horizontal(|ui| {
                                ui.label(icon_text(
                                    Icon::Columns3,
                                    &format!("Columns ({})", info.columns.len()),
                                    self.theme.text_secondary,
                                ));
                            });
                        })
                        .body(|ui| {
                            for column in &info.columns {
                                ui.horizontal(|ui| {
                                    ui.label(RichText::new(&column.name).small().color(self.theme.text_primary));
                                    ui.label(RichText::new(&column.data_type).small().color(self.theme.text_muted));
                                });
                            }
                        });

                    // Foreign keys folder
                    let fk_id = ui.make_persistent_id(("tbl_fk_folder", table));
                    egui::collapsing_header::CollapsingState::load_with_default_open(ui.ctx(), fk_id, false)
                        .show_header(ui, |ui| {
                            ui.horizontal(|ui| {
                                ui.label(icon_text(
                                    Icon::ArrowRightLeft,
                                    &format!("Foreign keys ({})", info.foreign_keys.len()),
                                    self.theme.text_secondary,
                                ));
                            });
                        })
                        .body(|ui| {
                            if info.foreign_keys.is_empty() {
                                ui.label(RichText::new("No foreign keys").small().color(self.theme.text_muted));
                            } else {
                                for fk in &info.foreign_keys {
                                    ui.label(RichText::new(&fk.name).small().color(self.theme.text_muted));
                                }
                            }
                        });

                    // Indexes folder
                    let idx_id = ui.make_persistent_id(("tbl_idx_folder", table));
                    egui::collapsing_header::CollapsingState::load_with_default_open(ui.ctx(), idx_id, false)
                        .show_header(ui, |ui| {
                            ui.horizontal(|ui| {
                                ui.label(icon_text(
                                    Icon::List,
                                    &format!("Indexes ({})", info.indexes.len()),
                                    self.theme.text_secondary,
                                ));
                            });
                        })
                        .body(|ui| {
                            if info.indexes.is_empty() {
                                ui.label(RichText::new("No indexes").small().color(self.theme.text_muted));
                            } else {
                                for idx in &info.indexes {
                                    ui.label(RichText::new(&idx.name).small().color(self.theme.text_muted));
                                }
                            }
                        });
                });
            }
        }
    }

    /// Views folder in DBeaver tree.
    fn draw_dbeaver_views_folder(&mut self, ui: &mut egui::Ui, views: &[UiViewSummary]) {
        let dimmed = views.is_empty();
        let folder_id = ui.make_persistent_id("dbeaver_views_folder");
        let label = format!("Views ({})", views.len());

        egui::collapsing_header::CollapsingState::load_with_default_open(ui.ctx(), folder_id, false)
            .show_header(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label(icon_text(
                        Icon::Folder,
                        &label,
                        if dimmed {
                            self.theme.text_muted
                        } else {
                            self.theme.text_secondary
                        },
                    ));
                });
            })
            .body(|ui| {
                if dimmed {
                    ui.label(RichText::new("No views").small().color(self.theme.text_muted));
                } else {
                    for view in views {
                        let is_selected = matches!(
                            self.selected_schema_object.as_ref(),
                            Some(SchemaObjectSelection::View(s)) if s == &view.name
                        );
                        let resp = sidebar_item(ui, Icon::Eye, &view.name, is_selected, self.theme);
                        let mut open_query = false;
                        resp.context_menu(|ui| {
                            if ui.button("Open in Query").clicked() {
                                open_query = true;
                                ui.close_menu();
                            }
                        });
                        if resp.clicked() {
                            self.selected_schema_object = Some(SchemaObjectSelection::View(view.name.clone()));
                            self.schema_object_view = SchemaObjectView::Definition;
                            self.selected_table = None;
                            self.table_info = None;
                            self.table_ddl = None;
                            self.table_view = TableView::Ddl;
                            self.active_tab = WorkspaceTab::SchemaObject;
                            self.runtime_message = format!("Opened view {}.{}", view.schema, view.name);
                        }
                        if open_query {
                            self.query_text = format!("SELECT *\nFROM {}\nLIMIT 100;", view.name);
                            self.active_tab = WorkspaceTab::Query;
                        }
                    }
                }
            });
    }

    /// Functions folder in DBeaver tree.
    fn draw_dbeaver_functions_folder(&mut self, ui: &mut egui::Ui, functions: &[UiFunctionSummary]) {
        let dimmed = functions.is_empty();
        let folder_id = ui.make_persistent_id("dbeaver_functions_folder");
        let label = format!("Functions ({})", functions.len());

        egui::collapsing_header::CollapsingState::load_with_default_open(ui.ctx(), folder_id, false)
            .show_header(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label(icon_text(
                        Icon::Folder,
                        &label,
                        if dimmed {
                            self.theme.text_muted
                        } else {
                            self.theme.text_secondary
                        },
                    ));
                });
            })
            .body(|ui| {
                if dimmed {
                    ui.label(RichText::new("No functions").small().color(self.theme.text_muted));
                } else {
                    for function in functions {
                        let is_selected = matches!(
                            self.selected_schema_object.as_ref(),
                            Some(SchemaObjectSelection::Function(s)) if s == &function.name
                        );
                        let icon = if function.routine_type.eq_ignore_ascii_case("procedure") {
                            Icon::GitBranch
                        } else {
                            Icon::Code2
                        };
                        let label = format!("{} · {}", function.name, function.routine_type);
                        let resp = sidebar_item(ui, icon, &label, is_selected, self.theme);
                        let mut open_query = false;
                        resp.context_menu(|ui| {
                            if ui.button("Open call in Query").clicked() {
                                open_query = true;
                                ui.close_menu();
                            }
                        });
                        if resp.clicked() {
                            self.selected_schema_object = Some(SchemaObjectSelection::Function(function.name.clone()));
                            self.schema_object_view = SchemaObjectView::Definition;
                            self.selected_table = None;
                            self.table_info = None;
                            self.table_ddl = None;
                            self.table_view = TableView::Ddl;
                            self.active_tab = WorkspaceTab::SchemaObject;
                            self.runtime_message = format!("Opened function {}.{}", function.schema, function.name);
                        }
                        if open_query {
                            self.query_text = format!("SELECT *\nFROM {}.{}();", function.schema, function.name);
                            self.active_tab = WorkspaceTab::Query;
                        }
                    }
                }
            });
    }

    /// Triggers folder in DBeaver tree.
    fn draw_dbeaver_triggers_folder(&mut self, ui: &mut egui::Ui, triggers: &[UiTriggerSummary]) {
        let dimmed = triggers.is_empty();
        let folder_id = ui.make_persistent_id("dbeaver_triggers_folder");
        let label = format!("Triggers ({})", triggers.len());

        egui::collapsing_header::CollapsingState::load_with_default_open(ui.ctx(), folder_id, false)
            .show_header(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label(icon_text(
                        Icon::Folder,
                        &label,
                        if dimmed {
                            self.theme.text_muted
                        } else {
                            self.theme.text_secondary
                        },
                    ));
                });
            })
            .body(|ui| {
                if dimmed {
                    ui.label(RichText::new("No triggers").small().color(self.theme.text_muted));
                } else {
                    for trigger in triggers {
                        let is_selected = matches!(
                            self.selected_schema_object.as_ref(),
                            Some(SchemaObjectSelection::Trigger(s)) if s == &trigger.name
                        );
                        let label = format!("{} · {}", trigger.name, trigger.event);
                        if sidebar_item(ui, Icon::Zap, &label, is_selected, self.theme).clicked() {
                            self.selected_schema_object = Some(SchemaObjectSelection::Trigger(trigger.name.clone()));
                            self.schema_object_view = SchemaObjectView::Definition;
                            self.selected_table = None;
                            self.table_info = None;
                            self.table_ddl = None;
                            self.table_view = TableView::Ddl;
                            self.active_tab = WorkspaceTab::SchemaObject;
                            self.runtime_message = format!("Opened trigger {}", trigger.name);
                        }
                    }
                }
            });
    }

    /// Schema loading progress and error banner.
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
    }

    /// Helper to initiate connection logic.
    fn connect_to_connection(&mut self, connection: &UiConnectionSummary) {
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
}
