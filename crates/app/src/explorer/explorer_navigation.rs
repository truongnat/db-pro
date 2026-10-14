//! Cross-feature state transitions initiated by the database explorer.

use super::{
    AgentState, ConnectionLifecycleState, FeedbackState, PendingNavigationAction, QueryExecutionPolicyState,
    RoutineState, SchemaExplorerState, SchemaObjectSelection, TableEditorState, UiConnectionSummary,
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

pub(crate) struct SchemaObjectActivationContext<'a> {
    explorer: &'a mut SchemaExplorerState,
    table: &'a mut TableEditorState,
    workspace: &'a mut WorkspaceShellState,
    routine: &'a mut RoutineState,
    feedback: &'a mut FeedbackState,
}

#[derive(Clone)]
pub(crate) struct SchemaObjectActivation {
    pub(crate) selection: SchemaObjectSelection,
    pub(crate) schema: String,
    pub(crate) name: String,
    pub(crate) kind: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ConnectRequest {
    pub(crate) connection_id: String,
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
        // Staged edits are wiped by `reset_workspace` below — gate on the same
        // discard-changes confirmation the connect path uses so disconnecting
        // never silently drops inserts/updates/deletes.
        if !self.table.mutation.staged_changes.is_empty() {
            self.workspace.pending_navigation_action =
                Some(PendingNavigationAction::Disconnect(connection.id.clone()));
            self.table.editing.discard_changes_confirmation = true;
            self.feedback
                .set_runtime_message("Apply or discard staged changes before disconnecting");
            return false;
        }
        if self.execution.any_transaction_busy() {
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

    pub(crate) fn connect(&mut self, connection: &UiConnectionSummary) -> Option<ConnectRequest> {
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
        if self.execution.any_transaction_busy() {
            self.execution.disconnect_txn_guard = true;
            self.feedback
                .set_runtime_message("Commit or rollback the open transaction before changing connection");
            return None;
        }

        Some(ConnectRequest {
            connection_id: connection.id.clone(),
        })
    }

    pub(crate) fn commit_connect(&mut self, connection: &UiConnectionSummary, request_id: RequestId) {
        self.workspace.pending_navigation_action = None;
        self.agent.clear_input();
        self.lifecycle.set_active_connection_id(Some(connection.id.clone()));
        self.lifecycle.set_pending_connection_id(Some(connection.id.clone()));
        self.lifecycle.clear_connection_error(&connection.id);
        self.schema.reset_connection_scope();
        self.table.reset_workspace();
        self.lifecycle.set_connected(false);
        self.lifecycle.set_pending_request(Some(request_id));
        self.lifecycle
            .set_pending_operation(Some(super::connection::PendingConnectionOperation::Connect));
        self.feedback
            .set_runtime_message(format!("Connecting to {}…", connection.name));
    }
}

#[cfg(test)]
#[path = "tests/explorer_navigation_tests.rs"]
mod tests;

