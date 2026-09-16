//! Session / active-query / admin monitoring model (#196 / #197).

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
    /// Age of the open transaction in milliseconds (PostgreSQL).
    pub xact_age_ms: Option<u64>,
    /// Age of the backend session in milliseconds (PostgreSQL).
    pub backend_age_ms: Option<u64>,
    /// True when `state` is idle in transaction (distinct from active).
    pub idle_in_transaction: bool,
    /// True when this row is the monitoring connection itself.
    pub is_current: bool,
}

/// Lock row with optional blocker identity (#197).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MonitorLock {
    pub locked_pid: i64,
    pub blocker_pid: Option<i64>,
    pub relation: Option<String>,
    pub lock_type: Option<String>,
    pub mode: Option<String>,
    pub granted: bool,
    pub waiting_ms: Option<u64>,
}

/// Relation size + scan counters. Sizes are exact byte counts (no float).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RelationSizeStat {
    pub schema: String,
    pub name: String,
    pub kind: String,
    pub total_bytes: u64,
    pub table_bytes: u64,
    pub index_bytes: u64,
    pub seq_scan: Option<u64>,
    pub idx_scan: Option<u64>,
    pub n_live_tup: Option<u64>,
    pub n_dead_tup: Option<u64>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct ServerSummary {
    pub version: Option<String>,
    pub current_database: Option<String>,
    pub max_connections: Option<u64>,
    pub current_connections: Option<u64>,
    pub database_size_bytes: Option<u64>,
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MaintenanceAction {
    Vacuum,
    Analyze,
    VacuumAnalyze,
}

impl MaintenanceAction {
    pub fn as_label(self) -> &'static str {
        match self {
            Self::Vacuum => "VACUUM",
            Self::Analyze => "ANALYZE",
            Self::VacuumAnalyze => "VACUUM ANALYZE",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MonitoringSnapshot {
    pub connection_id: String,
    pub driver: String,
    pub sessions: Vec<MonitorSession>,
    pub locks: Vec<MonitorLock>,
    pub relation_sizes: Vec<RelationSizeStat>,
    pub server: Option<ServerSummary>,
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
                !state.eq_ignore_ascii_case("idle")
                    && !s.idle_in_transaction
                    && s.query_text.as_ref().is_some_and(|q| !q.is_empty())
            })
            .collect()
    }

    pub fn idle_in_transaction_sessions(&self) -> Vec<&MonitorSession> {
        self.sessions.iter().filter(|s| s.idle_in_transaction).collect()
    }

    pub fn blocking_locks(&self) -> Vec<&MonitorLock> {
        self.locks
            .iter()
            .filter(|l| !l.granted || l.blocker_pid.is_some())
            .collect()
    }
}

/// Format byte counts without precision loss (exact integer, human units).
pub fn format_bytes_exact(bytes: u64) -> String {
    const KIB: u64 = 1024;
    const MIB: u64 = 1024 * KIB;
    const GIB: u64 = 1024 * MIB;
    if bytes >= GIB {
        format!("{}.{:02} GiB ({bytes} B)", bytes / GIB, (bytes % GIB) * 100 / GIB)
    } else if bytes >= MIB {
        format!("{}.{:02} MiB ({bytes} B)", bytes / MIB, (bytes % MIB) * 100 / MIB)
    } else if bytes >= KIB {
        format!("{}.{:02} KiB ({bytes} B)", bytes / KIB, (bytes % KIB) * 100 / KIB)
    } else {
        format!("{bytes} B")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn format_bytes_keeps_exact_byte_count() {
        assert_eq!(format_bytes_exact(1536), "1.50 KiB (1536 B)");
        assert!(format_bytes_exact(u64::MAX).contains(&u64::MAX.to_string()));
    }
}
