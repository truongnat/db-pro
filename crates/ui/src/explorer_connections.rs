//! Explorer connection/schema tree rendering.
use super::explorer_connection_row_view::ConnectionRowAction;
use super::explorer_schema_objects_view::ExplorerSchemaObjectsAction;
use super::explorer_schema_tree_view::ExplorerSchemaTreeAction;
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
    pub(super) fn apply_connection_row_action(
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

    pub(super) fn apply_schema_tree_action(
        &mut self,
        action: ExplorerSchemaTreeAction,
        connection_id: &str,
        ui: &mut egui::Ui,
    ) {
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
            ExplorerSchemaObjectsAction::SelectTable(table) => self.open_table(table),
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
