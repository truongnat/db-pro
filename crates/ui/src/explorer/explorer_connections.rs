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

/// Sample CREATE TABLE for the connection driver. This is not a table designer.
fn sample_create_table_sql(driver: &str, database: &str) -> String {
    let driver_key = driver.to_ascii_lowercase();
    let (label, id_column, created_column) = if driver_key.contains("sqlite") {
        (
            "SQLite",
            "id INTEGER PRIMARY KEY",
            "created_at TEXT DEFAULT CURRENT_TIMESTAMP",
        )
    } else if driver_key.contains("mysql") {
        (
            "MySQL",
            "id INT PRIMARY KEY AUTO_INCREMENT",
            "created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP",
        )
    } else if driver_key.contains("sqlserver") || driver_key.contains("mssql") || driver_key.contains("sql server") {
        (
            "SQL Server",
            "id INT IDENTITY(1,1) PRIMARY KEY",
            "created_at DATETIME2 DEFAULT SYSUTCDATETIME()",
        )
    } else if driver_key.contains("postgres") {
        (
            "PostgreSQL",
            "id SERIAL PRIMARY KEY",
            "created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP",
        )
    } else {
        (driver, "id INTEGER PRIMARY KEY", "created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP")
    };
    format!(
        "-- Sample SQL for {label} on database `{database}`\nCREATE TABLE new_table (\n    {id_column},\n    {created_column}\n);\n"
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
                self.open_query_on_connection(&connection.id);
                self.workspace.active_tab = WorkspaceTab::Query;
            }
            ConnectionRowAction::OpenErDiagram => {
                self.open_diagram_tab();
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
                self.open_query_on_connection(&connection.id);
                self.set_active_query_text(sample_create_table_sql(&connection.driver, &connection.database));
                self.workspace.active_tab = WorkspaceTab::Query;
            }
            ConnectionRowAction::CopyName => {
                ui.output_mut(|output| output.commands.push(egui::OutputCommand::CopyText(connection.name.clone())));
                self.feedback.runtime_message = format!("Copied `{}` to clipboard", connection.name);
            }
            ConnectionRowAction::CopyConnectionString => {
                ui.output_mut(|output| output.commands.push(egui::OutputCommand::CopyText(connection_display_uri(connection))));
                self.feedback.runtime_message = "Copied connection string to clipboard".to_owned();
            }
            ConnectionRowAction::Edit => self.open_edit_connection(connection),
            ConnectionRowAction::Duplicate => self.open_duplicate_connection(connection),
            ConnectionRowAction::Delete => self.overlay.connection_delete_confirmation_id = Some(connection.id.clone()),
        }
    }

    pub(super) fn open_query_on_connection(&mut self, connection_id: &str) {
        self.new_query_document();
        let index = self.query.session.active_document_index;
        self.set_document_connection(index, Some(connection_id.to_owned()));
    }

    pub(super) fn apply_schema_tree_action(
        &mut self,
        action: ExplorerSchemaTreeAction,
        connection_id: &str,
        ui: &mut egui::Ui,
    ) {
        match action {
            ExplorerSchemaTreeAction::RefreshSchema { schema } => {
                self.refresh_explorer_node(connection_id, schema.as_deref(), None);
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
            ExplorerSchemaTreeAction::OpenErDiagram => {
                self.open_diagram_tab();
            }
            ExplorerSchemaTreeAction::CopySchemaName(schema) => {
                ui.output_mut(|output| output.commands.push(egui::OutputCommand::CopyText(schema.clone())));
                self.feedback.runtime_message = format!("Copied `{schema}` to clipboard");
            }
            ExplorerSchemaTreeAction::DropSchema(schema) => self.plan_drop_schema(&schema),
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::UiSslMode;

    fn connection(id: &str, database: &str) -> UiConnectionSummary {
        UiConnectionSummary {
            id: id.to_owned(),
            name: id.to_owned(),
            host: "localhost".to_owned(),
            port: 5432,
            database: database.to_owned(),
            username: "postgres".to_owned(),
            driver: "PostgreSQL".to_owned(),
            ssl_mode: UiSslMode::Require,
            readonly: false,
            tags: Vec::new(),
            group: None,
            favorite: false,
            environment: "Development".to_owned(),
        }
    }

    fn run_row_action(app: &mut DbProApp, action: ConnectionRowAction, connection: &UiConnectionSummary) {
        let ctx = egui::Context::default();
        let _ = crate::test_frame::frame(&ctx, Default::default(), |ui| {
            app.apply_connection_row_action(action, connection, false, ui);
        });
    }

    #[test]
    fn tree_query_uses_clicked_connection() {
        let (bridge, _command_rx, _event_tx) = crate::TaskBridge::with_channels();
        let mut app = DbProApp::with_task_bridge(bridge);
        app.connection
            .lifecycle
            .set_active_connection_id(Some("active-conn".to_owned()));

        app.new_query_document();
        assert_eq!(app.active_query_connection_id(), Some("active-conn"));

        let clicked = connection("clicked-conn", "clicked_db");
        run_row_action(&mut app, ConnectionRowAction::NewScript, &clicked);
        assert_eq!(app.active_query_connection_id(), Some("clicked-conn"));
        assert_eq!(app.workspace.active_tab, WorkspaceTab::Query);

        let table = connection("table-conn", "table_db");
        run_row_action(&mut app, ConnectionRowAction::CreateTable, &table);
        assert_eq!(app.active_query_connection_id(), Some("table-conn"));
        assert!(app.query.session.active_text().contains("table_db"));

        let source = include_str!("explorer_connections.rs");
        let needle = concat!("self.open_query_on_connection(", "&connection.id)");
        assert_eq!(source.matches(needle).count(), 2);
    }

    #[test]
    fn disconnect_closes_handle_and_reconnect_opens_a_new_connection() {
        let (bridge, command_rx, _event_tx) = crate::TaskBridge::with_channels();
        let mut app = DbProApp::with_task_bridge(bridge);
        app.connection
            .lifecycle
            .set_active_connection_id(Some("clicked-conn".to_owned()));
        app.connection.lifecycle.set_connected(true);
        let clicked = connection("clicked-conn", "clicked_db");

        run_row_action(&mut app, ConnectionRowAction::Disconnect, &clicked);
        assert!(!app.connection.lifecycle.is_connected());
        assert!(matches!(
            command_rx.try_recv(),
            Ok(UiCommand::Disconnect { connection_id, .. }) if connection_id == "clicked-conn"
        ));
        assert!(command_rx.try_recv().is_err());

        app.connection.lifecycle.set_connected(true);
        run_row_action(&mut app, ConnectionRowAction::Reconnect, &clicked);
        assert!(matches!(
            command_rx.try_recv(),
            Ok(UiCommand::Disconnect { connection_id, .. }) if connection_id == "clicked-conn"
        ));
        assert!(matches!(
            command_rx.try_recv(),
            Ok(UiCommand::Connect { connection_id, .. }) if connection_id == "clicked-conn"
        ));
    }

    #[test]
    fn create_table_sample_sql_follows_the_driver() {
        let postgres = sample_create_table_sql("PostgreSQL", "app");
        assert!(postgres.contains("Sample SQL for PostgreSQL"));
        assert!(postgres.contains("id SERIAL PRIMARY KEY"));
        assert!(!postgres.contains("AUTO_INCREMENT"));
        assert!(!postgres.contains("IDENTITY"));

        let mysql = sample_create_table_sql("MySQL", "app");
        assert!(mysql.contains("AUTO_INCREMENT"));
        assert!(!mysql.contains("SERIAL"));

        let sqlite = sample_create_table_sql("SQLite", "app.db");
        assert!(sqlite.contains("INTEGER PRIMARY KEY"));
        assert!(!sqlite.contains("SERIAL"));

        let sql_server = sample_create_table_sql("SQL Server", "sales");
        assert!(sql_server.contains("IDENTITY(1,1)"));
        assert!(sql_server.contains("SYSUTCDATETIME()"));
        assert!(!sql_server.contains("SERIAL"));

        let (bridge, _command_rx, _event_tx) = crate::TaskBridge::with_channels();
        let mut app = DbProApp::with_task_bridge(bridge);
        let mut server = connection("sqlserver-conn", "sales");
        server.driver = "SQL Server".to_owned();
        run_row_action(&mut app, ConnectionRowAction::CreateTable, &server);
        assert_eq!(app.active_query_connection_id(), Some("sqlserver-conn"));
        assert_eq!(app.query.session.active_text(), sql_server);

        let menu = include_str!("explorer_connection_row_view.rs");
        assert!(menu.contains("Open sample CREATE TABLE SQL…"));
        assert!(!menu.contains("Create Table (DDL)"));
    }

    #[test]
    fn catalog_states_stay_separate_and_refresh_targets_the_selected_node() {
        use super::super::explorer_surface_view::{catalog_surface, CatalogSurface};
        use crate::RequestId;

        assert_eq!(catalog_surface(true, None, true), CatalogSurface::Loading);
        assert_eq!(catalog_surface(false, Some("catalog down"), true), CatalogSurface::Failed);
        assert_eq!(catalog_surface(false, None, true), CatalogSurface::Empty);
        assert_eq!(catalog_surface(true, Some("catalog down"), false), CatalogSurface::Ready);

        let (bridge, command_rx, _event_tx) = crate::TaskBridge::with_channels();
        let mut app = DbProApp::with_task_bridge(bridge);
        let request_id = RequestId(19);
        app.connection.lifecycle.note_connections_list_started(request_id);
        assert_eq!(
            catalog_surface(
                app.connection.lifecycle.connections_request_pending(),
                app.connection.lifecycle.connections_error(),
                true
            ),
            CatalogSurface::Loading
        );

        app.on_query_failed(request_id, "catalog down".to_owned(), None, None);
        assert!(!app.connection.lifecycle.connections_request_pending());
        assert_eq!(app.connection.lifecycle.connections_error(), Some("catalog down"));
        assert!(app.feedback.runtime_message.contains("Could not load connections"));

        app.on_connections_loaded(Vec::new());
        assert!(app.connection.lifecycle.connections_error().is_none());
        assert!(!app.connection.lifecycle.connections_request_pending());

        app.connection
            .lifecycle
            .set_active_connection_id(Some("conn-1".to_owned()));
        app.refresh_explorer_node("conn-1", Some("public"), Some("orders"));
        assert!(matches!(
            command_rx.try_recv(),
            Ok(UiCommand::LoadTableInfo {
                connection_id,
                schema,
                table,
                ..
            }) if connection_id == "conn-1" && schema == "public" && table == "orders"
        ));

        app.refresh_explorer_node("conn-1", Some("analytics"), None);
        assert!(matches!(
            command_rx.try_recv(),
            Ok(UiCommand::IntrospectSchema {
                connection_id,
                force_refresh: true,
                ..
            }) if connection_id == "conn-1"
        ));
        assert_eq!(app.schema.explorer.selected_schema.as_deref(), Some("analytics"));

        let tree = include_str!("explorer_schema_tree_view.rs");
        assert!(tree.contains("schema: Some(schema.to_owned())"));
    }
}
