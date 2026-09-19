//! Explorer connection/schema tree rendering.
use super::explorer_tree::{draw_codex_tree_row, draw_hint_row, CodexTreeRow};
use super::explorer_view::connection_context_menu;
use super::*;

/// Driver-aware URI suitable for "Copy Connection String" in the explorer.
fn connection_display_uri(connection: &UiConnectionSummary) -> String {
    let driver = connection.driver.to_ascii_lowercase();
    if driver.contains("sqlite") {
        return connection.database.clone();
    }
    let scheme = if driver.contains("mysql") {
        "mysql"
    } else if driver.contains("sqlserver") || driver.contains("mssql") {
        "sqlserver"
    } else {
        "postgresql"
    };
    format!(
        "{scheme}://{}@{}:{}/{}",
        connection.username, connection.host, connection.port, connection.database
    )
}

impl DbProApp {
    pub(super) fn draw_dbeaver_connections_tree(&mut self, ui: &mut egui::Ui) {
        let connection_count = self.connection_catalog.connections.len();
        for index in 0..connection_count {
            let connection = self.connection_catalog.connections[index].clone();
            let is_active = self.connection_lifecycle.active_connection_id.as_deref() == Some(&connection.id);
            let is_connected = self.connected && is_active;
            let is_connecting = self.connection_lifecycle.pending_request.is_some()
                && (self.connection_lifecycle.pending_connection_id.as_deref() == Some(&connection.id)
                    || (self.connection_lifecycle.pending_connection_id.is_none() && is_active));
            let is_failed = self.connection_lifecycle.failed_connection_ids.contains(&connection.id);
            let err_msg = self.connection_lifecycle.errors.get(&connection.id).cloned();
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
                self.workspace.active_tab = WorkspaceTab::Query;
            }
            if actions.er_diagram {
                self.workspace.active_tab = WorkspaceTab::Diagram;
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
                self.workspace.active_tab = WorkspaceTab::Query;
            }
            if actions.copy_name {
                ui.output_mut(|o| o.copied_text = connection.name.clone());
                self.runtime_message = format!("Copied `{}` to clipboard", connection.name);
            }
            if actions.copy_conn_string {
                let conn_str = connection_display_uri(&connection);
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
            let schema_count = self.schema.schemas.len();
            if schema_count == 0 {
                // Flat tables/views (e.g. SQLite)
                self.draw_dbeaver_schema_objects(ui, "");
            } else {
                // Nested Schemas (e.g. PostgreSQL: public, information_schema, etc.)
                for index in 0..schema_count {
                    let schema = self.schema.schemas[index].clone();
                    if !is_user_visible_schema(&schema) {
                        continue;
                    }
                    self.draw_dbeaver_schema_node(ui, &connection.id, &schema);
                }
            }
        }
    }

    /// A schema folder node inside the Database node (e.g. `public`).
    pub(super) fn draw_dbeaver_schema_node(&mut self, ui: &mut egui::Ui, connection_id: &str, schema: &str) {
        let is_active_schema = self.active_schema() == schema;
        let schema_id = ui.make_persistent_id(("codex_schema_node", connection_id, schema));
        let table_count = self.schema_table_count(schema);

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
            self.workspace.pending_navigation_action = Some(PendingNavigationAction::ChangeSchema(schema.to_owned()));
            self.discard_changes_confirmation = true;
            self.runtime_message = "Apply or discard staged changes before changing schema".to_owned();
            return;
        }
        self.workspace.pending_navigation_action = None;
        self.selected_schema = Some(schema.to_owned());
        self.selected_table = None;
        self.selected_schema_object = None;
        self.table_info = None;
        self.table_ddl = None;
        self.table_data_result = None;
        self.staged_changes.clear();
        self.staged_apply_targets.clear();
        self.table_mutation_error = None;
        self.explorer_nav_cache = None;
        self.activate_welcome_tab();
    }

    /// Renders the folders for a schema: Tables, Views, Functions, Triggers.
    pub(super) fn draw_dbeaver_schema_objects(&mut self, ui: &mut egui::Ui, schema: &str) {
        let search_query = self.explorer_search.trim().to_ascii_lowercase();
        // Counts are O(n) but allocate nothing; materialised lists are deferred until a
        // folder is actually open (see folder bodies below / Tables drawer).
        let total_tables = self.schema_table_count(schema);
        self.draw_tables_folder(ui, schema, total_tables, &search_query);

        let view_count = self.count_by_schema(&self.schema.views, schema, |v| &v.schema);
        self.draw_dbeaver_views_folder_lazy(ui, schema, view_count);

        if self.active_capabilities().allows(|c| c.schema.functions) {
            let function_count = self.count_by_schema(&self.schema.functions, schema, |f| &f.schema);
            self.draw_dbeaver_functions_folder_lazy(ui, schema, function_count);
        }

        let trigger_count = self.count_by_schema(&self.schema.triggers, schema, |t| &t.schema);
        self.draw_dbeaver_triggers_folder_lazy(ui, schema, trigger_count);
    }

    /// Narrows schema-scoped objects to `schema`. When the backend reports no schema
    /// list (e.g. SQLite) everything belongs to a single flat namespace.
    pub(super) fn filter_by_schema<T: Clone>(&self, all: &[T], schema: &str, schema_of: impl Fn(&T) -> &str) -> Vec<T> {
        if self.schema.schemas.is_empty() || schema.is_empty() {
            all.to_vec()
        } else {
            all.iter().filter(|item| schema_of(item) == schema).cloned().collect()
        }
    }

    pub(super) fn count_by_schema<T>(&self, all: &[T], schema: &str, schema_of: impl Fn(&T) -> &str) -> usize {
        if self.schema.schemas.is_empty() || schema.is_empty() {
            all.len()
        } else {
            all.iter().filter(|item| schema_of(item) == schema).count()
        }
    }

    /// Returns cached visible table names for `schema` + current search.
    fn cached_explorer_tables(&mut self, schema: &str, search_query: &str) -> (usize, usize, Vec<String>) {
        let connection_id = self
            .connection_lifecycle
            .active_connection_id
            .clone()
            .unwrap_or_default();
        if let Some(cache) = self.explorer_nav_cache.as_ref() {
            if cache.connection_id == connection_id && cache.schema == schema && cache.search == search_query {
                return (cache.total_count, cache.matching_count, cache.visible.clone());
            }
        }

        let all_tables = self.schema_table_names(schema);
        let (matching_count, visible) = filtered_explorer_tables(&all_tables, search_query);
        let total_count = all_tables.len();
        self.explorer_nav_cache = Some(ExplorerNavCache {
            connection_id,
            schema: schema.to_owned(),
            search: search_query.to_owned(),
            total_count,
            matching_count,
            visible: visible.clone(),
        });
        (total_count, matching_count, visible)
    }

    /// Tables folder. Unlike the other folders it reflects the active filter in both
    /// its count badge ("5/10") and its empty state.
    pub(super) fn draw_tables_folder(
        &mut self,
        ui: &mut egui::Ui,
        schema: &str,
        total_tables: usize,
        search_query: &str,
    ) {
        let folder_id = ui.make_persistent_id(("codex_tbl_folder", schema));
        let matching_table_count = if search_query.is_empty() {
            total_tables
        } else if let Some(cache) = self.explorer_nav_cache.as_ref().filter(|cache| {
            cache.schema == schema
                && cache.search == search_query
                && cache.connection_id
                    == self
                        .connection_lifecycle
                        .active_connection_id
                        .as_deref()
                        .unwrap_or_default()
        }) {
            cache.matching_count
        } else {
            self.schema_matching_table_count(schema, search_query)
        };
        let count_str = if search_query.is_empty() {
            total_tables.to_string()
        } else {
            format!("{matching_table_count}/{total_tables}")
        };

        // Default closed for large schemas; auto-open while the user is filtering.
        let default_open = !search_query.is_empty();
        let mut collapsing =
            egui::collapsing_header::CollapsingState::load_with_default_open(ui.ctx(), folder_id, default_open);
        if default_open && !collapsing.is_open() {
            collapsing.set_open(true);
            collapsing.store(ui.ctx());
        }
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

        let (_total, matching, tables) = self.cached_explorer_tables(schema, search_query);
        if tables.is_empty() {
            let empty_label = if total_tables == 0 {
                "No tables in schema"
            } else {
                "No matching tables"
            };
            draw_hint_row(ui, &self.theme, 4, Icon::Info, empty_label);
            return;
        }

        let clip = ui.clip_rect();
        for table in &tables {
            let row_top = ui.cursor().min.y;
            let row_bottom = row_top + EXPLORER_ROW_HEIGHT;
            if row_bottom < clip.top() || row_top > clip.bottom() {
                // Keep layout height without painting off-screen rows.
                let _ = ui.allocate_exact_size(
                    egui::vec2(ui.available_width(), EXPLORER_ROW_HEIGHT),
                    egui::Sense::hover(),
                );
                continue;
            }
            self.draw_dbeaver_table_item(ui, table);
        }

        if matching > tables.len() {
            draw_hint_row(
                ui,
                &self.theme,
                4,
                Icon::Ellipsis,
                &format!("Showing {} of {matching} — refine filter", tables.len()),
            );
        }
    }
}
