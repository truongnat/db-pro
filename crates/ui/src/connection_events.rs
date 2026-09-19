//! Connection lifecycle and file-picker runtime events.

use super::*;
use crate::RequestId;

impl DbProApp {
    /// Connection list refreshed; auto-select and auto-connect the first one when nothing is active.
    pub(super) fn on_connections_loaded(&mut self, connections: Vec<UiConnectionSummary>) {
        self.connection_lifecycle.connections_request_pending = false;
        self.connection_catalog.replace(connections);
        if self.connection_lifecycle.active_connection_id.is_none() {
            self.connection_lifecycle.active_connection_id = self
                .connection_catalog
                .connections
                .first()
                .map(|connection| connection.id.clone());
        }
        if !self.connection_lifecycle.connected && self.connection_lifecycle.pending_request.is_none() {
            if let Some(active) = self.active_connection().cloned() {
                self.connect_to_connection(&active);
            }
        }
        self.feedback.runtime_message = format!("Loaded {} connections", self.connection_catalog.connections.len());
    }

    /// Connection established: load schema, saved queries and query folders.
    pub(super) fn on_connected(&mut self, request_id: RequestId, connection_id: String) {
        if self
            .connection_lifecycle
            .pending_request
            .is_some_and(|expected_request| expected_request != request_id)
        {
            return;
        }
        self.connection_lifecycle.pending_request = None;
        self.connection_lifecycle.pending_connection_id = None;
        self.connection_lifecycle.active_connection_id = Some(connection_id.clone());
        self.connection_lifecycle.connected = true;
        self.connection_lifecycle.clear_connection_error(&connection_id);
        self.feedback.runtime_message = "Connection established".to_owned();
        if let Some(connection_id) = self.connection_lifecycle.active_connection_id.clone() {
            self.request_schema_introspection(connection_id.clone(), false);
            let request_id = self.task_bridge.next_request_id();
            self.dispatch_command(UiCommand::ListSavedQueries {
                request_id,
                connection_id: connection_id.clone(),
            });
            let request_id = self.task_bridge.next_request_id();
            self.dispatch_command(UiCommand::ListQueryFolders {
                request_id,
                connection_id,
            });
        }
    }
}
