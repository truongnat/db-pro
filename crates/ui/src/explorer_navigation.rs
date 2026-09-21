//! Cross-feature state transitions initiated by the database explorer.

use super::{
    AgentState, ConnectionLifecycleState, FeedbackState, PendingNavigationAction, QueryExecutionPolicyState,
    RoutineState, SchemaExplorerState, SchemaObjectSelection, TableEditorState, UiCommand, UiConnectionSummary,
    WorkspaceShellState,
};
use crate::RequestId;

pub(crate) struct ExplorerConnectionContext<'a> {
    lifecycle: &'a mut ConnectionLifecycleState,
    schema: &'a mut SchemaExplorerState,
    table: &'a mut TableEditorState,
    workspace: &'a mut WorkspaceShellState,
    agent: &'a mut AgentState,
    execution: &'a mut QueryExecutionPolicyState,
    feedback: &'a mut FeedbackState,
}

pub(crate) struct TableSelectionContext<'a> {
    schema: &'a mut SchemaExplorerState,
    table: &'a mut TableEditorState,
    workspace: &'a mut WorkspaceShellState,
    feedback: &'a mut FeedbackState,
}

pub(crate) struct SchemaObjectActivationContext<'a> {
    explorer: &'a mut SchemaExplorerState,
    table: &'a mut TableEditorState,
    workspace: &'a mut WorkspaceShellState,
    routine: &'a mut RoutineState,
    feedback: &'a mut FeedbackState,
}

pub(crate) struct SchemaObjectActivation {
    pub(crate) selection: SchemaObjectSelection,
    pub(crate) schema: String,
    pub(crate) name: String,
    pub(crate) kind: String,
}

impl<'a> SchemaObjectActivationContext<'a> {
    pub(crate) fn new(
        explorer: &'a mut SchemaExplorerState,
        table: &'a mut TableEditorState,
        workspace: &'a mut WorkspaceShellState,
        routine: &'a mut RoutineState,
        feedback: &'a mut FeedbackState,
    ) -> Self {
        Self {
            explorer,
            table,
            workspace,
            routine,
            feedback,
        }
    }

    pub(crate) fn open(&mut self, request: SchemaObjectActivation) {
        let SchemaObjectActivation {
            selection,
            schema,
            name,
            kind,
        } = request;
        let function = if let SchemaObjectSelection::Function {
            name: function_name,
            identity_arguments,
        } = &selection
        {
            self.explorer
                .schema
                .functions
                .iter()
                .find(|function| &function.name == function_name && &function.identity_arguments == identity_arguments)
                .cloned()
        } else {
            None
        };

        self.explorer.selected_schema_object = Some(selection);
        self.explorer.schema_object_view = super::SchemaObjectView::Definition;
        self.explorer.selected_table = None;
        self.table.reset_workspace();
        self.table.state.table_view = super::TableView::Ddl;
        self.workspace.active_tab = super::WorkspaceTab::SchemaObject;
        self.routine.routine_drop_confirm = false;
        self.routine.routine_ddl_preview = None;
        if let Some(function) = function {
            self.routine.sync_from(&function);
        }
        self.feedback.set_runtime_message(if schema.is_empty() {
            format!("Opened {kind} {name}")
        } else {
            format!("Opened {kind} {schema}.{name}")
        });
    }
}

impl<'a> TableSelectionContext<'a> {
    pub(crate) fn new(
        schema: &'a mut SchemaExplorerState,
        table: &'a mut TableEditorState,
        workspace: &'a mut WorkspaceShellState,
        feedback: &'a mut FeedbackState,
    ) -> Self {
        Self {
            schema,
            table,
            workspace,
            feedback,
        }
    }

    pub(crate) fn select(&mut self, table_name: &str, connection_id: Option<&str>, schema_name: &str) -> bool {
        if self.schema.selected_table.as_deref() != Some(table_name) && !self.table.mutation.staged_changes.is_empty() {
            self.feedback
                .set_runtime_message("Apply or discard staged changes before opening another table");
            return false;
        }

        let previous_scope =
            super::TableDataState::layout_scope(connection_id, schema_name, self.schema.selected_table.as_deref());
        self.table.data.persist_layout(previous_scope);
        self.schema.selected_table = Some(table_name.to_owned());
        self.schema.record_recent_table(table_name);
        self.schema.selected_schema_object = None;
        self.schema.schema_object_view = super::SchemaObjectView::Definition;
        self.table.reset_workspace();
        let next_scope = super::TableDataState::layout_scope(connection_id, schema_name, Some(table_name));
        self.table.data.restore_layout(next_scope);
        self.table.state.table_view = super::TableView::Data;
        self.workspace.active_tab = super::WorkspaceTab::Table;
        true
    }
}

impl<'a> ExplorerConnectionContext<'a> {
    pub(crate) fn new(
        lifecycle: &'a mut ConnectionLifecycleState,
        schema: &'a mut SchemaExplorerState,
        table: &'a mut TableEditorState,
        workspace: &'a mut WorkspaceShellState,
        agent: &'a mut AgentState,
        execution: &'a mut QueryExecutionPolicyState,
        feedback: &'a mut FeedbackState,
    ) -> Self {
        Self {
            lifecycle,
            schema,
            table,
            workspace,
            agent,
            execution,
            feedback,
        }
    }

    pub(crate) fn disconnect(&mut self, connection: &UiConnectionSummary) -> bool {
        if self.execution.query_in_transaction {
            self.execution.disconnect_txn_guard = true;
            self.feedback
                .set_runtime_message("Open transaction detected — commit or rollback before disconnecting");
            return false;
        }

        self.lifecycle.set_connected(false);
        self.schema.reset_connection_scope();
        self.table.reset_workspace();
        self.feedback
            .set_runtime_message(format!("Disconnected from {}", connection.name));
        true
    }

    pub(crate) fn connect(&mut self, connection: &UiConnectionSummary, request_id: RequestId) -> Option<UiCommand> {
        if self.lifecycle.active_connection_id() == Some(connection.id.as_str()) && self.lifecycle.is_connected() {
            return None;
        }
        if !self.table.mutation.staged_changes.is_empty() {
            self.workspace.pending_navigation_action =
                Some(PendingNavigationAction::ChangeConnection(connection.id.clone()));
            self.table.editing.discard_changes_confirmation = true;
            self.feedback
                .set_runtime_message("Apply or discard staged changes before changing connection");
            return None;
        }
        if self.execution.query_in_transaction {
            self.execution.disconnect_txn_guard = true;
            self.feedback
                .set_runtime_message("Commit or rollback the open transaction before changing connection");
            return None;
        }

        self.workspace.pending_navigation_action = None;
        self.agent.clear_input();
        *self.lifecycle.active_connection_id_mut() = Some(connection.id.clone());
        self.lifecycle.set_pending_connection_id(Some(connection.id.clone()));
        self.lifecycle.clear_connection_error(&connection.id);
        self.schema.reset_connection_scope();
        self.table.reset_workspace();
        self.lifecycle.set_connected(false);
        self.lifecycle.set_pending_request(Some(request_id));
        self.feedback
            .set_runtime_message(format!("Connecting to {}…", connection.name));

        Some(self.lifecycle.connect_command(request_id, connection.id.clone()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn connection() -> UiConnectionSummary {
        UiConnectionSummary {
            id: "conn-1".to_owned(),
            name: "Analytics".to_owned(),
            host: "localhost".to_owned(),
            port: 5432,
            database: "analytics".to_owned(),
            username: "postgres".to_owned(),
            driver: "PostgreSQL".to_owned(),
            ssl_mode: crate::UiSslMode::Require,
            readonly: false,
            tags: Vec::new(),
            group: None,
            favorite: false,
            environment: "Development".to_owned(),
        }
    }

    #[test]
    fn connect_blocks_when_staged_changes_are_pending() {
        let mut lifecycle = ConnectionLifecycleState::default();
        let mut schema = SchemaExplorerState::default();
        let mut table = TableEditorState::default();
        table.mutation.staged_changes.stage_insert(Vec::new(), Vec::new());
        let mut workspace = WorkspaceShellState::default();
        let mut agent = AgentState::default();
        let mut execution = QueryExecutionPolicyState::default();
        let mut feedback = FeedbackState::default();
        let connection = connection();

        let command = ExplorerConnectionContext::new(
            &mut lifecycle,
            &mut schema,
            &mut table,
            &mut workspace,
            &mut agent,
            &mut execution,
            &mut feedback,
        )
        .connect(&connection, RequestId(1));

        assert!(command.is_none());
        assert_eq!(
            workspace.pending_navigation_action,
            Some(PendingNavigationAction::ChangeConnection("conn-1".to_owned()))
        );
        assert!(table.editing.discard_changes_confirmation);
        assert!(lifecycle.active_connection_id().is_none());
    }

    #[test]
    fn connect_resets_connection_scoped_workspace_before_dispatch() {
        let mut lifecycle = ConnectionLifecycleState::default();
        let mut schema = SchemaExplorerState {
            selected_schema: Some("public".to_owned()),
            selected_table: Some("users".to_owned()),
            ..Default::default()
        };
        let mut table = TableEditorState::default();
        table.editing.data_edit_value = "draft".to_owned();
        let mut workspace = WorkspaceShellState {
            pending_navigation_action: Some(PendingNavigationAction::ChangeSchema("public".to_owned())),
            ..Default::default()
        };
        let mut agent = AgentState {
            input: "old prompt".to_owned(),
            ..Default::default()
        };
        let mut execution = QueryExecutionPolicyState::default();
        let mut feedback = FeedbackState::default();
        let connection = connection();

        let command = ExplorerConnectionContext::new(
            &mut lifecycle,
            &mut schema,
            &mut table,
            &mut workspace,
            &mut agent,
            &mut execution,
            &mut feedback,
        )
        .connect(&connection, RequestId(7));

        assert!(
            matches!(command, Some(UiCommand::Connect { request_id: RequestId(7), connection_id }) if connection_id == "conn-1")
        );
        assert_eq!(lifecycle.active_connection_id(), Some("conn-1"));
        assert_eq!(lifecycle.pending_connection_id(), Some("conn-1"));
        assert_eq!(lifecycle.pending_request(), Some(RequestId(7)));
        assert!(schema.selected_schema.is_none());
        assert!(schema.selected_table.is_none());
        assert!(table.editing.data_edit_value.is_empty());
        assert!(workspace.pending_navigation_action.is_none());
        assert!(agent.input.is_empty());
        assert!(feedback.runtime_message.contains("Connecting to Analytics"));
    }

    #[test]
    fn table_selection_blocks_staged_changes_without_changing_selection() {
        let mut schema = SchemaExplorerState {
            selected_table: Some("users".to_owned()),
            ..Default::default()
        };
        let mut table = TableEditorState::default();
        table.mutation.staged_changes.stage_insert(Vec::new(), Vec::new());
        let mut workspace = WorkspaceShellState::default();
        let mut feedback = FeedbackState::default();

        let selected = TableSelectionContext::new(&mut schema, &mut table, &mut workspace, &mut feedback).select(
            "orders",
            Some("conn-1"),
            "public",
        );

        assert!(!selected);
        assert_eq!(schema.selected_table.as_deref(), Some("users"));
        assert!(feedback.runtime_message.contains("Apply or discard"));
    }

    #[test]
    fn table_selection_resets_workspace_and_opens_table_surface() {
        let mut schema = SchemaExplorerState {
            selected_table: Some("users".to_owned()),
            selected_schema_object: Some(super::super::SchemaObjectSelection::View("active_view".to_owned())),
            schema_object_view: super::super::SchemaObjectView::Data,
            ..Default::default()
        };
        let mut table = TableEditorState::default();
        table.editing.data_edit_value = "draft".to_owned();
        let mut workspace = WorkspaceShellState {
            active_tab: super::super::WorkspaceTab::SchemaObject,
            ..Default::default()
        };
        let mut feedback = FeedbackState::default();

        let selected = TableSelectionContext::new(&mut schema, &mut table, &mut workspace, &mut feedback).select(
            "orders",
            Some("conn-1"),
            "public",
        );

        assert!(selected);
        assert_eq!(schema.selected_table.as_deref(), Some("orders"));
        assert!(schema.selected_schema_object.is_none());
        assert_eq!(schema.schema_object_view, super::super::SchemaObjectView::Definition);
        assert!(table.editing.data_edit_value.is_empty());
        assert_eq!(table.state.table_view, super::super::TableView::Data);
        assert_eq!(workspace.active_tab, super::super::WorkspaceTab::Table);
    }

    #[test]
    fn schema_object_activation_resets_table_surface_and_opens_definition() {
        let mut schema = SchemaExplorerState {
            selected_table: Some("users".to_owned()),
            ..Default::default()
        };
        let mut table = TableEditorState::default();
        table.editing.data_edit_value = "draft".to_owned();
        let mut workspace = WorkspaceShellState {
            active_tab: super::super::WorkspaceTab::Table,
            ..Default::default()
        };
        let mut routine = RoutineState {
            routine_drop_confirm: true,
            routine_ddl_preview: Some("stale ddl".to_owned()),
            ..Default::default()
        };
        let mut feedback = FeedbackState::default();

        SchemaObjectActivationContext::new(&mut schema, &mut table, &mut workspace, &mut routine, &mut feedback).open(
            SchemaObjectActivation {
                selection: SchemaObjectSelection::View("orders_view".to_owned()),
                schema: "public".to_owned(),
                name: "orders_view".to_owned(),
                kind: "View".to_owned(),
            },
        );

        assert!(schema.selected_table.is_none());
        assert!(
            matches!(schema.selected_schema_object, Some(SchemaObjectSelection::View(name)) if name == "orders_view")
        );
        assert_eq!(schema.schema_object_view, super::super::SchemaObjectView::Definition);
        assert!(table.editing.data_edit_value.is_empty());
        assert_eq!(table.state.table_view, super::super::TableView::Ddl);
        assert_eq!(workspace.active_tab, super::super::WorkspaceTab::SchemaObject);
        assert!(!routine.routine_drop_confirm);
        assert!(routine.routine_ddl_preview.is_none());
        assert_eq!(feedback.runtime_message, "Opened View public.orders_view");
    }
}
