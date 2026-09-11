//! Table rows and the nested detail folders (Columns / Foreign keys / Indexes)
//! of the Codex / DBeaver navigator tree.

use super::explorer_tree::{
    column_icon_and_color, draw_category_folder, draw_codex_tree_row, CategoryFolder, CodexTreeRow,
};
use super::*;
use egui::Color32;
use lucide_icons::Icon;

/// Actions selectable from a table row's context menu.
#[derive(Default)]
struct TableRowActions {
    open_query: bool,
    ask_agent: bool,
    refresh_schema: bool,
}

/// Collects the table row context-menu choices without touching `self`, so the
/// caller keeps a single mutable borrow for applying them.
fn table_row_context_menu(response: &egui::Response) -> TableRowActions {
    let mut actions = TableRowActions::default();
    response.context_menu(|ui| {
        if ui.button("Open in Query").clicked() {
            actions.open_query = true;
            ui.close_menu();
        }
        if ui.button("Ask Agent about table").clicked() {
            actions.ask_agent = true;
            ui.close_menu();
        }
        if ui.button("Refresh Schema").clicked() {
            actions.refresh_schema = true;
            ui.close_menu();
        }
    });
    actions
}

impl DbProApp {
    /// Renders an individual table item in the tree with selection and expandable details.
    pub(super) fn draw_dbeaver_table_item(&mut self, ui: &mut egui::Ui, table: &str) {
        let is_selected = self.selected_table.as_deref() == Some(table);
        let table_details_id = ui.make_persistent_id(("codex_tbl_details", table));
        let has_details = is_selected && self.table_info.is_some();

        let mut collapsing =
            egui::collapsing_header::CollapsingState::load_with_default_open(ui.ctx(), table_details_id, true);
        let is_open = collapsing.is_open();

        let (response, chevron_clicked) = draw_codex_tree_row(
            ui,
            &self.theme,
            CodexTreeRow {
                depth: 4,
                is_expandable: has_details,
                is_expanded: is_open,
                icon: Icon::Table2,
                icon_color: if is_selected {
                    self.theme.accent
                } else {
                    self.theme.text_secondary
                },
                label: table,
                is_selected,
                is_dimmed: false,
                status_dot: None,
                badge_text: None,
                badge_accent: false,
                count_text: None,
                detail_text: None,
            },
        );

        let actions = table_row_context_menu(&response);

        if chevron_clicked && has_details {
            collapsing.set_open(!is_open);
            collapsing.store(ui.ctx());
        } else if response.clicked() || actions.open_query || actions.ask_agent {
            self.select_table(table);
        }

        if actions.open_query {
            self.query_text = format!("SELECT *\nFROM {table}\nLIMIT 100;");
            self.active_tab = WorkspaceTab::Query;
        }
        if actions.ask_agent {
            self.open_agent_prompt(
                format!("Explain the `{table}` table and suggest useful read-only queries"),
                ui.ctx(),
            );
        }
        if actions.refresh_schema {
            if let Some(connection_id) = self.active_connection_id.clone() {
                self.request_schema_introspection(connection_id, true);
            }
        }

        // If table is selected and expanded, show nested details (Columns, Foreign keys, Indexes)
        if is_selected && collapsing.is_open() {
            if let Some(info) = self.table_info.clone() {
                self.draw_table_detail_folders(ui, table, &info);
            }
        }
    }

    /// Selects a table and resets the table workspace to a clean slate.
    fn select_table(&mut self, table: &str) {
        self.selected_table = Some(table.to_owned());
        self.selected_schema_object = None;
        self.schema_object_view = SchemaObjectView::Definition;
        self.reset_table_workspace_state();
        self.query_text = format!("SELECT *\nFROM {table}\nLIMIT 100;");
        self.request_table_info();
        self.active_tab = WorkspaceTab::Table;
    }

    /// Clears every table-workspace field. Shared by "select a table" and
    /// "connect to a connection" so both start from an identical slate.
    pub(super) fn reset_table_workspace_state(&mut self) {
        self.table_info = None;
        self.table_ddl = None;
        self.table_info_error = None;
        self.table_ddl_error = None;
        self.ddl_execute_confirmation = false;
        self.ddl_execution_request = None;
        self.table_data_result = None;
        self.table_data_total_rows = None;
        self.table_data_offset = 0;
        self.table_data_filter_column.clear();
        self.table_data_filter_value.clear();
        self.table_data_sort_column = None;
        self.table_data_sort_desc = false;
        self.table_data_error = None;
        self.table_info_request = None;
        self.table_ddl_request = None;
        self.table_data_request = None;
        self.table_mutation_request = None;
        self.staged_changes.clear();
        self.staged_apply_request = None;
        self.selected_cell = None;
        self.selected_row = None;
        self.data_editing_cell = None;
        self.data_edit_value.clear();
        self.data_delete_confirmation = false;
        self.table_view = TableView::Structure;
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
                            detail_text: Some(&column.data_type),
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
                            icon_color: Color32::from_rgb(37, 99, 235),
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
