//! Explorer connection/schema tree rendering.
use super::explorer_connection_node_view::{
    ExplorerConnectionNodeAction, ExplorerConnectionNodeModel, ExplorerConnectionNodeView,
};
use super::explorer_connection_row_view::ConnectionRowAction;
use super::explorer_schema_objects_view::ExplorerSchemaObjectsAction;
use super::explorer_schema_tree_view::{ExplorerSchemaTreeAction, ExplorerSchemaTreeModel};
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
            let is_active = self.connection.lifecycle.active_connection_id() == Some(connection.id.as_str());
            let is_connected = self.connection.lifecycle.is_connected() && is_active;
            let schema_model = is_connected.then(|| self.schema_tree_model(&connection));
            let model = ExplorerConnectionNodeModel {
                is_connected,
                is_connecting: self.is_connection_connecting(&connection, is_active),
                is_failed: self.connection.lifecycle.has_failed_connection(&connection.id),
                error: self
                    .connection
                    .lifecycle
                    .connection_error(&connection.id)
                    .map(str::to_owned),
            };
            let actions =
                ExplorerConnectionNodeView::new(self.theme, &connection, model, Self::primary_modifier_label())
                    .draw(ui, schema_model.map(|model| (&mut self.schema.explorer, model)));

            for action in actions {
                match action {
                    ExplorerConnectionNodeAction::Connection(action) => {
                        self.apply_connection_row_action(action, &connection, is_connected, ui);
                    }
                    ExplorerConnectionNodeAction::Schema(action) => {
                        self.apply_schema_tree_action(action, &connection.id, ui);
                    }
                }
            }
            ui.add_space(2.0);
        }
    }

    fn is_connection_connecting(&self, connection: &UiConnectionSummary, is_active: bool) -> bool {
        self.connection.lifecycle.pending_request().is_some()
            && (self.connection.lifecycle.pending_connection_id() == Some(connection.id.as_str())
                || (self.connection.lifecycle.pending_connection_id().is_none() && is_active))
    }

    fn schema_tree_model(&self, connection: &UiConnectionSummary) -> ExplorerSchemaTreeModel {
        ExplorerSchemaTreeModel {
            connection_id: connection.id.clone(),
            database: connection.database.clone(),
            active_schema: self.active_schema().to_owned(),
            schema_error: self.schema.explorer.schema_error.clone(),
            schema_loading: self.schema.explorer.schema_request.is_some(),
            reduce_motion: self.preferences.reduce_motion,
            selected_table: self.schema.explorer.selected_table.clone(),
            table_info: self.table.state.table_info.clone(),
            functions_enabled: self.active_capabilities().allows(|c| c.schema.functions),
        }
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

    fn apply_schema_tree_action(&mut self, action: ExplorerSchemaTreeAction, connection_id: &str, ui: &mut egui::Ui) {
        match action {
            ExplorerSchemaTreeAction::RefreshSchema => {
                self.request_schema_introspection(connection_id.to_owned(), true);
            }
            ExplorerSchemaTreeAction::ActivateSchema(schema) => {
                super::schema_explorer_state::SchemaActivationContext::new(
                    &mut self.schema.explorer,
                    &mut self.table,
                    &mut self.workspace,
                    &mut self.feedback,
                )
                .activate(&schema);
            }
            ExplorerSchemaTreeAction::SchemaObjects(action) => {
                self.apply_schema_objects_action(action, ui);
            }
        }
    }

    fn apply_schema_objects_action(&mut self, action: ExplorerSchemaObjectsAction, ui: &mut egui::Ui) {
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
