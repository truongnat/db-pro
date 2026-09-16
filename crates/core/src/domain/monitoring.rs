//! Session / active-query monitoring model (#196).

use serde::{Deserialize, Serialize};

/// One backend / session row from the monitoring adapter.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MonitorSession {
    /// Provider backend identity (PostgreSQL `pid`). Required for cancel/terminate.
    pub backend_id: i64,
    pub database: Option<String>,
    pub username: Option<String>,
    pub application_name: Option<String>,
    pub client_addr: Option<String>,
    pub state: Option<String>,
    pub wait_event_type: Option<String>,
    pub wait_event: Option<String>,
    /// Query text (may be truncated by the adapter).
    pub query_text: Option<String>,
    /// Duration of the current query in milliseconds, when known.
    pub query_duration_ms: Option<u64>,
    pub backend_start: Option<String>,
    pub xact_start: Option<String>,
    pub query_start: Option<String>,
    /// True when this row is the monitoring connection itself.
    pub is_current: bool,
}

/// SQLite (and other local engines) expose file/pragma state instead of server sessions.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct LocalMonitorState {
    pub journal_mode: Option<String>,
    pub page_count: Option<u64>,
    pub page_size: Option<u64>,
    pub freelist_count: Option<u64>,
    pub file_size_bytes: Option<u64>,
    pub note: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MonitoringSnapshot {
    pub connection_id: String,
    pub driver: String,
    pub sessions: Vec<MonitorSession>,
    pub local: Option<LocalMonitorState>,
    pub fetched_at_ms: u64,
    pub message: String,
}

impl MonitoringSnapshot {
    pub fn active_queries(&self) -> Vec<&MonitorSession> {
        self.sessions
            .iter()
            .filter(|s| {
                let state = s.state.as_deref().unwrap_or("");
                !state.eq_ignore_ascii_case("idle") && s.query_text.as_ref().is_some_and(|q| !q.is_empty())
            })
            .collect()
    }
}
