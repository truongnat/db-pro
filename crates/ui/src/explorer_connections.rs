//! Explorer connection/schema tree rendering.
use super::explorer_connection_row_view::{ConnectionRowAction, ConnectionRowContext};
use super::explorer_database_node_view::DatabaseNodeContext;
use super::explorer_schema_node_view::SchemaNodeContext;
use super::explorer_table_folder_view::TableFolderContext;
use super::explorer_tree::draw_hint_row;
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
        let connection_count = self.connection.catalog.len();
        for index in 0..connection_count {
            let Some(connection) = self.connection.catalog.get(index).cloned() else {
                continue;
            };
            self.draw_connection_node(ui, connection);
        }
    }

    fn draw_connection_node(&mut self, ui: &mut egui::Ui, connection: UiConnectionSummary) {
        let is_active = self.connection.lifecycle.active_connection_id() == Some(&connection.id);
        let is_connected = self.connection.lifecycle.is_connected() && is_active;
        let is_connecting = self.connection.lifecycle.pending_request().is_some()
            && (self.connection.lifecycle.pending_connection_id() == Some(connection.id.as_str())
                || (self.connection.lifecycle.pending_connection_id().is_none() && is_active));
        let is_failed = self.connection.lifecycle.has_failed_connection(&connection.id);
        let err_msg = self
            .connection
            .lifecycle
            .connection_error(&connection.id)
            .map(str::to_owned);
        let render = ConnectionRowContext {
            theme: self.theme,
            connection: &connection,
            is_connected,
            is_connecting,
            is_failed,
            modifier: Self::primary_modifier_label(),
        }
        .draw(ui);
        let mut actions = render.actions;

        if render.is_open {
            if is_connected {
                self.draw_dbeaver_connected_body(ui, &connection);
            } else if is_failed {
                let err_str = err_msg.as_deref().unwrap_or("Connection failed");
                let hint = format!("Failed: {} — Click to retry", err_str);
                if draw_hint_row(ui, &self.theme, 1, Icon::AlertCircle, &hint).clicked() {
                    actions.push(ConnectionRowAction::Connect);
                }
            } else if is_connecting {
                draw_hint_row(ui, &self.theme, 1, Icon::LoaderCircle, "Connecting…");
            } else if draw_hint_row(ui, &self.theme, 1, Icon::Circle, "Disconnected — click to connect").clicked() {
                actions.push(ConnectionRowAction::Connect);
            }
        }

        for action in actions {
            self.apply_connection_row_action(action, &connection, is_connected, ui);
        }
        ui.add_space(2.0);
    }

    fn apply_connection_row_action(
        &mut self,
        action: ConnectionRowAction,
        connection: &UiConnectionSummary,
        is_connected: bool,
        ui: &mut egui::Ui,
    ) {
        match action {
            ConnectionRowAction::Connect => self.connect_to_connection(connection),
            ConnectionRowAction::Disconnect => self.disconnect_from_connection(connection),
            ConnectionRowAction::Reconnect => {
                self.disconnect_from_connection(connection);
                self.connect_to_connection(connection);
            }
            ConnectionRowAction::RefreshSchema => self.request_schema_introspection(connection.id.clone(), true),
            ConnectionRowAction::NewScript => {
                self.new_query_document();
                self.workspace.active_tab = WorkspaceTab::Query;
            }
            ConnectionRowAction::OpenErDiagram => {
                self.workspace.active_tab = WorkspaceTab::Diagram;
                if !is_connected {
                    self.connect_to_connection(connection);
                }
            }
            ConnectionRowAction::AskAgent => self.open_agent_prompt(
                format!(
                    "Analyze the database `{}` on connection `{}` ({}) and describe the schema architecture.",
                    connection.database, connection.name, connection.driver
                ),
                ui.ctx(),
            ),
            ConnectionRowAction::CreateTable => {
                self.new_query_document();
                self.set_active_query_text(format!(
                    "-- Create table on database `{}`\nCREATE TABLE new_table (\n    id SERIAL PRIMARY KEY,\n    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP\n);\n",
                    connection.database
                ));
                self.workspace.active_tab = WorkspaceTab::Query;
            }
            ConnectionRowAction::CopyName => {
                ui.output_mut(|output| output.copied_text = connection.name.clone());
                self.feedback.runtime_message = format!("Copied `{}` to clipboard", connection.name);
            }
            ConnectionRowAction::CopyConnectionString => {
                ui.output_mut(|output| output.copied_text = connection_display_uri(connection));
                self.feedback.runtime_message = "Copied connection string to clipboard".to_owned();
            }
            ConnectionRowAction::Edit => self.open_edit_connection(connection),
            ConnectionRowAction::Duplicate => self.open_duplicate_connection(connection),
            ConnectionRowAction::Delete => self.overlay.delete_confirmation_id = Some(connection.id.clone()),
        }
    }

    /// Body rendered when a connection node is expanded.
    pub(super) fn draw_dbeaver_connected_body(&mut self, ui: &mut egui::Ui, connection: &UiConnectionSummary) {
        // Schema feedback (progress / error)
        self.draw_explorer_schema_feedback(ui);

        // Database node
        let database_open = DatabaseNodeContext {
            theme: self.theme,
            connection_id: &connection.id,
            database: &connection.database,
        }
        .draw(ui);

        if database_open {
            let schema_count = self.schema.explorer.schema.schemas.len();
            if schema_count == 0 {
                // Flat tables/views (e.g. SQLite)
                self.draw_dbeaver_schema_objects(ui, "");
            } else {
                // Nested Schemas (e.g. PostgreSQL: public, information_schema, etc.)
                for index in 0..schema_count {
                    let schema = self.schema.explorer.schema.schemas[index].clone();
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
        let table_count = self.schema_table_count(schema);
        let render = SchemaNodeContext {
            theme: self.theme,
            connection_id,
            schema,
            is_active: is_active_schema,
            table_count,
        }
        .draw(ui);

        let mut activate_schema = render.should_activate;
        if render.is_open {
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
        if self.schema.explorer.selected_schema.as_deref() == Some(schema) {
            return;
        }
        if !self.table.mutation.staged_changes.is_empty() {
            self.workspace.pending_navigation_action = Some(PendingNavigationAction::ChangeSchema(schema.to_owned()));
            self.table.editing.discard_changes_confirmation = true;
            self.feedback.runtime_message = "Apply or discard staged changes before changing schema".to_owned();
            return;
        }
        self.workspace.pending_navigation_action = None;
        self.schema.explorer.selected_schema = Some(schema.to_owned());
        self.schema.explorer.selected_table = None;
        self.schema.explorer.selected_schema_object = None;
        self.schema.explorer.schema_object_view = SchemaObjectView::Definition;
        self.table.reset_workspace();
        self.schema.explorer.explorer_nav_cache = None;
        self.activate_welcome_tab();
    }

    /// Renders the folders for a schema: Tables, Views, Functions, Triggers.
    pub(super) fn draw_dbeaver_schema_objects(&mut self, ui: &mut egui::Ui, schema: &str) {
        let search_query = self.schema.explorer.explorer_search.trim().to_ascii_lowercase();
        // Counts are O(n) but allocate nothing; materialised lists are deferred until a
        // folder is actually open (see folder bodies below / Tables drawer).
        let total_tables = self.schema_table_count(schema);
        self.draw_tables_folder(ui, schema, total_tables, &search_query);

        let view_count = self
            .schema
            .explorer
            .count_by_schema(&self.schema.explorer.schema.views, schema, |view| &view.schema);
        self.draw_dbeaver_views_folder_lazy(ui, schema, view_count);

        if self.active_capabilities().allows(|c| c.schema.functions) {
            let function_count =
                self.schema
                    .explorer
                    .count_by_schema(&self.schema.explorer.schema.functions, schema, |function| {
                        &function.schema
                    });
            self.draw_dbeaver_functions_folder_lazy(ui, schema, function_count);
        }

        let trigger_count =
            self.schema
                .explorer
                .count_by_schema(&self.schema.explorer.schema.triggers, schema, |trigger| &trigger.schema);
        self.draw_dbeaver_triggers_folder_lazy(ui, schema, trigger_count);
    }

    /// Narrows schema-scoped objects to `schema`. When the backend reports no schema
    /// list (e.g. SQLite) everything belongs to a single flat namespace.
    /// Tables folder. Unlike the other folders it reflects the active filter in both
    /// its count badge ("5/10") and its empty state.
    pub(super) fn draw_tables_folder(
        &mut self,
        ui: &mut egui::Ui,
        schema: &str,
        total_tables: usize,
        search_query: &str,
    ) {
        let matching_table_count = if search_query.is_empty() {
            total_tables
        } else if let Some(cache) = self.schema.explorer.explorer_nav_cache.as_ref().filter(|cache| {
            cache.schema == schema
                && cache.search == search_query
                && cache.connection_id == self.connection.lifecycle.active_connection_id().unwrap_or_default()
        }) {
            cache.matching_count
        } else {
            self.schema.explorer.matching_table_count(schema, search_query)
        };
        let folder = TableFolderContext {
            theme: self.theme,
            schema,
            total_tables,
            matching_tables: matching_table_count,
            search_query,
        };
        let render = folder.draw_header(ui);
        if !render.is_open {
            return;
        }

        let connection_id = self.connection.lifecycle.active_connection_id().unwrap_or_default();
        let (_total, _matching, tables) = self.schema.explorer.cached_tables(connection_id, schema, search_query);
        if tables.is_empty() {
            folder.draw_empty_state(ui);
            return;
        }

        let clip = ui.clip_rect();
        for table in &tables {
            let row_top = ui.cursor().min.y;
            let row_bottom = row_top + EXPLORER_ROW_HEIGHT;
            if row_bottom < clip.top() || row_top > clip.bottom() {
                // Keep layout height without painting off-screen rows.
                folder.draw_offscreen_row_spacer(ui);
                continue;
            }
            self.draw_dbeaver_table_item(ui, table);
        }

        folder.draw_overflow_hint(ui, tables.len());
    }
}
