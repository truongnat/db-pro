//! Table rows and the nested detail folders (Columns / Foreign keys / Indexes)
//! of the Codex / DBeaver navigator tree.

use super::explorer_table_row_view::TableRowAction;
use super::schema_workbench::SchemaWorkbenchMode;
use super::*;
use db_pro_core::domain::object_mutation::ObjectAction;

fn build_insert_query(schema: &str, table: &str, info: Option<&UiTableInfo>) -> String {
    let columns = info
        .map(|table_info| {
            table_info
                .columns
                .iter()
                .map(|column| column.name.as_str())
                .collect::<Vec<_>>()
                .join(", ")
        })
        .filter(|columns| !columns.is_empty())
        .unwrap_or_else(|| "column1, column2".to_owned());
    let values = info
        .map(|table_info| {
            table_info
                .columns
                .iter()
                .map(|_| "DEFAULT")
                .collect::<Vec<_>>()
                .join(", ")
        })
        .filter(|values| !values.is_empty())
        .unwrap_or_else(|| "'value1', 'value2'".to_owned());
    format!("INSERT INTO {schema}.{table} ({columns})\nVALUES ({values});")
}

fn build_update_query(schema: &str, table: &str, info: Option<&UiTableInfo>) -> String {
    let set_clause = info
        .map(|table_info| {
            table_info
                .columns
                .iter()
                .filter(|column| !column.is_primary_key)
                .map(|column| format!("    {} = DEFAULT", column.name))
                .collect::<Vec<_>>()
                .join(",\n")
        })
        .filter(|clause| !clause.is_empty())
        .unwrap_or_else(|| "    column_name = DEFAULT".to_owned());
    let where_clause = info
        .map(|table_info| {
            table_info
                .columns
                .iter()
                .filter(|column| column.is_primary_key)
                .map(|column| format!("{} = 1", column.name))
                .collect::<Vec<_>>()
                .join(" AND ")
        })
        .filter(|clause| !clause.is_empty())
        .unwrap_or_else(|| "id = 1".to_owned());
    format!("UPDATE {schema}.{table}\nSET\n{set_clause}\nWHERE {where_clause};")
}

fn build_delete_query(schema: &str, table: &str, info: Option<&UiTableInfo>) -> String {
    let where_clause = info
        .map(|table_info| {
            table_info
                .columns
                .iter()
                .filter(|column| column.is_primary_key)
                .map(|column| format!("{} = 1", column.name))
                .collect::<Vec<_>>()
                .join(" AND ")
        })
        .filter(|clause| !clause.is_empty())
        .unwrap_or_else(|| "id = 1".to_owned());
    format!("DELETE FROM {schema}.{table}\nWHERE {where_clause};")
}

impl DbProApp {
    pub(super) fn apply_table_row_action(
        &mut self,
        action: TableRowAction,
        table: &str,
        schema: &str,
        ui: &mut egui::Ui,
    ) {
        match action {
            TableRowAction::OpenData => self.open_table_view(TableView::Data),
            TableRowAction::OpenStructure => self.open_table_view(TableView::Structure),
            TableRowAction::OpenModifyTable => {
                self.schema.workbench.schema = schema.to_owned();
                self.schema.workbench.name = table.to_owned();
                self.schema.workbench.parent_table = table.to_owned();
                self.schema.workbench.mode = SchemaWorkbenchMode::Table;
                if let Some(details) = self
                    .schema
                    .explorer
                    .schema
                    .table_details
                    .iter()
                    .find(|t| t.schema == schema && t.name == table)
                    .cloned()
                {
                    self.schema.workbench.load_from_table_details(&details);
                }
                self.workspace.activity = Activity::Schema;
                self.workspace.active_tab = WorkspaceTab::SchemaWorkbench;
                self.workspace.sidebar_open = true;
            }
            TableRowAction::DropTable => {
                self.schema.workbench.schema = schema.to_owned();
                self.schema.workbench.name = table.to_owned();
                self.schema.workbench.parent_table = table.to_owned();
                self.schema.workbench.mode = SchemaWorkbenchMode::Table;
                self.plan_workbench_action(ObjectAction::Drop);
                self.schema.workbench.apply_confirmation = true;
                self.workspace.activity = Activity::Schema;
                self.workspace.active_tab = WorkspaceTab::SchemaWorkbench;
                self.workspace.sidebar_open = true;
            }
            TableRowAction::OpenDdl => self.open_table_view(TableView::Ddl),
            TableRowAction::OpenQuery => {
                self.open_query_document(format!("SELECT *\nFROM {schema}.{table}\nLIMIT 100;"))
            }
            TableRowAction::GenerateInsert => {
                let query = build_insert_query(schema, table, self.table.state.table_info.as_ref());
                self.open_query_document(query);
            }
            TableRowAction::GenerateUpdate => {
                let query = build_update_query(schema, table, self.table.state.table_info.as_ref());
                self.open_query_document(query);
            }
            TableRowAction::GenerateDelete => {
                let query = build_delete_query(schema, table, self.table.state.table_info.as_ref());
                self.open_query_document(query);
            }
            TableRowAction::CopyQualifiedName => {
                let qualified_name = format!("{schema}.{table}");
                ui.output_mut(|output| output.copied_text = qualified_name.clone());
                self.feedback.runtime_message = format!("Copied `{qualified_name}` to clipboard");
            }
            TableRowAction::CopyName => {
                ui.output_mut(|output| output.copied_text = table.to_owned());
                self.feedback.runtime_message = format!("Copied `{table}` to clipboard");
            }
            TableRowAction::AskAgent => self.open_agent_prompt(
                format!("Explain the `{schema}.{table}` table structure and suggest useful queries"),
                ui.ctx(),
            ),
            TableRowAction::RefreshSchema => {
                if let Some(connection_id) = self.connection.lifecycle.active_connection_id().map(str::to_owned) {
                    self.request_schema_introspection(connection_id, true);
                }
            }
        }
    }

    fn open_table_view(&mut self, view: TableView) {
        self.table.state.table_view = view;
        self.workspace.active_tab = WorkspaceTab::Table;
    }

    fn open_query_document(&mut self, query: String) {
        self.set_active_query_text(query);
        self.workspace.active_tab = WorkspaceTab::Query;
    }
}
