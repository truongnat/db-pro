//! Table rows and the nested detail folders (Columns / Foreign keys / Indexes)
//! of the Codex / DBeaver navigator tree.

use super::explorer_table_row_view::{TableRowAction, TableRowContext};
use super::explorer_tree::{
    column_icon_and_color, draw_category_folder, draw_codex_tree_row, shorten_data_type, CategoryFolder, CodexTreeRow,
};
use super::*;
use lucide_icons::Icon;

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
                self.draw_table_detail_folders(ui, table, &info);
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
        if self.schema.explorer.selected_table.as_deref() != Some(table)
            && !self.table.mutation.staged_changes.is_empty()
        {
            self.feedback.runtime_message = "Apply or discard staged changes before opening another table".to_owned();
            return;
        }
        let scope = TableDataState::layout_scope(
            self.connection.lifecycle.active_connection_id(),
            self.active_schema(),
            self.schema.explorer.selected_table.as_deref(),
        );
        self.table.data.persist_layout(scope);
        self.schema.explorer.selected_table = Some(table.to_owned());
        self.schema.explorer.record_recent_table(table);
        self.schema.explorer.selected_schema_object = None;
        self.schema.explorer.schema_object_view = SchemaObjectView::Definition;
        self.reset_table_workspace_state();
        let scope = TableDataState::layout_scope(
            self.connection.lifecycle.active_connection_id(),
            self.active_schema(),
            self.schema.explorer.selected_table.as_deref(),
        );
        self.table.data.restore_layout(scope);
        self.table.state.table_view = TableView::Data;
        let schema = self.active_schema();
        self.set_active_query_text(format!("SELECT *\nFROM {schema}.{table}\nLIMIT 100;"));
        self.request_table_info();
        self.request_table_data();
        self.workspace.active_tab = WorkspaceTab::Table;
    }

    /// Clears every table-workspace field. Shared by "select a table" and
    /// "connect to a connection" so both start from an identical slate.
    pub(super) fn reset_table_workspace_state(&mut self) {
        self.table.state.table_info = None;
        self.table.state.table_ddl = None;
        self.table.state.table_info_error = None;
        self.table.state.table_ddl_error = None;
        self.table.state.ddl_execute_confirmation = false;
        self.table.state.ddl_execution_request = None;
        self.table.data_query.reset_for_table();
        self.table.state.table_info_request = None;
        self.table.state.table_ddl_request = None;
        self.table.mutation.table_mutation_request = None;
        self.table.mutation.staged_changes.clear();
        self.table.mutation.staged_apply_request = None;
        self.table.mutation.staged_apply_targets.clear();
        self.table.mutation.table_mutation_retry_after_reload = false;
        self.table.mutation.table_mutation_retry_target = None;
        self.table.mutation.table_mutation_error = None;
        self.table.data.selected_cell = None;
        self.table.data.selected_row = None;
        self.table.data.selected_rows.clear();
        self.table.data.selection_anchor_row = None;
        self.table.data.selection_anchor_cell = None;
        self.table.editing.data_editing_cell = None;
        self.table.editing.data_edit_value.clear();
        self.table.editing.data_edit_error = None;
        self.table.editing.data_delete_confirmation = false;
        self.table.editing.discard_changes_confirmation = false;
        self.table.state.table_view = TableView::Data;
    }

    /// Nested detail folders shown under a selected, expanded table.
    fn draw_table_detail_folders(&mut self, ui: &mut egui::Ui, table: &str, info: &UiTableInfo) {
        self.draw_table_columns_folder(ui, table, info);
        self.draw_table_foreign_keys_folder(ui, table, info);
        self.draw_table_indexes_folder(ui, table, info);
    }

    /// "Columns" folder listing every column with its inferred icon and type.
    fn draw_table_columns_folder(&mut self, ui: &mut egui::Ui, table: &str, info: &UiTableInfo) {
        let theme = self.theme;
        let folder_id = ui.make_persistent_id(("codex_tbl_col_folder", table));
        draw_category_folder(
            ui,
            &theme,
            CategoryFolder {
                depth: 5,
                id: folder_id,
                icon: Icon::Columns3,
                icon_color: theme.text_secondary,
                label: "Columns",
                count: info.columns.len(),
                empty_label: None,
            },
            |ui| {
                for column in &info.columns {
                    let is_fk = info
                        .foreign_keys
                        .iter()
                        .any(|fk| fk.from_columns.contains(&column.name));
                    let (icon, icon_color) =
                        column_icon_and_color(&column.data_type, column.is_primary_key, is_fk, &theme);
                    let short_type = shorten_data_type(&column.data_type);
                    draw_codex_tree_row(
                        ui,
                        &theme,
                        CodexTreeRow {
                            depth: 6,
                            is_expandable: false,
                            is_expanded: false,
                            icon,
                            icon_color,
                            label: &column.name,
                            is_selected: false,
                            is_dimmed: false,
                            status_dot: None,
                            badge_text: None,
                            badge_accent: false,
                            count_text: None,
                            detail_text: Some(&short_type),
                        },
                    );
                }
            },
        );
    }

    /// "Foreign keys" folder listing outgoing references.
    fn draw_table_foreign_keys_folder(&mut self, ui: &mut egui::Ui, table: &str, info: &UiTableInfo) {
        let theme = self.theme;
        let folder_id = ui.make_persistent_id(("codex_tbl_fk_folder", table));
        draw_category_folder(
            ui,
            &theme,
            CategoryFolder {
                depth: 5,
                id: folder_id,
                icon: Icon::ArrowRightLeft,
                icon_color: theme.text_secondary,
                label: "Foreign keys",
                count: info.foreign_keys.len(),
                empty_label: Some("No foreign keys"),
            },
            |ui| {
                for fk in &info.foreign_keys {
                    draw_codex_tree_row(
                        ui,
                        &theme,
                        CodexTreeRow {
                            depth: 6,
                            is_expandable: false,
                            is_expanded: false,
                            icon: Icon::Link,
                            icon_color: theme.info,
                            label: &fk.name,
                            is_selected: false,
                            is_dimmed: false,
                            status_dot: None,
                            badge_text: None,
                            badge_accent: false,
                            count_text: None,
                            detail_text: Some(&fk.to_table),
                        },
                    );
                }
            },
        );
    }

    /// "Indexes" folder listing declared indexes.
    fn draw_table_indexes_folder(&mut self, ui: &mut egui::Ui, table: &str, info: &UiTableInfo) {
        let theme = self.theme;
        let folder_id = ui.make_persistent_id(("codex_tbl_idx_folder", table));
        draw_category_folder(
            ui,
            &theme,
            CategoryFolder {
                depth: 5,
                id: folder_id,
                icon: Icon::List,
                icon_color: theme.text_secondary,
                label: "Indexes",
                count: info.indexes.len(),
                empty_label: Some("No indexes"),
            },
            |ui| {
                for index in &info.indexes {
                    draw_codex_tree_row(
                        ui,
                        &theme,
                        CodexTreeRow {
                            depth: 6,
                            is_expandable: false,
                            is_expanded: false,
                            icon: Icon::Zap,
                            icon_color: theme.text_muted,
                            label: &index.name,
                            is_selected: false,
                            is_dimmed: false,
                            status_dot: None,
                            badge_text: None,
                            badge_accent: false,
                            count_text: None,
                            detail_text: None,
                        },
                    );
                }
            },
        );
    }
}
