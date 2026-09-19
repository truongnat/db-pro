use super::*;

/// Owns database schema navigation, selection and explorer read-model state.
#[derive(Debug)]
pub(crate) struct SchemaExplorerState {
    pub(crate) schema: UiSchemaSummary,
    pub(crate) schema_symbol_index: SchemaSymbolIndex,
    pub(crate) selected_schema: Option<String>,
    pub(crate) explorer_search: String,
    pub(crate) explorer_nav_cache: Option<ExplorerNavCache>,
    pub(crate) schema_error: Option<String>,
    pub(crate) schema_request: Option<crate::RequestId>,
    pub(crate) selected_table: Option<String>,
    pub(crate) pinned_tables: Vec<String>,
    pub(crate) recent_tables: Vec<String>,
    pub(crate) selected_schema_object: Option<SchemaObjectSelection>,
    pub(crate) schema_object_view: SchemaObjectView,
    pub(crate) connections_pane_height: f32,
    pub(crate) schemas_pane_height: f32,
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
}
