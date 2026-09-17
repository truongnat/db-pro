//! Active connection, schema names, and statusbar helpers.
use super::*;
use egui::Color32;
use lucide_icons::Icon;

impl DbProApp {
    pub(super) fn active_connection(&self) -> Option<&UiConnectionSummary> {
        self.connections
            .iter()
            .find(|connection| Some(connection.id.as_str()) == self.active_connection_id.as_deref())
    }

    pub(super) fn active_connection_name(&self) -> &str {
        self.active_connection()
            .map(|connection| connection.name.as_str())
            .unwrap_or(self.connection_name.as_str())
    }

    pub(super) fn active_driver(&self) -> &str {
        self.active_connection()
            .map(|connection| connection.driver.as_str())
            .unwrap_or("PostgreSQL")
    }

    pub(crate) fn active_capabilities(&self) -> CapabilityLookup {
        match self.active_connection() {
            Some(connection) => CapabilityLookup::for_driver_label(&connection.driver),
            None => CapabilityLookup::NoActiveConnection,
        }
    }

    pub(super) fn active_schema(&self) -> &str {
        self.selected_schema
            .as_deref()
            .or_else(|| self.schema.schemas.first().map(String::as_str))
            .unwrap_or_else(|| {
                if self.active_driver().eq_ignore_ascii_case("sqlite") {
                    "main"
                } else {
                    "public"
                }
            })
    }

    pub(super) fn active_schema_table_names(&self) -> Vec<String> {
        self.schema_table_names(self.active_schema())
    }

    /// Table names belonging to `schema`. Empty `schema` (SQLite flat tree) returns all tables.
    pub(super) fn schema_table_names(&self, schema: &str) -> Vec<String> {
        if self.schema.schemas.is_empty() || self.schema.table_details.is_empty() {
            return self.schema.tables.clone();
        }
        if schema.is_empty() {
            return self
                .schema
                .table_details
                .iter()
                .map(|table| table.name.clone())
                .collect();
        }
        self.schema
            .table_details
            .iter()
            .filter(|table| table.schema == schema)
            .map(|table| table.name.clone())
            .collect()
    }

    pub(super) fn schema_table_count(&self, schema: &str) -> usize {
        if self.schema.schemas.is_empty() || self.schema.table_details.is_empty() {
            return self.schema.tables.len();
        }
        if schema.is_empty() {
            return self.schema.table_details.len();
        }
        self.schema
            .table_details
            .iter()
            .filter(|table| table.schema == schema)
            .count()
    }

    /// Count matching table names without allocating a name list.
    pub(super) fn schema_matching_table_count(&self, schema: &str, query: &str) -> usize {
        if query.is_empty() {
            return self.schema_table_count(schema);
        }
        if self.schema.schemas.is_empty() || self.schema.table_details.is_empty() {
            return self
                .schema
                .tables
                .iter()
                .filter(|table| matches_explorer_table(table, query))
                .count();
        }
        self.schema
            .table_details
            .iter()
            .filter(|table| schema.is_empty() || table.schema == schema)
            .filter(|table| matches_explorer_table(&table.name, query))
            .count()
    }

    pub(super) fn active_schema_column_names(&self) -> Vec<String> {
        if self.schema.schemas.is_empty() || self.schema.table_details.is_empty() {
            return self.schema.columns.clone();
        }
        self.schema
            .table_details
            .iter()
            .filter(|table| table.schema == self.active_schema())
            .flat_map(|table| table.columns.iter().map(|column| column.name.clone()))
            .collect()
    }

    pub(super) fn has_runtime_error(&self) -> bool {
        self.runtime_message.contains("failed")
            || self.runtime_message.contains("Failed")
            || self.runtime_message.contains("error")
            || self.runtime_message.contains("Error")
    }

    /// The status-bar message and the colour it is rendered in.
    ///
    /// Every `runtime_message` is user-facing: a refusal such as "Connect with write
    /// access to delete rows" is the *only* feedback a blocked action produces, so the
    /// bar shows any non-empty message and reserves the danger colour for errors.
    /// Restricting the bar to strings containing "failed"/"error" hid every refusal,
    /// gate and informational message the app sets.
    pub(super) fn runtime_status(&self) -> Option<(String, Color32)> {
        if self.runtime_message.trim().is_empty() {
            None
        } else if self.has_runtime_error() {
            Some((self.runtime_message.clone(), self.theme.danger))
        } else {
            Some((self.runtime_message.clone(), self.theme.text_secondary))
        }
    }

    pub(super) fn statusbar_state(&self) -> (Icon, Color32, &'static str) {
        if self.connected && self.active_connection_id.is_some() {
            return (Icon::CircleCheck, self.theme.success, "Connected");
        }
        if self.runtime_message.starts_with("Connecting") {
            return (Icon::Circle, self.theme.accent, "Connecting…");
        }
        if self.has_runtime_error() {
            return (Icon::TriangleAlert, self.theme.danger, "Runtime error");
        }
        (Icon::Circle, self.theme.warning, "Not connected")
    }

    pub(super) fn shows_editor_status(&self) -> bool {
        self.active_tab == WorkspaceTab::Query
    }

    pub(super) fn statusbar_context_label(&self) -> &'static str {
        match self.active_tab {
            WorkspaceTab::Welcome => "Workspace",
            WorkspaceTab::Query => "SQL Editor",
            WorkspaceTab::Table => match self.table_view {
                TableView::Structure => "Table Structure",
                TableView::Data => "Data Editor",
                TableView::Profile => "Column Profile",
                TableView::Indexes => "Table Indexes",
                TableView::Relations => "Table Relations",
                TableView::Constraints => "Table Constraints",
                TableView::Dependencies => "Table Dependencies",
                TableView::Ddl => "Table DDL",
            },
            WorkspaceTab::SchemaObject => "Schema Object",
            WorkspaceTab::Diagram => "ER Diagram",
            WorkspaceTab::SchemaWorkbench => "Schema Workbench",
            WorkspaceTab::SchemaCompare => "Schema Compare",
            WorkspaceTab::ComponentGallery => "Component Gallery",
        }
    }

    pub(super) fn connection_indicator(&self, connection: &UiConnectionSummary) -> (Icon, Color32) {
        let is_active = self.active_connection_id.as_deref() == Some(connection.id.as_str());
        let is_connected = is_active && self.connected;
        let is_failed = self.failed_connection_ids.contains(&connection.id);
        let icon = if is_connected {
            Icon::CircleCheck
        } else if is_failed {
            Icon::AlertCircle
        } else {
            Icon::Circle
        };
        let color = if is_connected && connection.readonly {
            self.theme.warning
        } else if is_connected {
            self.theme.success
        } else if is_failed {
            self.theme.danger
        } else if is_active {
            self.theme.accent
        } else {
            self.theme.text_muted
        };
        (icon, color)
    }
}
