//! Entry point for the Database Navigator in Codex / DBeaver style: a unified
//! hierarchical tree where connections are the root nodes.
//!
//! Row painting lives in `explorer_tree`, table details in `explorer_details`,
//! and the Views / Functions / Triggers folders in `explorer_folders`.

use super::explorer_tree::{draw_codex_tree_row, draw_hint_row, CodexTreeRow};
use super::*;
use egui::{vec2, Color32, FontFamily, Margin, Rounding, Stroke};
use lucide_icons::Icon;

/// Actions selectable from a connection row's context menu.
#[derive(Default)]
struct ConnectionRowActions {
    connect: bool,
    disconnect: bool,
    reconnect: bool,
    refresh: bool,
    new_script: bool,
    er_diagram: bool,
    ask_agent: bool,
    create_table: bool,
    copy_name: bool,
    copy_conn_string: bool,
    edit: bool,
    duplicate: bool,
    delete: bool,
}

/// Collects the connection context-menu choices in full DBeaver style.
fn connection_context_menu(
    ui: &mut egui::Ui,
    response: &egui::Response,
    is_connected: bool,
    theme: DbProTheme,
) -> ConnectionRowActions {
    let mut actions = ConnectionRowActions::default();
    let modifier = DbProApp::primary_modifier_label();
    context_action_menu(ui, response, theme, |ui, close_menu| {
        // ── 1. Connection Lifecycle ──
        if is_connected {
            if ctx_menu_item(ui, Some(Icon::Unplug), "Disconnect", None, theme.text_primary, theme).clicked() {
                actions.disconnect = true;
                *close_menu = true;
            }
            if ctx_menu_item(ui, Some(Icon::RefreshCw), "Reconnect", None, theme.text_primary, theme).clicked() {
                actions.reconnect = true;
                *close_menu = true;
            }
            if ctx_menu_item(
                ui,
                Some(Icon::RotateCcw),
                "Refresh Schema",
                Some("F5"),
                theme.text_primary,
                theme,
            )
            .clicked()
            {
                actions.refresh = true;
                *close_menu = true;
            }
        } else if ctx_menu_item(ui, Some(Icon::Plug), "Connect", None, theme.text_primary, theme).clicked() {
            actions.connect = true;
            *close_menu = true;
        }

        ui.separator();

        // ── 2. SQL & Diagrams ──
        let new_script_sc = format!("{modifier}T");
        if ctx_menu_item(
            ui,
            Some(Icon::FileCode2),
            "SQL Editor / New Script",
            Some(&new_script_sc),
            theme.text_primary,
            theme,
        )
        .clicked()
        {
            actions.new_script = true;
            *close_menu = true;
        }
        if ctx_menu_item(
            ui,
            Some(Icon::Workflow),
            "View ER Diagram",
            None,
            theme.text_primary,
            theme,
        )
        .clicked()
        {
            actions.er_diagram = true;
            *close_menu = true;
        }
        if ctx_menu_item(
            ui,
            Some(Icon::Bot),
            "Ask AI Agent about Database",
            None,
            theme.text_primary,
            theme,
        )
        .clicked()
        {
            actions.ask_agent = true;
            *close_menu = true;
        }

        ui.separator();

        // ── 3. Database Tools & Copy ──
        if ctx_menu_item(
            ui,
            Some(Icon::Plus),
            "Create Table (DDL)…",
            None,
            theme.text_primary,
            theme,
        )
        .clicked()
        {
            actions.create_table = true;
            *close_menu = true;
        }
        if ctx_menu_item(
            ui,
            Some(Icon::Copy),
            "Copy Connection Name",
            None,
            theme.text_primary,
            theme,
        )
        .clicked()
        {
            actions.copy_name = true;
            *close_menu = true;
        }
        if ctx_menu_item(
            ui,
            Some(Icon::Link),
            "Copy Connection String",
            None,
            theme.text_primary,
            theme,
        )
        .clicked()
        {
            actions.copy_conn_string = true;
            *close_menu = true;
        }

        ui.separator();

        // ── 4. Configuration & Management ──
        if ctx_menu_item(
            ui,
            Some(Icon::Pencil),
            "Edit Connection…",
            Some("F4"),
            theme.text_primary,
            theme,
        )
        .clicked()
        {
            actions.edit = true;
            *close_menu = true;
        }
        if ctx_menu_item(
            ui,
            Some(Icon::CopyPlus),
            "Duplicate Connection",
            None,
            theme.text_primary,
            theme,
        )
        .clicked()
        {
            actions.duplicate = true;
            *close_menu = true;
        }
        if ctx_menu_item(
            ui,
            Some(Icon::Trash2),
            "Delete Connection",
            Some("Del"),
            theme.danger,
            theme,
        )
        .clicked()
        {
            actions.delete = true;
            *close_menu = true;
        }
    });
    actions
}

impl DbProApp {
    /// Entry-point for Database Navigator in Codex / DBeaver style:
    /// Unified hierarchical tree where connections are root nodes.
    pub(super) fn draw_explorer_sub_panes(&mut self, ui: &mut egui::Ui) {
        self.draw_explorer_toolbar(ui);
        ui.add_space(6.0);

        // ── Unified Database Navigator Tree ─────────────────────────────
        egui::ScrollArea::vertical()
            .id_salt("codex_navigator_scroll")
            .auto_shrink([false, false])
            .show(ui, |ui| {
                let max_w = ui.available_width();
                ui.set_max_width(max_w);
                if self.connections.is_empty() {
                    self.draw_dbeaver_empty_state(ui);
                } else {
                    self.draw_dbeaver_connections_tree(ui);
                }
            });
    }

    /// Search bar plus new-connection / refresh actions above the tree.
    fn draw_explorer_toolbar(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            let clear_width = if self.explorer_search.is_empty() { 0.0 } else { 22.0 };
            let search_width = (ui.available_width() - clear_width - 56.0).max(80.0);

            egui::Frame {
                fill: self.theme.surface_hover,
                rounding: Rounding::same(6.0),
                stroke: Stroke::new(1.0, self.theme.border_subtle),
                inner_margin: Margin::symmetric(6.0, 3.0),
                ..Default::default()
            }
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label(
                        RichText::new(char::from(Icon::Search).to_string())
                            .family(FontFamily::Name("lucide".into()))
                            .size(12.0)
                            .color(self.theme.text_muted),
                    );
                    ui.add_sized(
                        vec2(search_width - 24.0, 20.0),
                        egui::TextEdit::singleline(&mut self.explorer_search)
                            .hint_text(RichText::new("Filter objects…").size(12.0).color(self.theme.text_muted))
                            .frame(false)
                            .text_color(self.theme.text_primary),
                    );
                });
            });

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
    }

    /// Empty state shown when no connections exist yet.
    fn draw_dbeaver_empty_state(&mut self, ui: &mut egui::Ui) {
        ui.add_space(36.0);
        ui.vertical_centered(|ui| {
            ui.label(
                RichText::new(char::from(Icon::Database).to_string())
                    .family(FontFamily::Name("lucide".into()))
                    .size(28.0)
                    .color(self.theme.text_muted),
            );
            ui.add_space(8.0);
            ui.label(RichText::new("No connections").strong().color(self.theme.text_primary));
            ui.add_space(3.0);
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

    /// Renders the list of connections as expandable root nodes (Codex / DBeaver style).
    fn draw_dbeaver_connections_tree(&mut self, ui: &mut egui::Ui) {
        let connections = self.connections.clone();
        for connection in connections {
            let is_active = self.active_connection_id.as_deref() == Some(&connection.id);
            let is_connected = self.connected && is_active;
            let is_connecting = self.pending_connection_request.is_some()
                && (self.pending_connection_id.as_deref() == Some(&connection.id)
                    || (self.pending_connection_id.is_none() && is_active));
            let is_failed = self.failed_connection_ids.contains(&connection.id);
            let err_msg = self.connection_errors.get(&connection.id).cloned();
            let id = ui.make_persistent_id(("codex_conn_node", &connection.id));

            let mut collapsing = egui::collapsing_header::CollapsingState::load_with_default_open(
                ui.ctx(),
                id,
                is_connected || is_failed,
            );
            let is_open = collapsing.is_open();

            let status_dot = if is_connected {
                Some(Color32::from_rgb(34, 197, 94)) // vibrant emerald green
            } else if is_connecting {
                Some(self.theme.accent)
            } else if is_failed {
                Some(Color32::from_rgb(239, 68, 68)) // red error badge / dot
            } else {
                Some(Color32::from_rgb(156, 163, 175)) // neutral slate gray
            };

            let badge_text = if is_failed {
                "ERR"
            } else if connection.driver.eq_ignore_ascii_case("postgresql") {
                "PG"
            } else {
                "SQLITE"
            };

            let (response, chevron_clicked) = draw_codex_tree_row(
                ui,
                &self.theme,
                CodexTreeRow {
                    depth: 0,
                    is_expandable: true,
                    is_expanded: is_open,
                    icon: Icon::Database,
                    icon_color: if is_connected {
                        self.theme.accent
                    } else if is_failed {
                        self.theme.danger
                    } else {
                        self.theme.text_muted
                    },
                    label: &connection.name,
                    is_selected: false,
                    is_dimmed: !is_connected && !is_failed && !is_connecting,
                    status_dot,
                    badge_text: Some(badge_text),
                    badge_accent: is_connected || is_failed,
                    count_text: None,
                    detail_text: None,
                },
            );

            let is_ctx = is_context_menu_triggered(&response, ui);
            let mut actions = connection_context_menu(ui, &response, is_connected, self.theme);

            if chevron_clicked {
                collapsing.set_open(!is_open);
                collapsing.store(ui.ctx());
            } else if response.clicked() && !is_ctx {
                if !is_connected {
                    if !is_connecting {
                        actions.connect = true;
                    }
                    collapsing.set_open(true);
                    collapsing.store(ui.ctx());
                } else {
                    collapsing.set_open(!is_open);
                    collapsing.store(ui.ctx());
                }
            }

            if collapsing.is_open() {
                if is_connected {
                    self.draw_dbeaver_connected_body(ui, &connection);
                } else if is_failed {
                    let err_str = err_msg.as_deref().unwrap_or("Connection failed");
                    let hint = format!("Failed: {} — Click to retry", err_str);
                    if draw_hint_row(ui, &self.theme, 1, Icon::AlertCircle, &hint).clicked() {
                        actions.connect = true;
                    }
                } else if is_connecting {
                    draw_hint_row(ui, &self.theme, 1, Icon::LoaderCircle, "Connecting…");
                } else if draw_hint_row(ui, &self.theme, 1, Icon::Circle, "Disconnected — click to connect").clicked()
                {
                    actions.connect = true;
                }
            }

            if actions.connect {
                self.connect_to_connection(&connection);
            }
            if actions.disconnect {
                self.disconnect_from_connection(&connection);
            }
            if actions.reconnect {
                self.disconnect_from_connection(&connection);
                self.connect_to_connection(&connection);
            }
            if actions.refresh {
                self.request_schema_introspection(connection.id.clone(), true);
            }
            if actions.new_script {
                self.new_query_document();
                self.active_tab = WorkspaceTab::Query;
            }
            if actions.er_diagram {
                self.active_tab = WorkspaceTab::Diagram;
                if !is_connected {
                    self.connect_to_connection(&connection);
                }
            }
            if actions.ask_agent {
                self.open_agent_prompt(
                    format!(
                        "Analyze the database `{}` on connection `{}` ({}) and describe the schema architecture.",
                        connection.database, connection.name, connection.driver
                    ),
                    ui.ctx(),
                );
            }
            if actions.create_table {
                self.new_query_document();
                self.query_text = format!(
                    "-- Create table on database `{}`\nCREATE TABLE new_table (\n    id SERIAL PRIMARY KEY,\n    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP\n);\n",
                    connection.database
                );
                self.active_tab = WorkspaceTab::Query;
            }
            if actions.copy_name {
                ui.output_mut(|o| o.copied_text = connection.name.clone());
                self.runtime_message = format!("Copied `{}` to clipboard", connection.name);
            }
            if actions.copy_conn_string {
                let conn_str = if connection.driver == "SQLite" {
                    connection.database.clone()
                } else {
                    format!(
                        "postgresql://{}@{}:{}/{}",
                        connection.username, connection.host, connection.port, connection.database
                    )
                };
                ui.output_mut(|o| o.copied_text = conn_str);
                self.runtime_message = "Copied connection string to clipboard".to_owned();
            }
            if actions.edit {
                self.open_edit_connection(&connection);
            }
            if actions.duplicate {
                self.open_duplicate_connection(&connection);
            }
            if actions.delete {
                self.delete_confirmation_id = Some(connection.id.clone());
            }
            ui.add_space(2.0);
        }
    }

    /// Body rendered when a connection node is expanded.
    fn draw_dbeaver_connected_body(&mut self, ui: &mut egui::Ui, connection: &UiConnectionSummary) {
        // Schema feedback (progress / error)
        self.draw_explorer_schema_feedback(ui);

        // Database node
        let db_id = ui.make_persistent_id(("codex_db_node", &connection.id, &connection.database));
        let db_name = if connection.database.is_empty() {
            "database".to_owned()
        } else {
            connection.database.clone()
        };

        let mut collapsing = egui::collapsing_header::CollapsingState::load_with_default_open(ui.ctx(), db_id, true);
        let is_open = collapsing.is_open();

        let (resp, chevron_clicked) = draw_codex_tree_row(
            ui,
            &self.theme,
            CodexTreeRow {
                depth: 1,
                is_expandable: true,
                is_expanded: is_open,
                icon: Icon::Database,
                icon_color: self.theme.accent,
                label: &db_name,
                is_selected: false,
                is_dimmed: false,
                status_dot: None,
                badge_text: None,
                badge_accent: false,
                count_text: None,
                detail_text: None,
            },
        );

        if resp.clicked() || chevron_clicked {
            collapsing.set_open(!is_open);
            collapsing.store(ui.ctx());
        }

        if collapsing.is_open() {
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
        }
    }

    /// A schema folder node inside the Database node (e.g. `public`).
    fn draw_dbeaver_schema_node(&mut self, ui: &mut egui::Ui, connection_id: &str, schema: &str) {
        let is_active_schema = self.active_schema() == schema;
        let schema_id = ui.make_persistent_id(("codex_schema_node", connection_id, schema));
        let table_count = self.schema.table_details.iter().filter(|t| t.schema == schema).count();

        let mut collapsing =
            egui::collapsing_header::CollapsingState::load_with_default_open(ui.ctx(), schema_id, is_active_schema);
        let is_open = collapsing.is_open();

        let (resp, chevron_clicked) = draw_codex_tree_row(
            ui,
            &self.theme,
            CodexTreeRow {
                depth: 2,
                is_expandable: true,
                is_expanded: is_open,
                icon: if is_open { Icon::FolderOpen } else { Icon::Folder },
                icon_color: if is_active_schema {
                    Color32::from_rgb(217, 119, 6) // warm amber
                } else {
                    self.theme.text_secondary
                },
                label: schema,
                is_selected: false,
                is_dimmed: !is_active_schema,
                status_dot: None,
                badge_text: None,
                badge_accent: false,
                count_text: if table_count > 0 {
                    Some(table_count.to_string())
                } else {
                    None
                },
                detail_text: None,
            },
        );

        let mut activate_schema = false;
        if chevron_clicked {
            collapsing.set_open(!is_open);
            collapsing.store(ui.ctx());
        } else if resp.clicked() {
            if !is_active_schema {
                activate_schema = true;
                collapsing.set_open(true);
                collapsing.store(ui.ctx());
            } else {
                collapsing.set_open(!is_open);
                collapsing.store(ui.ctx());
            }
        }

        if collapsing.is_open() {
            if is_active_schema {
                self.draw_dbeaver_schema_objects(ui, schema);
            } else if draw_hint_row(ui, &self.theme, 3, Icon::Circle, "Inactive schema — click to activate").clicked()
            {
                activate_schema = true;
            }
        }

        if activate_schema {
            self.activate_schema(schema);
        }
    }

    /// Activates a schema and clears the workspace state that depended on the old one.
    fn activate_schema(&mut self, schema: &str) {
        self.selected_schema = Some(schema.to_owned());
        self.selected_table = None;
        self.selected_schema_object = None;
        self.table_info = None;
        self.table_ddl = None;
        self.table_data_result = None;
        self.staged_changes.clear();
        self.staged_apply_targets.clear();
        self.table_mutation_error = None;
        self.activate_welcome_tab();
    }

    /// Renders the folders for a schema: Tables, Views, Functions, Triggers.
    fn draw_dbeaver_schema_objects(&mut self, ui: &mut egui::Ui, schema: &str) {
        let search_query = self.explorer_search.trim().to_ascii_lowercase();
        let all_tables = self.active_schema_table_names();
        let total_tables = all_tables.len();
        let (matching_table_count, tables) = filtered_explorer_tables(&all_tables, &search_query);

        self.draw_tables_folder(ui, schema, &tables, total_tables, matching_table_count, &search_query);

        let views = self.filter_by_schema(&self.schema.views, schema, |v| &v.schema);
        self.draw_dbeaver_views_folder(ui, &views);

        // Functions are PostgreSQL-only.
        if self.active_capabilities().is_some_and(|c| c.schema.functions) {
            let functions = self.filter_by_schema(&self.schema.functions, schema, |f| &f.schema);
            self.draw_dbeaver_functions_folder(ui, &functions);
        }

        let triggers = self.filter_by_schema(&self.schema.triggers, schema, |t| &t.schema);
        self.draw_dbeaver_triggers_folder(ui, &triggers);
    }

    /// Narrows schema-scoped objects to `schema`. When the backend reports no schema
    /// list (e.g. SQLite) everything belongs to a single flat namespace.
    fn filter_by_schema<T: Clone>(&self, all: &[T], schema: &str, schema_of: impl Fn(&T) -> &str) -> Vec<T> {
        if self.schema.schemas.is_empty() {
            all.to_vec()
        } else {
            all.iter().filter(|item| schema_of(item) == schema).cloned().collect()
        }
    }

    /// Tables folder. Unlike the other folders it reflects the active filter in both
    /// its count badge ("5/10") and its empty state.
    fn draw_tables_folder(
        &mut self,
        ui: &mut egui::Ui,
        schema: &str,
        tables: &[String],
        total_tables: usize,
        matching_table_count: usize,
        search_query: &str,
    ) {
        let folder_id = ui.make_persistent_id(("codex_tbl_folder", schema));
        let count_str = if search_query.is_empty() {
            total_tables.to_string()
        } else {
            format!("{matching_table_count}/{total_tables}")
        };

        let mut collapsing =
            egui::collapsing_header::CollapsingState::load_with_default_open(ui.ctx(), folder_id, true);
        let is_open = collapsing.is_open();

        let (response, chevron_clicked) = draw_codex_tree_row(
            ui,
            &self.theme,
            CodexTreeRow {
                depth: 3,
                is_expandable: true,
                is_expanded: is_open,
                icon: Icon::Table2,
                icon_color: Color32::from_rgb(37, 99, 235), // slate blue
                label: "Tables",
                is_selected: false,
                is_dimmed: total_tables == 0,
                status_dot: None,
                badge_text: None,
                badge_accent: false,
                count_text: Some(count_str),
                detail_text: None,
            },
        );

        if response.clicked() || chevron_clicked {
            collapsing.set_open(!is_open);
            collapsing.store(ui.ctx());
        }

        if !collapsing.is_open() {
            return;
        }
        if tables.is_empty() {
            let empty_label = if total_tables == 0 {
                "No tables in schema"
            } else {
                "No matching tables"
            };
            draw_hint_row(ui, &self.theme, 4, Icon::Info, empty_label);
        } else {
            for table in tables {
                self.draw_dbeaver_table_item(ui, table);
            }
        }
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

    /// Clears the connected state after an explicit disconnect.
    fn disconnect_from_connection(&mut self, connection: &UiConnectionSummary) {
        self.connected = false;
        self.schema = UiSchemaSummary::default();
        self.selected_table = None;
        self.selected_schema_object = None;
        self.runtime_message = format!("Disconnected from {}", connection.name);
    }

    /// Helper to initiate connection logic.
    pub(crate) fn connect_to_connection(&mut self, connection: &UiConnectionSummary) {
        self.reset_agent_context();
        self.active_connection_id = Some(connection.id.clone());
        self.pending_connection_id = Some(connection.id.clone());
        self.connection_errors.remove(&connection.id);
        self.failed_connection_ids.remove(&connection.id);
        self.selected_schema = None;
        self.schema = UiSchemaSummary::default();
        self.selected_table = None;
        self.selected_schema_object = None;
        self.reset_table_workspace_state();
        self.explorer_search.clear();
        let request_id = self.task_bridge.next_request_id();
        self.connected = false;
        self.pending_connection_request = Some(request_id);
        self.schema_request = None;
        self.schema_error = None;
        self.dispatch_command(UiCommand::Connect {
            request_id,
            connection_id: connection.id.clone(),
        });
        self.runtime_message = format!("Connecting to {}…", connection.name);
    }
}
