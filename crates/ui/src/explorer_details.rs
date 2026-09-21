//! Table rows and the nested detail folders (Columns / Foreign keys / Indexes)
//! of the Codex / DBeaver navigator tree.

use super::explorer_table_row_view::{TableRowAction, TableRowContext};
use super::*;

struct TableRowActionInput<'a> {
    table: &'a str,
    schema: &'a str,
}

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
        .filter(|set_clause| !set_clause.is_empty())
        .unwrap_or_else(|| "    column1 = 'value1'".to_owned());
    format!(
        "UPDATE {schema}.{table}\nSET\n{set_clause}\nWHERE {};",
        primary_key_clause(info)
    )
}

fn build_delete_query(schema: &str, table: &str, info: Option<&UiTableInfo>) -> String {
    format!("DELETE FROM {schema}.{table}\nWHERE {};", primary_key_clause(info))
}

fn primary_key_clause(info: Option<&UiTableInfo>) -> String {
    info.and_then(|table_info| table_info.primary_key.as_ref())
        .map(|columns| {
            columns
                .iter()
                .map(|column| format!("{column} = 1"))
                .collect::<Vec<_>>()
                .join(" AND ")
        })
        .filter(|clause| !clause.is_empty())
        .unwrap_or_else(|| "id = 1".to_owned())
}

impl DbProApp {
    /// Renders an individual table item in the tree with selection and expandable details.
    pub(super) fn draw_dbeaver_table_item(&mut self, ui: &mut egui::Ui, table: &str) {
        let is_selected = self.schema.explorer.selected_table.as_deref() == Some(table);
        let has_details = is_selected && self.table.state.table_info.is_some();
        let render = TableRowContext {
            theme: self.theme,
            table,
            is_selected,
            has_details,
        }
        .draw(ui);

        if render.should_select {
            self.select_table(table);
        }

        let schema = self.active_schema().to_owned();
        for action in render.actions {
            self.apply_table_row_action(action, TableRowActionInput { table, schema: &schema }, ui);
        }

        // If table is selected and expanded, show nested details (Columns, Foreign keys, Indexes)
        if is_selected && render.is_open {
            if let Some(info) = self.table.state.table_info.clone() {
                explorer_table_details_view::TableDetailsView::new(&self.theme).draw(ui, table, &info);
            }
        }
    }

    fn apply_table_row_action(&mut self, action: TableRowAction, input: TableRowActionInput<'_>, ui: &mut egui::Ui) {
        let table = input.table;
        let schema = input.schema;
        match action {
            TableRowAction::OpenData => self.open_table_view(TableView::Data),
            TableRowAction::OpenStructure => self.open_table_view(TableView::Structure),
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

    /// Selects a table and resets the table workspace to a clean slate.
    pub(crate) fn select_table(&mut self, table: &str) {
        let connection_id = self.connection.lifecycle.active_connection_id().map(str::to_owned);
        let schema = self.active_schema().to_owned();
        let selected = explorer_navigation::TableSelectionContext::new(
            &mut self.schema.explorer,
            &mut self.table,
            &mut self.workspace,
            &mut self.feedback,
        )
        .select(table, connection_id.as_deref(), &schema);
        if !selected {
            return;
        }
        self.set_active_query_text(format!("SELECT *\nFROM {schema}.{table}\nLIMIT 100;"));
        self.request_table_info();
        self.request_table_data();
    }
}
