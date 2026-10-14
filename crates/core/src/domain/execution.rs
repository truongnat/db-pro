use serde::{Deserialize, Serialize};

use super::connection::ConnectionId;

/// Unique identifier for a single query execution.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct QueryExecutionId(pub String);

impl QueryExecutionId {
    pub fn new() -> Self {
        Self(uuid::Uuid::new_v4().to_string())
    }
}

impl Default for QueryExecutionId {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for QueryExecutionId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

/// Lifecycle status of a query execution.
///
/// State machine:
/// ```text
/// Created → Running → Success
///                     → Error
///                     → Cancelled
///                     → TimedOut
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ExecutionStatus {
    Created,
    Running,
    Success,
    Error,
    Cancelled,
    TimedOut,
}

impl ExecutionStatus {
    /// Whether the execution is in a terminal state.
    pub fn is_terminal(self) -> bool {
        matches!(self, Self::Success | Self::Error | Self::Cancelled | Self::TimedOut)
    }
}

impl std::fmt::Display for ExecutionStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Created => write!(f, "created"),
            Self::Running => write!(f, "running"),
            Self::Success => write!(f, "success"),
            Self::Error => write!(f, "error"),
            Self::Cancelled => write!(f, "cancelled"),
            Self::TimedOut => write!(f, "timed_out"),
        }
    }
}

/// Tracks the full lifecycle of a single query execution.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryExecution {
    pub id: QueryExecutionId,
    pub connection_id: ConnectionId,
    pub status: ExecutionStatus,
    pub started_at: chrono::DateTime<chrono::Utc>,
    pub finished_at: Option<chrono::DateTime<chrono::Utc>>,
    pub statement_count: u32,
    pub rows_affected: u64,
    pub rows_returned: u64,
}

impl QueryExecution {
    /// Create a new execution in `Created` state.
    pub fn new(connection_id: ConnectionId) -> Self {
        let now = chrono::Utc::now();
        Self {
            id: QueryExecutionId::new(),
            connection_id,
            status: ExecutionStatus::Created,
            started_at: now,
            finished_at: None,
            statement_count: 0,
            rows_affected: 0,
            rows_returned: 0,
        }
    }

    /// Transition to `Running`.
    pub fn start(&mut self) {
        if self.status == ExecutionStatus::Created {
            self.status = ExecutionStatus::Running;
            self.started_at = chrono::Utc::now();
        }
    }

    /// Transition to a terminal status. Returns `false` if already terminal
    /// or if `status` is not terminal (both are no-ops).
    pub fn finish(&mut self, status: ExecutionStatus) -> bool {
        if self.status.is_terminal() || !status.is_terminal() {
            return false;
        }
        self.status = status;
        self.finished_at = Some(chrono::Utc::now());
        true
    }

    /// Mark as successful with result metrics.
    pub fn succeed(&mut self, rows_returned: u64, rows_affected: u64, statement_count: u32) {
        if self.status != ExecutionStatus::Running {
            return;
        }
        self.rows_returned = rows_returned;
        self.rows_affected = rows_affected;
        self.statement_count = statement_count;
        self.finish(ExecutionStatus::Success);
    }

    /// Duration of execution (uses `finished_at` if available, else now).
    pub fn duration_ms(&self) -> u64 {
        let end = self.finished_at.unwrap_or_else(chrono::Utc::now);
        (end - self.started_at).num_milliseconds().max(0) as u64
    }
}

#[cfg(test)]
#[path = "tests/execution_tests.rs"]
mod tests;
