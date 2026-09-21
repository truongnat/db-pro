//! Explorer connection/schema tree rendering.
use super::explorer_connection_row_view::{ConnectionRowAction, ConnectionRowContext};
use super::explorer_database_node_view::DatabaseNodeContext;
use super::explorer_schema_node_view::SchemaNodeContext;
use super::explorer_schema_objects_view::{
    ExplorerSchemaObjectsAction, ExplorerSchemaObjectsModel, ExplorerSchemaObjectsView,
};
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
            super::schema_explorer_state::SchemaActivationContext::new(
                &mut self.schema.explorer,
                &mut self.table,
                &mut self.workspace,
                &mut self.feedback,
            )
            .activate(schema);
        }
    }

    /// Renders the folders for a schema: Tables, Views, Functions, Triggers.
    pub(super) fn draw_dbeaver_schema_objects(&mut self, ui: &mut egui::Ui, schema: &str) {
        let connection_id = self
            .connection
            .lifecycle
            .active_connection_id()
            .unwrap_or_default()
            .to_owned();
        let selected_table = self.schema.explorer.selected_table.clone();
        let table_info = self.table.state.table_info.clone();
        let functions_enabled = self.active_capabilities().allows(|c| c.schema.functions);
        let actions = ExplorerSchemaObjectsView::new(
            self.theme,
            &mut self.schema.explorer,
            ExplorerSchemaObjectsModel {
                selected_table,
                table_info,
                connection_id,
                functions_enabled,
            },
        )
        .draw(ui, schema);

        for action in actions {
            match action {
                ExplorerSchemaObjectsAction::SelectTable(table) => self.select_table(&table),
                ExplorerSchemaObjectsAction::TableRow { table, action } => {
                    let schema = self.active_schema().to_owned();
                    self.apply_table_row_action(action, &table, &schema, ui);
                }
                ExplorerSchemaObjectsAction::SchemaObject(action) => {
                    self.apply_schema_object_folder_action(action, ui);
                }
            }
        }
    }
}
