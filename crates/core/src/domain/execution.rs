use serde::{Deserialize, Serialize};

use super::connection::ConnectionId;

/// Unique identifier for a single query execution.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
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
    /// (idempotent — calling finish on a finished execution is a no-op).
    pub fn finish(&mut self, status: ExecutionStatus) -> bool {
        if self.status.is_terminal() {
            return false;
        }
        debug_assert!(
            status.is_terminal(),
            "finish() called with non-terminal status: {status}"
        );
        self.status = status;
        self.finished_at = Some(chrono::Utc::now());
        true
    }

    /// Mark as successful with result metrics.
    pub fn succeed(&mut self, rows_returned: u64, rows_affected: u64, statement_count: u32) {
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
mod tests {
    use super::*;

    fn new_execution() -> QueryExecution {
        QueryExecution::new(ConnectionId::new())
    }

    #[test]
    fn new_execution_starts_created() {
        let exec = new_execution();
        assert_eq!(exec.status, ExecutionStatus::Created);
        assert!(exec.finished_at.is_none());
    }

    #[test]
    fn start_transitions_to_running() {
        let mut exec = new_execution();
        exec.start();
        assert_eq!(exec.status, ExecutionStatus::Running);
    }

    #[test]
    fn finish_success_is_terminal() {
        let mut exec = new_execution();
        exec.start();
        assert!(exec.finish(ExecutionStatus::Success));
        assert_eq!(exec.status, ExecutionStatus::Success);
        assert!(exec.finished_at.is_some());
    }

    #[test]
    fn finish_error_is_terminal() {
        let mut exec = new_execution();
        exec.start();
        assert!(exec.finish(ExecutionStatus::Error));
        assert_eq!(exec.status, ExecutionStatus::Error);
    }

    #[test]
    fn finish_cancelled_is_terminal() {
        let mut exec = new_execution();
        exec.start();
        assert!(exec.finish(ExecutionStatus::Cancelled));
        assert_eq!(exec.status, ExecutionStatus::Cancelled);
    }

    #[test]
    fn finish_timed_out_is_terminal() {
        let mut exec = new_execution();
        exec.start();
        assert!(exec.finish(ExecutionStatus::TimedOut));
        assert_eq!(exec.status, ExecutionStatus::TimedOut);
    }

    #[test]
    fn finish_is_idempotent() {
        let mut exec = new_execution();
        exec.start();
        assert!(exec.finish(ExecutionStatus::Success));
        // Second call returns false — already terminal.
        assert!(!exec.finish(ExecutionStatus::Error));
        assert_eq!(exec.status, ExecutionStatus::Success);
    }

    #[test]
    fn succeed_sets_metrics() {
        let mut exec = new_execution();
        exec.start();
        exec.succeed(42, 3, 1);
        assert_eq!(exec.status, ExecutionStatus::Success);
        assert_eq!(exec.rows_returned, 42);
        assert_eq!(exec.rows_affected, 3);
        assert_eq!(exec.statement_count, 1);
    }

    #[test]
    fn is_terminal_variants() {
        assert!(!ExecutionStatus::Created.is_terminal());
        assert!(!ExecutionStatus::Running.is_terminal());
        assert!(ExecutionStatus::Success.is_terminal());
        assert!(ExecutionStatus::Error.is_terminal());
        assert!(ExecutionStatus::Cancelled.is_terminal());
        assert!(ExecutionStatus::TimedOut.is_terminal());
    }

    #[test]
    fn duration_ms_non_negative() {
        let exec = new_execution();
        // finished_at is None, uses now — should be ~0ms.
        assert!(exec.duration_ms() < 100);
    }

    #[test]
    fn execution_id_display() {
        let id = QueryExecutionId::new();
        assert!(!id.0.is_empty());
        assert_eq!(format!("{id}"), id.0);
    }
}
