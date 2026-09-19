//! Schema-introspection reducers and selection reconciliation.

use super::*;
use crate::RequestId;

/// Follow-up work requested by a successful schema transition.
#[derive(Debug, Default, PartialEq, Eq)]
pub(crate) struct SchemaLoadedTransition {
    pub(crate) refresh_selected_table: bool,
}

/// Applies a schema request failure only when it belongs to the pending request.
pub(crate) fn handle_schema_request_failure(
    schema_explorer: &mut SchemaExplorerState,
    feedback: &mut FeedbackState,
    request_id: RequestId,
    message: &str,
) -> bool {
    if schema_explorer.schema_request != Some(request_id) {
        return false;
    }
    schema_explorer.schema_request = None;
    schema_explorer.schema_error = Some(message.to_owned());
    feedback.runtime_message = format!("Schema introspection failed · {message}");
    true
}

/// Applies an introspection result and reconciles selections against the new read model.
pub(crate) fn on_schema_loaded(
    schema_explorer: &mut SchemaExplorerState,
    table_state: &mut TableState,
    workspace: &mut WorkspaceShellState,
    palette: &mut PaletteState,
    feedback: &mut FeedbackState,
    request_id: RequestId,
    mut schema: UiSchemaSummary,
) -> SchemaLoadedTransition {
    if schema_explorer
        .schema_request
        .is_some_and(|expected_request| expected_request != request_id)
    {
        return SchemaLoadedTransition::default();
    }

    schema_explorer.schema_request = None;
    schema_explorer.schema_error = None;
    let refresh_selected_table = table_state.refresh_table_info_after_schema;
    table_state.refresh_table_info_after_schema = false;

    schema.schemas.retain(|name| is_user_visible_schema(name));
    schema_explorer.schema_symbol_index = SchemaSymbolIndex::build(&schema);
    schema_explorer.schema = schema;
    schema_explorer.explorer_nav_cache = None;
    palette.search_index.invalidate();

    if schema_explorer
        .selected_schema
        .as_ref()
        .is_none_or(|selected| !schema_explorer.schema.schemas.iter().any(|schema| schema == selected))
    {
        schema_explorer.selected_schema = schema_explorer.schema.schemas.first().cloned();
    }

    if schema_explorer
        .selected_table
        .as_ref()
        .is_some_and(|table| !schema_explorer.schema.tables.iter().any(|candidate| candidate == table))
    {
        clear_missing_selected_table(schema_explorer, table_state, workspace);
    }

    if !selected_schema_object_exists(schema_explorer) {
        schema_explorer.selected_schema_object = None;
        if workspace.active_tab == WorkspaceTab::SchemaObject {
            activate_welcome_tab(workspace);
        }
    }

    feedback.runtime_message = format!(
        "Schema loaded · {} tables · {} views · {} triggers · {} functions",
        schema_explorer.schema.tables.len(),
        schema_explorer.schema.views.len(),
        schema_explorer.schema.triggers.len(),
        schema_explorer.schema.functions.len()
    );

    SchemaLoadedTransition {
        refresh_selected_table: refresh_selected_table
            && workspace.active_tab == WorkspaceTab::Table
            && schema_explorer.selected_table.is_some(),
    }
}

/// Clears table state when a refresh proves the selected table no longer exists.
fn clear_missing_selected_table(
    schema_explorer: &mut SchemaExplorerState,
    table_state: &mut TableState,
    workspace: &mut WorkspaceShellState,
) {
    schema_explorer.selected_table = None;
    schema_explorer.selected_schema_object = None;
    table_state.table_info = None;
    table_state.table_ddl = None;
    table_state.table_info_error = None;
    table_state.table_ddl_error = None;
    table_state.ddl_execute_confirmation = false;
    table_state.ddl_execution_request = None;
    table_state.table_data_result = None;
    table_state.table_data_total_rows = None;
    table_state.table_data_offset = 0;
    table_state.table_data_filter_column.clear();
    table_state.table_data_filter_operator = UiTableFilterOperator::default();
    table_state.table_data_filter_value.clear();
    table_state.table_data_filters.clear();
    table_state.table_data_sorts.clear();
    table_state.table_data_error = None;
    table_state.table_info_request = None;
    table_state.table_ddl_request = None;
    table_state.table_data_request = None;
    if workspace.active_tab == WorkspaceTab::Table {
        activate_welcome_tab(workspace);
    }
}

/// Whether the selected schema object still exists in the refreshed read model.
fn selected_schema_object_exists(schema_explorer: &SchemaExplorerState) -> bool {
    match schema_explorer.selected_schema_object.as_ref() {
        Some(SchemaObjectSelection::View(name)) => schema_explorer.schema.views.iter().any(|view| &view.name == name),
        Some(SchemaObjectSelection::Trigger(name)) => schema_explorer
            .schema
            .triggers
            .iter()
            .any(|trigger| &trigger.name == name),
        Some(SchemaObjectSelection::Function {
            name,
            identity_arguments,
        }) => schema_explorer
            .schema
            .functions
            .iter()
            .any(|function| &function.name == name && &function.identity_arguments == identity_arguments),
        None => true,
    }
}

fn activate_welcome_tab(workspace: &mut WorkspaceShellState) {
    workspace.welcome_open = true;
    workspace.active_tab = WorkspaceTab::Welcome;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stale_schema_results_do_not_mutate_state() {
        let mut schema_explorer = SchemaExplorerState {
            schema_request: Some(RequestId(7)),
            ..Default::default()
        };
        let mut table_state = TableState::default();
        let mut workspace = WorkspaceShellState::default();
        let mut palette = PaletteState::default();
        let mut feedback = FeedbackState::default();

        let transition = on_schema_loaded(
            &mut schema_explorer,
            &mut table_state,
            &mut workspace,
            &mut palette,
            &mut feedback,
            RequestId(8),
            UiSchemaSummary::default(),
        );

        assert_eq!(transition, SchemaLoadedTransition::default());
        assert_eq!(schema_explorer.schema_request, Some(RequestId(7)));
        assert!(schema_explorer.schema.tables.is_empty());
    }

    #[test]
    fn missing_selected_table_clears_table_state_and_returns_to_welcome() {
        let mut schema_explorer = SchemaExplorerState {
            schema_request: Some(RequestId(7)),
            selected_table: Some("gone".to_owned()),
            ..Default::default()
        };
        let mut table_state = TableState {
            table_info: Some(UiTableInfo {
                schema: "public".to_owned(),
                name: "gone".to_owned(),
                row_count: None,
                columns: Vec::new(),
                primary_key: None,
                indexes: Vec::new(),
                foreign_keys: Vec::new(),
                check_constraints: Vec::new(),
                dependencies: Vec::new(),
            }),
            refresh_table_info_after_schema: true,
            ..Default::default()
        };
        let mut workspace = WorkspaceShellState {
            active_tab: WorkspaceTab::Table,
            ..Default::default()
        };
        let mut palette = PaletteState::default();
        let mut feedback = FeedbackState::default();

        let transition = on_schema_loaded(
            &mut schema_explorer,
            &mut table_state,
            &mut workspace,
            &mut palette,
            &mut feedback,
            RequestId(7),
            UiSchemaSummary::default(),
        );

        assert!(!transition.refresh_selected_table);
        assert!(schema_explorer.selected_table.is_none());
        assert!(table_state.table_info.is_none());
        assert_eq!(workspace.active_tab, WorkspaceTab::Welcome);
    }
}
