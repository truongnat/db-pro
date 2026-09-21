use super::*;

/// Owns database schema navigation, selection and explorer read-model state.
#[derive(Debug)]
pub(crate) struct SchemaExplorerState {
    pub(super) schema: UiSchemaSummary,
    pub(super) schema_symbol_index: SchemaSymbolIndex,
    pub(super) selected_schema: Option<String>,
    pub(super) explorer_search: String,
    pub(super) explorer_nav_cache: Option<ExplorerNavCache>,
    pub(super) schema_error: Option<String>,
    pub(super) schema_request: Option<crate::RequestId>,
    pub(super) selected_table: Option<String>,
    pub(super) pinned_tables: Vec<String>,
    pub(super) recent_tables: Vec<String>,
    pub(super) selected_schema_object: Option<SchemaObjectSelection>,
    pub(super) schema_object_view: SchemaObjectView,
    pub(super) connections_pane_height: f32,
    pub(super) schemas_pane_height: f32,
}

impl Default for SchemaExplorerState {
    fn default() -> Self {
        Self {
            schema: UiSchemaSummary::default(),
            schema_symbol_index: SchemaSymbolIndex::default(),
            selected_schema: None,
            explorer_search: String::new(),
            explorer_nav_cache: None,
            schema_error: None,
            schema_request: None,
            selected_table: None,
            pinned_tables: Vec::new(),
            recent_tables: Vec::new(),
            selected_schema_object: None,
            schema_object_view: SchemaObjectView::Definition,
            connections_pane_height: 160.0,
            schemas_pane_height: 90.0,
        }
    }
}

impl SchemaExplorerState {
    pub(super) fn filter_by_schema<T: Clone>(&self, all: &[T], schema: &str, schema_of: impl Fn(&T) -> &str) -> Vec<T> {
        if self.schema.schemas.is_empty() || schema.is_empty() {
            all.to_vec()
        } else {
            all.iter().filter(|item| schema_of(item) == schema).cloned().collect()
        }
    }

    pub(super) fn count_by_schema<T>(&self, all: &[T], schema: &str, schema_of: impl Fn(&T) -> &str) -> usize {
        if self.schema.schemas.is_empty() || schema.is_empty() {
            all.len()
        } else {
            all.iter().filter(|item| schema_of(item) == schema).count()
        }
    }

    pub(super) fn matching_table_count(&self, schema: &str, query: &str) -> usize {
        super::connection_status::schema_matching_table_count(self, schema, query)
    }

    pub(super) fn cached_tables(
        &mut self,
        connection_id: &str,
        schema: &str,
        search_query: &str,
    ) -> (usize, usize, Vec<String>) {
        if let Some(cache) = self.explorer_nav_cache.as_ref() {
            if cache.connection_id == connection_id && cache.schema == schema && cache.search == search_query {
                return (cache.total_count, cache.matching_count, cache.visible.clone());
            }
        }

        let all_tables = super::connection_status::schema_table_names(self, schema);
        let (matching_count, visible) = filtered_explorer_tables(&all_tables, search_query);
        let total_count = all_tables.len();
        self.explorer_nav_cache = Some(ExplorerNavCache {
            connection_id: connection_id.to_owned(),
            schema: schema.to_owned(),
            search: search_query.to_owned(),
            total_count,
            matching_count,
            visible: visible.clone(),
        });
        (total_count, matching_count, visible)
    }

    pub(super) fn record_recent_table(&mut self, table: &str) {
        if table.is_empty() {
            return;
        }
        self.recent_tables.retain(|item| item != table);
        self.recent_tables.insert(0, table.to_owned());
        if self.recent_tables.len() > RECENT_TABLES_MAX {
            self.recent_tables.truncate(RECENT_TABLES_MAX);
        }
    }

    pub(super) fn remove_recent_table(&mut self, table: &str) {
        self.recent_tables.retain(|item| item != table);
    }
}

pub(crate) struct SchemaActivationContext<'a> {
    explorer: &'a mut SchemaExplorerState,
    table: &'a mut super::TableEditorState,
    workspace: &'a mut super::WorkspaceShellState,
    feedback: &'a mut super::FeedbackState,
}

impl<'a> SchemaActivationContext<'a> {
    pub(crate) fn new(
        explorer: &'a mut SchemaExplorerState,
        table: &'a mut super::TableEditorState,
        workspace: &'a mut super::WorkspaceShellState,
        feedback: &'a mut super::FeedbackState,
    ) -> Self {
        Self {
            explorer,
            table,
            workspace,
            feedback,
        }
    }

    pub(crate) fn activate(&mut self, schema: &str) {
        if self.explorer.selected_schema.as_deref() == Some(schema) {
            return;
        }
        if !self.table.mutation.staged_changes.is_empty() {
            self.workspace.pending_navigation_action =
                Some(super::PendingNavigationAction::ChangeSchema(schema.to_owned()));
            self.table.editing.discard_changes_confirmation = true;
            self.feedback
                .set_runtime_message("Apply or discard staged changes before changing schema");
            return;
        }
        self.workspace.pending_navigation_action = None;
        self.explorer.selected_schema = Some(schema.to_owned());
        self.explorer.selected_table = None;
        self.explorer.selected_schema_object = None;
        self.explorer.schema_object_view = super::SchemaObjectView::Definition;
        self.table.reset_workspace();
        self.explorer.explorer_nav_cache = None;
        self.workspace.active_tab = super::WorkspaceTab::Welcome;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_schema_explorer_state_has_no_selection_or_pending_request() {
        let state = SchemaExplorerState::default();

        assert!(state.schema.tables.is_empty());
        assert!(state.selected_schema.is_none());
        assert!(state.selected_table.is_none());
        assert!(state.schema_request.is_none());
        assert_eq!(state.schema_object_view, SchemaObjectView::Definition);
        assert_eq!(state.connections_pane_height, 160.0);
        assert_eq!(state.schemas_pane_height, 90.0);
    }

    #[test]
    fn recent_tables_are_owned_as_a_bounded_mru_list() {
        let mut state = SchemaExplorerState::default();

        state.record_recent_table("users");
        state.record_recent_table("orders");
        state.record_recent_table("users");
        state.remove_recent_table("orders");

        assert_eq!(state.recent_tables, vec!["users"]);
    }

    #[test]
    fn schema_activation_resets_table_workspace_and_welcome_surface() {
        let mut explorer = SchemaExplorerState {
            selected_schema: Some("public".to_owned()),
            selected_table: Some("users".to_owned()),
            selected_schema_object: Some(SchemaObjectSelection::View("active_view".to_owned())),
            schema_object_view: SchemaObjectView::Data,
            explorer_nav_cache: Some(ExplorerNavCache {
                connection_id: "conn-1".to_owned(),
                schema: "public".to_owned(),
                search: String::new(),
                total_count: 1,
                matching_count: 1,
                visible: vec!["users".to_owned()],
            }),
            ..Default::default()
        };
        let mut table = super::TableEditorState::default();
        table.mutation.pending_changes_open = true;
        table.editing.data_edit_value = "draft".to_owned();
        let mut workspace = super::WorkspaceShellState {
            active_tab: super::WorkspaceTab::Table,
            ..Default::default()
        };
        let mut feedback = super::FeedbackState::default();

        SchemaActivationContext::new(&mut explorer, &mut table, &mut workspace, &mut feedback).activate("analytics");

        assert_eq!(explorer.selected_schema.as_deref(), Some("analytics"));
        assert!(explorer.selected_table.is_none());
        assert!(explorer.selected_schema_object.is_none());
        assert_eq!(explorer.schema_object_view, SchemaObjectView::Definition);
        assert!(explorer.explorer_nav_cache.is_none());
        assert!(!table.mutation.pending_changes_open);
        assert!(table.editing.data_edit_value.is_empty());
        assert_eq!(workspace.active_tab, super::WorkspaceTab::Welcome);
    }
}
