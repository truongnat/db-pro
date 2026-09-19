//! Connection lifecycle reducers over explicit feature state.

use super::*;
use crate::RequestId;

pub(super) fn handle_connection_request_failure(
    lifecycle: &mut ConnectionLifecycleState,
    dialog: &mut ConnectionDialogState,
    schema_explorer: &mut SchemaExplorerState,
    feedback: &mut FeedbackState,
    request_id: RequestId,
    message: &str,
) -> bool {
    if lifecycle.pending_request() != Some(request_id) {
        return false;
    }

    lifecycle.clear_pending_request();
    let connection_id = lifecycle
        .take_pending_connection_id()
        .or_else(|| lifecycle.active_connection_id().map(str::to_owned));
    let is_delete = feedback.runtime_message.to_ascii_lowercase().contains("delet");
    if is_delete {
        let formatted = format!("Delete failed · {message}");
        feedback.set_runtime_message(formatted.clone());
        feedback.show_error_toast(formatted);
    } else {
        if let Some(connection_id) = connection_id {
            lifecycle.record_connection_failure(connection_id, message.to_owned());
        }
        if !dialog.is_open() {
            lifecycle.set_connected(false);
            schema_explorer.schema_request = None;
            schema_explorer.schema_error = None;
        }
        dialog.set_error(message);
        feedback.set_runtime_message(format!("Connection failed · {message}"));
    }
    true
}

/// Replace the connection read model and return the connection that should be
/// auto-connected by the composition root, if any.
pub(super) fn on_connections_loaded(
    lifecycle: &mut ConnectionLifecycleState,
    catalog: &mut ConnectionCatalogState,
    feedback: &mut FeedbackState,
    connections: Vec<UiConnectionSummary>,
) -> Option<UiConnectionSummary> {
    lifecycle.set_connections_request_pending(false);
    catalog.replace(connections);
    if lifecycle.active_connection_id().is_none() {
        *lifecycle.active_connection_id_mut() = catalog.get(0).map(|connection| connection.id.clone());
    }
    let should_connect = !lifecycle.is_connected() && lifecycle.pending_request().is_none();
    let active = should_connect
        .then(|| lifecycle.active_connection_id().map(str::to_owned))
        .flatten()
        .and_then(|connection_id| catalog.find(&connection_id).cloned());
    feedback.set_runtime_message(format!("Loaded {} connections", catalog.len()));
    active
}

/// Apply the authoritative connected transition and return the active id for
/// the follow-up schema/query refreshes owned by the composition root.
pub(super) fn on_connected(
    lifecycle: &mut ConnectionLifecycleState,
    feedback: &mut FeedbackState,
    request_id: RequestId,
    connection_id: String,
) -> Option<String> {
    if lifecycle
        .pending_request()
        .is_some_and(|expected_request| expected_request != request_id)
    {
        return None;
    }
    lifecycle.clear_pending_request();
    lifecycle.set_pending_connection_id(None);
    *lifecycle.active_connection_id_mut() = Some(connection_id.clone());
    lifecycle.set_connected(true);
    lifecycle.clear_connection_error(&connection_id);
    feedback.set_runtime_message("Connection established");
    Some(connection_id)
}
