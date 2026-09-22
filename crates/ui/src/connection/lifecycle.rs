use crate::RequestId;
use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum PendingConnectionOperation {
    Connect,
    Test,
    Save,
    Delete,
}

/// Connection lifecycle state owned by the connection feature.
///
/// The saved connection collection remains in `DbProApp` for the next
/// migration step because it is also used as the explorer's read model.
#[derive(Debug, Default)]
pub(crate) struct ConnectionLifecycleState {
    #[cfg(test)]
    pub(in crate::app) fallback_name: String,
    #[cfg(not(test))]
    fallback_name: String,
    #[cfg(test)]
    pub(in crate::app) connected: bool,
    #[cfg(not(test))]
    connected: bool,
    #[cfg(test)]
    pub(in crate::app) active_connection_id: Option<String>,
    #[cfg(not(test))]
    active_connection_id: Option<String>,
    #[cfg(test)]
    pub(in crate::app) pending_connection_id: Option<String>,
    #[cfg(not(test))]
    pending_connection_id: Option<String>,
    #[cfg(test)]
    pub(in crate::app) pending_request: Option<RequestId>,
    #[cfg(not(test))]
    pending_request: Option<RequestId>,
    #[cfg(test)]
    pub(in crate::app) pending_operation: Option<PendingConnectionOperation>,
    #[cfg(not(test))]
    pending_operation: Option<PendingConnectionOperation>,
    #[cfg(test)]
    pub(in crate::app) errors: HashMap<String, String>,
    #[cfg(not(test))]
    errors: HashMap<String, String>,
    #[cfg(test)]
    pub(in crate::app) failed_connection_ids: HashSet<String>,
    #[cfg(not(test))]
    failed_connection_ids: HashSet<String>,
    #[cfg(test)]
    pub(in crate::app) connections_requested: bool,
    #[cfg(not(test))]
    connections_requested: bool,
    #[cfg(test)]
    pub(in crate::app) connections_request_pending: bool,
    #[cfg(not(test))]
    connections_request_pending: bool,
}

impl ConnectionLifecycleState {
    pub(crate) fn active_connection_id(&self) -> Option<&str> {
        self.active_connection_id.as_deref()
    }

    pub(crate) fn set_active_connection_id(&mut self, connection_id: Option<String>) {
        self.active_connection_id = connection_id;
    }

    #[cfg(test)]
    pub(in crate::app) fn active_connection_id_mut(&mut self) -> &mut Option<String> {
        &mut self.active_connection_id
    }

    pub(crate) fn is_connected(&self) -> bool {
        self.connected
    }

    pub(crate) fn set_connected(&mut self, connected: bool) {
        self.connected = connected;
    }

    pub(crate) fn with_fallback_name(name: impl Into<String>) -> Self {
        Self {
            fallback_name: name.into(),
            ..Default::default()
        }
    }

    pub(crate) fn fallback_name(&self) -> &str {
        &self.fallback_name
    }

    pub(crate) fn connections_requested(&self) -> bool {
        self.connections_requested
    }

    pub(crate) fn mark_connections_requested(&mut self) {
        self.connections_requested = true;
    }

    pub(crate) fn clear_connections_requested(&mut self) {
        self.connections_requested = false;
    }

    pub(crate) fn connections_request_pending(&self) -> bool {
        self.connections_request_pending
    }

    pub(crate) fn set_connections_request_pending(&mut self, pending: bool) {
        self.connections_request_pending = pending;
    }

    pub(crate) fn clear_pending_request(&mut self) {
        self.pending_request = None;
        self.pending_operation = None;
    }

    pub(crate) fn pending_request(&self) -> Option<RequestId> {
        self.pending_request
    }

    pub(crate) fn set_pending_request(&mut self, request_id: Option<RequestId>) {
        self.pending_request = request_id;
    }

    pub(crate) fn pending_operation(&self) -> Option<PendingConnectionOperation> {
        self.pending_operation
    }

    pub(crate) fn set_pending_operation(&mut self, operation: Option<PendingConnectionOperation>) {
        self.pending_operation = operation;
    }

    pub(crate) fn pending_connection_id(&self) -> Option<&str> {
        self.pending_connection_id.as_deref()
    }

    pub(crate) fn set_pending_connection_id(&mut self, connection_id: Option<String>) {
        self.pending_connection_id = connection_id;
    }

    pub(crate) fn take_pending_connection_id(&mut self) -> Option<String> {
        self.pending_connection_id.take()
    }

    pub(crate) fn has_failed_connection(&self, connection_id: &str) -> bool {
        self.failed_connection_ids.contains(connection_id)
    }

    pub(crate) fn connection_error(&self, connection_id: &str) -> Option<&str> {
        self.errors.get(connection_id).map(String::as_str)
    }

    pub(crate) fn record_connection_failure(&mut self, connection_id: String, message: String) {
        self.failed_connection_ids.insert(connection_id.clone());
        self.errors.insert(connection_id, message);
    }

    pub(crate) fn clear_connection_error(&mut self, connection_id: &str) {
        self.errors.remove(connection_id);
        self.failed_connection_ids.remove(connection_id);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clearing_connection_error_removes_failure_marker_and_message() {
        let mut state = ConnectionLifecycleState {
            errors: [("conn-1".to_owned(), "failed".to_owned())].into_iter().collect(),
            failed_connection_ids: ["conn-1".to_owned()].into_iter().collect(),
            ..Default::default()
        };

        state.clear_connection_error("conn-1");

        assert!(state.errors.is_empty());
        assert!(state.failed_connection_ids.is_empty());
    }

    #[test]
    fn default_lifecycle_starts_disconnected_with_a_local_fallback_name() {
        let state = ConnectionLifecycleState {
            fallback_name: "Local PostgreSQL".to_owned(),
            ..Default::default()
        };

        assert!(!state.connected);
        assert_eq!(state.fallback_name, "Local PostgreSQL");
    }

}
