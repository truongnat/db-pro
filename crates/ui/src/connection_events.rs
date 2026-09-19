//! Connection lifecycle and file-picker runtime events.

use super::*;
use crate::RequestId;

impl DbProApp {
    pub(super) fn handle_connection_request_failure(&mut self, request_id: RequestId, message: &str) -> bool {
        if self.connection_lifecycle.pending_request != Some(request_id) {
            return false;
        }

        self.connection_lifecycle.clear_pending_request();
        let connection_id = self
            .connection_lifecycle
            .pending_connection_id
            .take()
            .or_else(|| self.connection_lifecycle.active_connection_id().map(str::to_owned));
        let is_delete = self.feedback.runtime_message.to_ascii_lowercase().contains("delet");
        if is_delete {
            let formatted = format!("Delete failed · {message}");
            self.feedback.runtime_message = formatted.clone();
            self.show_toast_error(formatted);
        } else {
            if let Some(connection_id) = connection_id {
                self.connection_lifecycle
                    .failed_connection_ids
                    .insert(connection_id.clone());
                self.connection_lifecycle
                    .errors
                    .insert(connection_id, message.to_owned());
            }
            if !self.connection_dialog.is_open() {
                self.connection_lifecycle.set_connected(false);
                self.schema_explorer.schema_request = None;
                self.schema_explorer.schema_error = None;
            }
            self.connection_dialog.set_error(message);
            self.feedback.runtime_message = format!("Connection failed · {message}");
        }
        true
    }

    /// Connection list refreshed; auto-select and auto-connect the first one when nothing is active.
    pub(super) fn on_connections_loaded(&mut self, connections: Vec<UiConnectionSummary>) {
        self.connection_lifecycle.set_connections_request_pending(false);
        self.connection_catalog.replace(connections);
        if self.connection_lifecycle.active_connection_id().is_none() {
            *self.connection_lifecycle.active_connection_id_mut() =
                self.connection_catalog.get(0).map(|connection| connection.id.clone());
        }
        if !self.connection_lifecycle.is_connected() && self.connection_lifecycle.pending_request.is_none() {
            if let Some(active) = self.active_connection().cloned() {
                self.connect_to_connection(&active);
            }
        }
        self.feedback.runtime_message = format!("Loaded {} connections", self.connection_catalog.len());
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
        *self.connection_lifecycle.active_connection_id_mut() = Some(connection_id.clone());
        self.connection_lifecycle.set_connected(true);
        self.connection_lifecycle.clear_connection_error(&connection_id);
        self.feedback.runtime_message = "Connection established".to_owned();
        if let Some(connection_id) = self.connection_lifecycle.active_connection_id().map(str::to_owned) {
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
