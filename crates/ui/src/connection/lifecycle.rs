use crate::RequestId;
use std::collections::{HashMap, HashSet};

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
    pub(in crate::app) active_connection_id: Option<String>,
    pub(in crate::app) pending_connection_id: Option<String>,
    pub(in crate::app) pending_request: Option<RequestId>,
    pub(in crate::app) errors: HashMap<String, String>,
    pub(in crate::app) failed_connection_ids: HashSet<String>,
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
