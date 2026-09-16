//! Explorer connection/schema tree rendering.
use super::explorer_tree::{draw_codex_tree_row, draw_hint_row, CodexTreeRow};
use super::explorer_view::connection_context_menu;
use super::*;

impl DbProApp {
    pub(super) fn draw_dbeaver_connections_tree(&mut self, ui: &mut egui::Ui) {
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
                Some(self.theme.success) // connection status dot belongs to the theme
            } else if is_connecting {
                Some(self.theme.accent)
            } else if is_failed {
                Some(self.theme.danger)
            } else {
                Some(self.theme.text_muted) // neutral, follows the active theme
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
                self.set_active_query_text(format!(
                    "-- Create table on database `{}`\nCREATE TABLE new_table (\n    id SERIAL PRIMARY KEY,\n    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP\n);\n",
                    connection.database
                ));
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
    pub(super) fn draw_dbeaver_connected_body(&mut self, ui: &mut egui::Ui, connection: &UiConnectionSummary) {
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
    pub(super) fn draw_dbeaver_schema_node(&mut self, ui: &mut egui::Ui, connection_id: &str, schema: &str) {
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
                    self.theme.warning // warm amber for the active schema
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
    pub(crate) fn activate_schema(&mut self, schema: &str) {
        if self.selected_schema.as_deref() == Some(schema) {
            return;
        }
        if !self.staged_changes.is_empty() {
            self.pending_navigation_action = Some(PendingNavigationAction::ChangeSchema(schema.to_owned()));
            self.discard_changes_confirmation = true;
            self.runtime_message = "Apply or discard staged changes before changing schema".to_owned();
            return;
        }
        self.pending_navigation_action = None;
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
    pub(super) fn draw_dbeaver_schema_objects(&mut self, ui: &mut egui::Ui, schema: &str) {
        let search_query = self.explorer_search.trim().to_ascii_lowercase();
        let all_tables = self.active_schema_table_names();
        let total_tables = all_tables.len();
        let (matching_table_count, tables) = filtered_explorer_tables(&all_tables, &search_query);

        self.draw_tables_folder(ui, schema, &tables, total_tables, matching_table_count, &search_query);

        let views = self.filter_by_schema(&self.schema.views, schema, |v| &v.schema);
        self.draw_dbeaver_views_folder(ui, &views);

        // Capability-gated, not driver-gated: the folder appears when the
        // provider reports routines (PostgreSQL and MySQL do, SQLite does not).
        if self.active_capabilities().allows(|c| c.schema.functions) {
            let functions = self.filter_by_schema(&self.schema.functions, schema, |f| &f.schema);
            self.draw_dbeaver_functions_folder(ui, &functions);
        }

        let triggers = self.filter_by_schema(&self.schema.triggers, schema, |t| &t.schema);
        self.draw_dbeaver_triggers_folder(ui, &triggers);
    }

    /// Narrows schema-scoped objects to `schema`. When the backend reports no schema
    /// list (e.g. SQLite) everything belongs to a single flat namespace.
    pub(super) fn filter_by_schema<T: Clone>(&self, all: &[T], schema: &str, schema_of: impl Fn(&T) -> &str) -> Vec<T> {
        if self.schema.schemas.is_empty() {
            all.to_vec()
        } else {
            all.iter().filter(|item| schema_of(item) == schema).cloned().collect()
        }
    }

    /// Tables folder. Unlike the other folders it reflects the active filter in both
    /// its count badge ("5/10") and its empty state.
    pub(super) fn draw_tables_folder(
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
                icon_color: self.theme.info,
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
}
