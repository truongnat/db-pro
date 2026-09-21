//! Cross-feature state transitions initiated by the database explorer.

use super::{
    AgentState, ConnectionLifecycleState, FeedbackState, PendingNavigationAction, QueryExecutionPolicyState,
    SchemaExplorerState, TableEditorState, UiCommand, UiConnectionSummary, WorkspaceShellState,
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
}
