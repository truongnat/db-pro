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
}
