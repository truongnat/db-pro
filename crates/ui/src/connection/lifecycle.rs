use crate::RequestId;
use std::collections::{HashMap, HashSet};

/// Connection lifecycle state owned by the connection feature.
///
/// The saved connection collection remains in `DbProApp` for the next
/// migration step because it is also used as the explorer's read model.
#[derive(Debug, Default)]
pub(crate) struct ConnectionLifecycleState {
    pub(crate) active_connection_id: Option<String>,
    pub(crate) pending_connection_id: Option<String>,
    pub(crate) pending_request: Option<RequestId>,
    pub(crate) errors: HashMap<String, String>,
    pub(crate) failed_connection_ids: HashSet<String>,
    pub(crate) connections_requested: bool,
    pub(crate) connections_request_pending: bool,
}

impl ConnectionLifecycleState {
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
}
