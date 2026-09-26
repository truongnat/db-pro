//! Presentation and intent mapping for schema-scoped objects in the Explorer.
//!
//! The view owns tree layout, filtering and row interaction. The application
//! root remains responsible for turning the returned intents into workspace,
//! query and runtime changes.

use super::explorer_schema_object_folders_view::SchemaObjectFolderAction;
use super::explorer_schema_object_folders_view::SchemaObjectFoldersView;
use super::explorer_table_details_view::TableDetailsView;
use super::explorer_table_folder_view::TableFolderContext;
use super::explorer_table_row_view::{TableRowAction, TableRowContext};
use super::{DbProTheme, SchemaExplorerState, UiTableInfo, EXPLORER_ROW_HEIGHT};
use eframe::egui;

pub(super) enum ExplorerSchemaObjectsAction {
    SelectTable(String),
    TableRow { table: String, action: TableRowAction },
    SchemaObject(SchemaObjectFolderAction),
}

pub(super) struct ExplorerSchemaObjectsModel {
    pub(super) selected_table: Option<String>,
    pub(super) table_info: Option<UiTableInfo>,
    pub(super) connection_id: String,
    pub(super) functions_enabled: bool,
}

pub(super) struct ExplorerSchemaObjectsView<'a> {
    theme: DbProTheme,
    explorer: &'a mut SchemaExplorerState,
    model: ExplorerSchemaObjectsModel,
}

impl<'a> ExplorerSchemaObjectsView<'a> {
    pub(super) fn new(
        theme: DbProTheme,
        explorer: &'a mut SchemaExplorerState,
        model: ExplorerSchemaObjectsModel,
    ) -> Self {
        Self { theme, explorer, model }
    }

    pub(super) fn draw(&mut self, ui: &mut egui::Ui, schema: &str) -> Vec<ExplorerSchemaObjectsAction> {
        let search_query = self.explorer.explorer_search.trim().to_ascii_lowercase();
        let mut actions = self.draw_tables(ui, schema, &search_query);

        let view_count = self
            .explorer
            .count_by_schema(&self.explorer.schema.views, schema, |view| &view.schema);
        actions.extend(
            SchemaObjectFoldersView::new(self.theme, &*self.explorer)
                .draw_views(ui, schema, &search_query, view_count)
                .into_iter()
                .map(ExplorerSchemaObjectsAction::SchemaObject),
        );

        if self.model.functions_enabled {
            let function_count = self
                .explorer
                .count_by_schema(&self.explorer.schema.functions, schema, |function| &function.schema);
            actions.extend(
                SchemaObjectFoldersView::new(self.theme, &*self.explorer)
                    .draw_functions(ui, schema, &search_query, function_count)
                    .into_iter()
                    .map(ExplorerSchemaObjectsAction::SchemaObject),
            );
        }

        let trigger_count = self
            .explorer
            .count_by_schema(&self.explorer.schema.triggers, schema, |trigger| &trigger.schema);
        actions.extend(
            SchemaObjectFoldersView::new(self.theme, &*self.explorer)
                .draw_triggers(ui, schema, &search_query, trigger_count)
                .into_iter()
                .map(ExplorerSchemaObjectsAction::SchemaObject),
        );
        actions
    }

    fn draw_tables(&mut self, ui: &mut egui::Ui, schema: &str, search_query: &str) -> Vec<ExplorerSchemaObjectsAction> {
        let total_tables = super::connection_status::schema_table_count(self.explorer, schema);
        let matching_table_count = self.matching_table_count(schema, search_query, total_tables);
        let folder = TableFolderContext {
            theme: self.theme,
            schema,
            total_tables,
            matching_tables: matching_table_count,
            search_query,
        };
        let render = folder.draw_header(ui);
        if !render.is_open {
            return Vec::new();
        }

        let (_total, _matching, tables) = self
            .explorer
            .cached_tables(&self.model.connection_id, schema, search_query);
        if tables.is_empty() {
            folder.draw_empty_state(ui);
            return Vec::new();
        }

        let mut actions = Vec::new();
        let clip = ui.clip_rect();
        for table in &tables {
            let row_top = ui.cursor().min.y;
            let row_bottom = row_top + EXPLORER_ROW_HEIGHT;
            if row_bottom < clip.top() || row_top > clip.bottom() {
                folder.draw_offscreen_row_spacer(ui);
                continue;
            }
            actions.extend(self.draw_table_row(ui, table));
        }
        folder.draw_overflow_hint(ui, tables.len());
        actions
    }

    fn draw_table_row(&self, ui: &mut egui::Ui, table: &str) -> Vec<ExplorerSchemaObjectsAction> {
        let is_selected = self.model.selected_table.as_deref() == Some(table);
        let has_details = is_selected && self.model.table_info.is_some();
        let render = TableRowContext {
            theme: self.theme,
            table,
            is_selected,
            has_details,
        }
        .draw(ui);

        let mut actions = Vec::new();
        if render.should_select {
            actions.push(ExplorerSchemaObjectsAction::SelectTable(table.to_owned()));
        }
        actions.extend(
            render
                .actions
                .into_iter()
                .map(|action| ExplorerSchemaObjectsAction::TableRow {
                    table: table.to_owned(),
                    action,
                }),
        );

        if is_selected && render.is_open {
            if let Some(info) = self.model.table_info.as_ref() {
                TableDetailsView::new(&self.theme).draw(ui, table, info);
            }
        }
        actions
    }

    fn matching_table_count(&self, schema: &str, search_query: &str, total_tables: usize) -> usize {
        if search_query.is_empty() {
            return total_tables;
        }
        self.explorer
            .explorer_nav_cache
            .as_ref()
            .filter(|cache| {
                cache.schema == schema
                    && cache.search == search_query
                    && cache.connection_id == self.model.connection_id
            })
            .map(|cache| cache.matching_count)
            .unwrap_or_else(|| self.explorer.matching_table_count(schema, search_query))
    }
}
