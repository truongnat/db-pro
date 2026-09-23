use super::palette_search_view::PaletteSearchContext;
use super::*;

impl DbProApp {
    fn palette_entries(&self, mode: PaletteMode) -> Vec<(SearchKind, PaletteItem)> {
        let active_schema = self.active_schema().to_owned();
        let table_names = self.active_schema_table_names();
        let column_names = self.active_schema_column_names();
        PaletteSearchContext {
            workspace: &self.workspace,
            schema: &self.schema,
            query: &self.query,
            connection: &self.connection,
            active_schema: &active_schema,
            table_names: &table_names,
            column_names: &column_names,
        }
        .entries(mode)
    }

    fn search_fingerprint(&self) -> String {
        let workspace_files = self
            .workspace
            .files
            .ide_workspace
            .index()
            .into_iter()
            .filter(|entry| entry.is_sql)
            .count();
        SearchService::build_fingerprint(SearchFingerprintParts {
            connection_id: self.connection.lifecycle.active_connection_id(),
            schema: self.active_schema(),
            tables: self.schema.explorer.schema.tables.len(),
            views: self.schema.explorer.schema.views.len(),
            functions: self.schema.explorer.schema.functions.len(),
            columns: self.active_schema_column_names().len(),
            saved_queries: self.query.library.saved_queries.len(),
            history: self.query.editor.query_history_entries.len(),
            connections: self.connection.catalog.len(),
            workspace_files,
        })
    }

    fn ensure_search_index(&mut self, mode: PaletteMode) {
        let fingerprint = format!("{}|{:?}", self.search_fingerprint(), mode);
        if self.palette.search_index.fingerprint() == fingerprint && !self.palette.search_index.is_empty() {
            return;
        }
        let entries = self.palette_entries(mode);
        self.palette.search_index.replace(fingerprint, entries);
    }

    pub(crate) fn filtered_palette_items(&self, mode: PaletteMode) -> Vec<PaletteItem> {
        let entries = if self.palette.search_index.fingerprint().contains(&format!("{mode:?}"))
            && !self.palette.search_index.is_empty()
        {
            self.palette.search_index.entries().to_vec()
        } else {
            self.palette_entries(mode)
        };
        SearchService::filter_rank(&entries, &self.palette.query, self.palette.scope, 120)
    }

    /// Rebuild + rank for mutable callers (palette draw path).
    pub(crate) fn filtered_palette_items_fresh(&mut self, mode: PaletteMode) -> Vec<PaletteItem> {
        self.ensure_search_index(mode);
        self.filtered_palette_items(mode)
    }

    pub(super) fn refresh_schema_palette(&mut self) {
        if let Some(connection_id) = self.connection.lifecycle.active_connection_id().map(str::to_owned) {
            self.table.state.refresh_table_info_after_schema = self.schema.explorer.selected_table.is_some();
            self.request_schema_introspection(connection_id, true);
        } else {
            self.feedback.runtime_message = "Connect to a database before refreshing schema".to_owned();
        }
    }

    pub(crate) fn open_table_from_palette(&mut self, table: String) {
        if self.schema.explorer.selected_table.as_deref() != Some(table.as_str())
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
        self.schema.explorer.record_recent_table(&table);
        self.schema.explorer.selected_table = Some(table.clone());
        let scope = TableDataState::layout_scope(
            self.connection.lifecycle.active_connection_id(),
            self.active_schema(),
            self.schema.explorer.selected_table.as_deref(),
        );
        self.table.data.restore_layout(scope);
        self.schema.explorer.selected_schema_object = None;
        self.table.state.table_view = TableView::Structure;
        self.table.state.table_info = None;
        self.table.state.table_ddl = None;
        self.table.data_query.result = None;
        self.request_table_info();
        self.workspace.active_tab = WorkspaceTab::Table;
        self.feedback.runtime_message = format!("Opening table {table}");
    }

    pub(super) fn open_saved_query_from_palette(&mut self, query_id: String) {
        let Some(query) = self
            .query
            .library
            .saved_queries
            .iter()
            .find(|item| item.id == query_id)
            .cloned()
        else {
            self.feedback.runtime_message = "Saved query is no longer available".to_owned();
            return;
        };
        self.new_query_document();
        if let Some(doc) = self
            .query
            .session
            .documents
            .get_mut(self.query.session.active_document_index)
        {
            doc.set_text(query.sql.clone());
            doc.title = query.name.clone();
            doc.saved_query_id = Some(query.id.clone());
            doc.mark_saved();
        }
        self.workspace.active_tab = WorkspaceTab::Query;
        self.feedback.runtime_message = format!("Opened saved query {}", query.name);
    }

    pub(crate) fn toggle_pinned_table(&mut self, table: String) {
        let target = if table.is_empty() {
            self.schema.explorer.selected_table.clone()
        } else {
            Some(table)
        };
        let Some(table) = target else {
            self.feedback.runtime_message = "Select a table before pinning".to_owned();
            return;
        };
        if let Some(index) = self
            .schema
            .explorer
            .pinned_tables
            .iter()
            .position(|item| item == &table)
        {
            self.schema.explorer.pinned_tables.remove(index);
            self.feedback.runtime_message = format!("Unpinned table {table}");
        } else {
            self.schema.explorer.pinned_tables.push(table.clone());
            self.feedback.runtime_message = format!("Pinned table {table}");
        }
    }

    pub(super) fn export_results_from_palette(&mut self) {
        if self.query.session.active_result().is_some() {
            self.query.output.active_tab = OutputTab::Results;
            self.overlay.export_open = true;
            self.workspace.active_tab = WorkspaceTab::Query;
        } else {
            self.feedback.runtime_message = "Run a query before exporting results".to_owned();
        }
    }

    pub(super) fn switch_connection_from_palette(&mut self, connection_id: String) {
        let connection = self.connection.catalog.find(&connection_id).cloned();
        if let Some(connection) = connection {
            self.connect_to_connection(&connection);
        }
    }

    pub(super) fn draw_palette(&mut self, ctx: &egui::Context) {
        let Some(mode) = self.palette.mode else {
            return;
        };
        let items = self.filtered_palette_items_fresh(mode);
        self.clamp_palette_selection(&items);
        let title = if mode == PaletteMode::QuickOpen {
            "Quick Open"
        } else if self.palette.scope == SearchScope::Connections {
            "Switch Connection"
        } else {
            "Command Palette"
        };
        let description = if mode == PaletteMode::QuickOpen {
            "Switch workspaces, tabs, or open editors"
        } else if self.palette.scope == SearchScope::Connections {
            "Pick a saved connection to open"
        } else {
            "Search commands, actions, and database tools"
        };
        let actions = {
            let mut context = palette_surface_view::PaletteSurfaceContext {
                theme: self.theme,
                palette: &mut self.palette,
                items: &items,
                title,
                description,
            };
            context.draw(ctx)
        };
        for action in actions {
            match action {
                palette_surface_view::PaletteSurfaceAction::Close => self.palette.mode = None,
                palette_surface_view::PaletteSurfaceAction::Activate(index) => {
                    if let Some(item) = items.get(index) {
                        self.execute_palette_action(item.action.clone(), ctx);
                    }
                }
            }
        }
    }

    fn clamp_palette_selection(&mut self, items: &[PaletteItem]) {
        if items.is_empty() {
            self.palette.selected = 0;
        } else {
            self.palette.selected = self.palette.selected.min(items.len() - 1);
        }
    }
}
