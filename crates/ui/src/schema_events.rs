//! Schema-introspection runtime events and selection reconciliation.

use super::*;
use crate::RequestId;

impl DbProApp {
    /// Schema introspection result, revalidating the current schema/table/object selection.
    pub(super) fn on_schema_loaded(&mut self, request_id: RequestId, schema: UiSchemaSummary) {
        if self
            .schema_explorer
            .schema_request
            .is_some_and(|expected_request| expected_request != request_id)
        {
            return;
        }
        self.schema_explorer.schema_request = None;
        self.schema_explorer.schema_error = None;
        let refresh_selected_table = self.table_state.refresh_table_info_after_schema;
        self.table_state.refresh_table_info_after_schema = false;
        let mut schema = schema;
        schema.schemas.retain(|name| is_user_visible_schema(name));
        self.schema_explorer.schema_symbol_index = SchemaSymbolIndex::build(&schema);
        self.schema_explorer.schema = schema;
        self.schema_explorer.explorer_nav_cache = None;
        self.palette.search_index.invalidate();
        if self.schema_explorer.selected_schema.as_ref().is_none_or(|selected| {
            !self
                .schema_explorer
                .schema
                .schemas
                .iter()
                .any(|schema| schema == selected)
        }) {
            self.schema_explorer.selected_schema = self.schema_explorer.schema.schemas.first().cloned();
        }
        if self.schema_explorer.selected_table.as_ref().is_some_and(|table| {
            !self
                .schema_explorer
                .schema
                .tables
                .iter()
                .any(|candidate| candidate == table)
        }) {
            self.clear_missing_selected_table();
        }
        if !self.selected_schema_object_exists() {
            self.schema_explorer.selected_schema_object = None;
            if self.workspace.active_tab == WorkspaceTab::SchemaObject {
                self.activate_welcome_tab();
            }
        }
        self.feedback.runtime_message = format!(
            "Schema loaded · {} tables · {} views · {} triggers · {} functions",
            self.schema_explorer.schema.tables.len(),
            self.schema_explorer.schema.views.len(),
            self.schema_explorer.schema.triggers.len(),
            self.schema_explorer.schema.functions.len()
        );
        if refresh_selected_table
            && self.workspace.active_tab == WorkspaceTab::Table
            && self.schema_explorer.selected_table.is_some()
        {
            self.request_table_info();
        }
    }

    /// Drops the selected table after a refresh proves it no longer exists.
    fn clear_missing_selected_table(&mut self) {
        self.schema_explorer.selected_table = None;
        self.schema_explorer.selected_schema_object = None;
        self.table_state.table_info = None;
        self.table_state.table_ddl = None;
        self.table_state.table_info_error = None;
        self.table_state.table_ddl_error = None;
        self.table_state.ddl_execute_confirmation = false;
        self.table_state.ddl_execution_request = None;
        self.table_state.table_data_result = None;
        self.table_state.table_data_total_rows = None;
        self.table_state.table_data_offset = 0;
        self.table_state.table_data_filter_column.clear();
        self.table_state.table_data_filter_operator = UiTableFilterOperator::default();
        self.table_state.table_data_filter_value.clear();
        self.table_state.table_data_filters.clear();
        self.table_state.table_data_sorts.clear();
        self.table_state.table_data_error = None;
        self.table_state.table_info_request = None;
        self.table_state.table_ddl_request = None;
        self.table_state.table_data_request = None;
        if self.workspace.active_tab == WorkspaceTab::Table {
            self.activate_welcome_tab();
        }
    }

    /// Whether the selected schema object (view / trigger / function) still exists.
    fn selected_schema_object_exists(&self) -> bool {
        match self.schema_explorer.selected_schema_object.as_ref() {
            Some(SchemaObjectSelection::View(name)) => {
                self.schema_explorer.schema.views.iter().any(|view| &view.name == name)
            }
            Some(SchemaObjectSelection::Trigger(name)) => self
                .schema_explorer
                .schema
                .triggers
                .iter()
                .any(|trigger| &trigger.name == name),
            Some(SchemaObjectSelection::Function {
                name,
                identity_arguments,
            }) => self
                .schema_explorer
                .schema
                .functions
                .iter()
                .any(|function| &function.name == name && &function.identity_arguments == identity_arguments),
            None => true,
        }
    }
}
