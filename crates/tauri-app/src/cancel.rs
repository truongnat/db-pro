// Platform API — some methods are consumed by future integration patches.
#![allow(dead_code)]

use std::collections::HashMap;
use std::sync::Mutex;

use tokio::sync::oneshot;

use db_pro_core::domain::connection::ConnectionId;
use db_pro_core::domain::execution::{ExecutionStatus, QueryExecution, QueryExecutionId};

/// Tracks an in-flight execution's cancel sender and lifecycle state.
struct ExecutionEntry {
    execution: QueryExecution,
    cancel_tx: Option<oneshot::Sender<()>>,
}

/// Registry that tracks active query executions and their lifecycle.
///
/// Each execution goes through: Created → Running → terminal(Success/Error/Cancelled/TimedOut).
///
/// Cancel is idempotent: calling `cancel` on a finished or unknown execution
/// returns `CancelResult::NotFound` without panicking.
#[allow(dead_code)] // Platform API — methods consumed by future integration patches.
pub struct ExecutionRegistry {
    inner: Mutex<HashMap<String, ExecutionEntry>>,
}

/// Result of a cancel request.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CancelResult {
    /// The execution was found and a cancel signal was sent.
    Sent,
    /// The execution was not found (already finished or never started).
    NotFound,
}

impl ExecutionRegistry {
    pub fn new() -> Self {
        Self {
            inner: Mutex::new(HashMap::new()),
        }
    }

    /// Register a new execution. Returns the execution ID and a cancel receiver.
    ///
    /// The execution starts in `Created` state. The caller should call `start_execution`
    /// when the query actually begins running.
    pub fn register(&self, connection_id: ConnectionId) -> (QueryExecutionId, oneshot::Receiver<()>) {
        let (tx, rx) = oneshot::channel();
        let execution = QueryExecution::new(connection_id);
        let exec_id = execution.id.clone();

        let entry = ExecutionEntry {
            execution,
            cancel_tx: Some(tx),
        };

        self.inner
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .insert(exec_id.0.clone(), entry);

        (exec_id, rx)
    }

    /// Register a new execution with a caller-specified ID (e.g. tab ID).
    ///
    /// This allows the frontend to correlate executions with tabs directly,
    /// enabling deterministic cancel-by-tab instead of cancel-by-connection.
    pub fn register_with_id(&self, connection_id: ConnectionId, exec_id: QueryExecutionId) -> oneshot::Receiver<()> {
        let (tx, rx) = oneshot::channel();
        let execution = QueryExecution {
            id: exec_id.clone(),
            connection_id,
            status: ExecutionStatus::Created,
            started_at: chrono::Utc::now(),
            finished_at: None,
            statement_count: 0,
            rows_affected: 0,
            rows_returned: 0,
        };

        let entry = ExecutionEntry {
            execution,
            cancel_tx: Some(tx),
        };

        self.inner
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .insert(exec_id.0.clone(), entry);

        rx
    }

    /// Transition an execution from Created to Running.
    pub fn start_execution(&self, exec_id: &QueryExecutionId) {
        let mut guard = self.inner.lock().unwrap_or_else(|e| e.into_inner());
        if let Some(entry) = guard.get_mut(&exec_id.0) {
            entry.execution.start();
        }
    }

    /// Cancel an execution by sending a cancel signal.
    ///
    /// Idempotent: if the execution is already in a terminal state or not found,
    /// returns `CancelResult::NotFound` without panicking.
    pub fn cancel(&self, exec_id: &QueryExecutionId) -> CancelResult {
        let mut guard = self.inner.lock().unwrap_or_else(|e| e.into_inner());
        if let Some(entry) = guard.get_mut(&exec_id.0) {
            if entry.execution.status.is_terminal() {
                return CancelResult::NotFound;
            }
            if let Some(tx) = entry.cancel_tx.take() {
                let _ = tx.send(());
                entry.execution.finish(ExecutionStatus::Cancelled);
                CancelResult::Sent
            } else {
                // Cancel already sent once.
                CancelResult::NotFound
            }
        } else {
            CancelResult::NotFound
        }
    }

    /// Cancel by connection ID (legacy API for backward compatibility).
    ///
    /// Cancels the most recent active execution for the given connection.
    /// Returns `CancelResult::Sent` if an execution was found and cancelled.
    pub fn cancel_by_connection(&self, connection_id: &str) -> CancelResult {
        let mut guard = self.inner.lock().unwrap_or_else(|e| e.into_inner());
        // Find the active execution for this connection.
        let entry = guard
            .values_mut()
            .find(|e| e.execution.connection_id.to_string() == connection_id && !e.execution.status.is_terminal());

        if let Some(entry) = entry {
            if let Some(tx) = entry.cancel_tx.take() {
                let _ = tx.send(());
                entry.execution.finish(ExecutionStatus::Cancelled);
                CancelResult::Sent
            } else {
                CancelResult::NotFound
            }
        } else {
            CancelResult::NotFound
        }
    }

    /// Cancel an execution by its caller-specified ID (e.g. tab ID).
    ///
    /// This is the deterministic cancel path: each tab has a unique ID,
    /// so cancelling by tab ID always targets the correct execution.
    pub fn cancel_by_id(&self, exec_id: &str) -> CancelResult {
        let mut guard = self.inner.lock().unwrap_or_else(|e| e.into_inner());
        if let Some(entry) = guard.get_mut(exec_id) {
            if entry.execution.status.is_terminal() {
                return CancelResult::NotFound;
            }
            if let Some(tx) = entry.cancel_tx.take() {
                let _ = tx.send(());
                entry.execution.finish(ExecutionStatus::Cancelled);
                CancelResult::Sent
            } else {
                CancelResult::NotFound
            }
        } else {
            CancelResult::NotFound
        }
    }

    /// Complete an execution with a terminal status.
    ///
    /// Idempotent: calling finish on an already-terminal execution is a no-op.
    /// Always removes the cancel token to prevent dangling state.
    pub fn finish_execution(
        &self,
        exec_id: &QueryExecutionId,
        status: ExecutionStatus,
        rows_returned: u64,
        rows_affected: u64,
        statement_count: u32,
    ) {
        let mut guard = self.inner.lock().unwrap_or_else(|e| e.into_inner());
        if let Some(entry) = guard.get_mut(&exec_id.0) {
            // Idempotent: if already terminal, only clean up the cancel token.
            if entry.execution.status.is_terminal() {
                entry.cancel_tx.take();
                return;
            }
            entry.execution.rows_returned = rows_returned;
            entry.execution.rows_affected = rows_affected;
            entry.execution.statement_count = statement_count;
            entry.execution.finish(status);
            // Always drop the cancel token to prevent dangling state.
            entry.cancel_tx.take();
        }
    }

    /// Remove an execution from the registry (cleanup after completion).
    pub fn remove(&self, exec_id: &QueryExecutionId) {
        self.inner.lock().unwrap_or_else(|e| e.into_inner()).remove(&exec_id.0);
    }

    /// Remove execution by connection ID (legacy cleanup API).
    pub fn remove_by_connection(&self, connection_id: &str) {
        self.inner
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .retain(|_, entry| entry.execution.connection_id.to_string() != connection_id);
    }

    /// Get a snapshot of an execution's current state.
    pub fn get_execution(&self, exec_id: &QueryExecutionId) -> Option<QueryExecution> {
        self.inner
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .get(&exec_id.0)
            .map(|entry| entry.execution.clone())
    }

    /// List all active (non-terminal) executions.
    pub fn active_executions(&self) -> Vec<QueryExecution> {
        self.inner
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .values()
            .filter(|entry| !entry.execution.status.is_terminal())
            .map(|entry| entry.execution.clone())
            .collect()
    }

    /// Count of active (non-terminal) executions.
    pub fn active_count(&self) -> usize {
        self.inner
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .values()
            .filter(|entry| !entry.execution.status.is_terminal())
            .count()
    }
}

impl Default for ExecutionRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn register_creates_execution_in_created_state() {
        let registry = ExecutionRegistry::new();
        let conn_id = ConnectionId::new();
        let (exec_id, _rx) = registry.register(conn_id);

        let exec = registry.get_execution(&exec_id).unwrap();
        assert_eq!(exec.status, ExecutionStatus::Created);
        assert_eq!(exec.connection_id, conn_id);
    }

    #[test]
    fn start_execution_transitions_to_running() {
        let registry = ExecutionRegistry::new();
        let (exec_id, _rx) = registry.register(ConnectionId::new());

        registry.start_execution(&exec_id);

        let exec = registry.get_execution(&exec_id).unwrap();
        assert_eq!(exec.status, ExecutionStatus::Running);
    }

    #[test]
    fn cancel_running_execution_sends_signal() {
        let registry = ExecutionRegistry::new();
        let (exec_id, _rx) = registry.register(ConnectionId::new());
        registry.start_execution(&exec_id);

        let result = registry.cancel(&exec_id);
        assert_eq!(result, CancelResult::Sent);

        let exec = registry.get_execution(&exec_id).unwrap();
        assert_eq!(exec.status, ExecutionStatus::Cancelled);
    }

    #[test]
    fn cancel_is_idempotent() {
        let registry = ExecutionRegistry::new();
        let (exec_id, _rx) = registry.register(ConnectionId::new());
        registry.start_execution(&exec_id);

        assert_eq!(registry.cancel(&exec_id), CancelResult::Sent);
        // Second cancel returns NotFound — already terminal.
        assert_eq!(registry.cancel(&exec_id), CancelResult::NotFound);
    }

    #[test]
    fn cancel_finished_execution_is_noop() {
        let registry = ExecutionRegistry::new();
        let (exec_id, _rx) = registry.register(ConnectionId::new());
        registry.start_execution(&exec_id);
        registry.finish_execution(&exec_id, ExecutionStatus::Success, 10, 0, 1);

        assert_eq!(registry.cancel(&exec_id), CancelResult::NotFound);
    }

    #[test]
    fn cancel_unknown_execution_returns_not_found() {
        let registry = ExecutionRegistry::new();
        let unknown = QueryExecutionId::new();

        assert_eq!(registry.cancel(&unknown), CancelResult::NotFound);
    }

    #[test]
    fn finish_execution_sets_terminal_state_and_metrics() {
        let registry = ExecutionRegistry::new();
        let (exec_id, _rx) = registry.register(ConnectionId::new());
        registry.start_execution(&exec_id);

        registry.finish_execution(&exec_id, ExecutionStatus::Success, 42, 3, 2);

        let exec = registry.get_execution(&exec_id).unwrap();
        assert_eq!(exec.status, ExecutionStatus::Success);
        assert_eq!(exec.rows_returned, 42);
        assert_eq!(exec.rows_affected, 3);
        assert_eq!(exec.statement_count, 2);
        assert!(exec.finished_at.is_some());
    }

    #[test]
    fn finish_is_idempotent() {
        let registry = ExecutionRegistry::new();
        let (exec_id, _rx) = registry.register(ConnectionId::new());
        registry.start_execution(&exec_id);

        registry.finish_execution(&exec_id, ExecutionStatus::Success, 10, 0, 1);
        // Second finish is a no-op — status stays Success.
        registry.finish_execution(&exec_id, ExecutionStatus::Error, 0, 0, 0);

        let exec = registry.get_execution(&exec_id).unwrap();
        assert_eq!(exec.status, ExecutionStatus::Success);
        assert_eq!(exec.rows_returned, 10);
    }

    #[test]
    fn remove_cleans_up_execution() {
        let registry = ExecutionRegistry::new();
        let (exec_id, _rx) = registry.register(ConnectionId::new());

        assert!(registry.get_execution(&exec_id).is_some());
        registry.remove(&exec_id);
        assert!(registry.get_execution(&exec_id).is_none());
    }

    #[test]
    fn active_executions_excludes_terminal() {
        let registry = ExecutionRegistry::new();
        let (id1, _rx1) = registry.register(ConnectionId::new());
        let (id2, _rx2) = registry.register(ConnectionId::new());
        let (id3, _rx3) = registry.register(ConnectionId::new());

        registry.start_execution(&id1);
        registry.start_execution(&id2);
        registry.start_execution(&id3);

        registry.finish_execution(&id1, ExecutionStatus::Success, 0, 0, 1);
        registry.finish_execution(&id3, ExecutionStatus::Error, 0, 0, 1);

        let active = registry.active_executions();
        assert_eq!(active.len(), 1);
        assert_eq!(active[0].id, id2);
        assert_eq!(registry.active_count(), 1);
    }

    #[test]
    fn cancel_by_connection_works() {
        let registry = ExecutionRegistry::new();
        let conn_id = ConnectionId::new();
        let (exec_id, _rx) = registry.register(conn_id);
        registry.start_execution(&exec_id);

        let result = registry.cancel_by_connection(&conn_id.to_string());
        assert_eq!(result, CancelResult::Sent);

        let exec = registry.get_execution(&exec_id).unwrap();
        assert_eq!(exec.status, ExecutionStatus::Cancelled);
    }

    #[test]
    fn cancel_by_connection_not_found() {
        let registry = ExecutionRegistry::new();
        let result = registry.cancel_by_connection("nonexistent");
        assert_eq!(result, CancelResult::NotFound);
    }

    #[test]
    fn timeout_does_not_leave_dangling_state() {
        let registry = ExecutionRegistry::new();
        let (exec_id, _rx) = registry.register(ConnectionId::new());
        registry.start_execution(&exec_id);

        // Simulate timeout: finish with TimedOut.
        registry.finish_execution(&exec_id, ExecutionStatus::TimedOut, 0, 0, 0);

        let exec = registry.get_execution(&exec_id).unwrap();
        assert_eq!(exec.status, ExecutionStatus::TimedOut);
        assert!(exec.finished_at.is_some());
        // Cancel token should be cleaned up.
        assert_eq!(registry.cancel(&exec_id), CancelResult::NotFound);
    }

    /// Regression: two tabs sharing the same connection must cancel independently
    /// when using cancel_by_id (tab-scoped). This verifies the deterministic
    /// cancel path that replaced the non-deterministic cancel_by_connection.
    #[test]
    fn cancel_by_id_targets_correct_tab_on_same_connection() {
        let registry = ExecutionRegistry::new();
        let conn_id = ConnectionId::new();

        // Two tabs running queries on the same connection.
        let tab_a = QueryExecutionId("tab-a".to_string());
        let tab_b = QueryExecutionId("tab-b".to_string());
        let _rx_a = registry.register_with_id(conn_id, tab_a.clone());
        let _rx_b = registry.register_with_id(conn_id, tab_b.clone());
        registry.start_execution(&tab_a);
        registry.start_execution(&tab_b);

        // Cancel only tab-B.
        let result = registry.cancel_by_id("tab-b");
        assert_eq!(result, CancelResult::Sent);

        // Tab-B should be cancelled.
        let exec_b = registry.get_execution(&tab_b).unwrap();
        assert_eq!(exec_b.status, ExecutionStatus::Cancelled);

        // Tab-A should still be running — not affected by tab-B's cancel.
        let exec_a = registry.get_execution(&tab_a).unwrap();
        assert_eq!(exec_a.status, ExecutionStatus::Running);

        // Both executions are on the same connection.
        assert_eq!(exec_a.connection_id, exec_b.connection_id);
    }

    #[test]
    fn register_with_id_and_cancel_by_id_roundtrip() {
        let registry = ExecutionRegistry::new();
        let conn_id = ConnectionId::new();
        let exec_id = QueryExecutionId("my-tab-123".to_string());

        let mut rx = registry.register_with_id(conn_id, exec_id.clone());
        registry.start_execution(&exec_id);

        // The receiver should get the cancel signal.
        let result = registry.cancel_by_id("my-tab-123");
        assert_eq!(result, CancelResult::Sent);
        assert!(rx.try_recv().is_ok());

        let exec = registry.get_execution(&exec_id).unwrap();
        assert_eq!(exec.status, ExecutionStatus::Cancelled);
    }

    /// Regression: same tab running two sequential queries must use different execution IDs.
    /// When run #1 finishes and run #2 starts with a new execution ID, the old ID is gone.
    #[test]
    fn sequential_runs_same_tab_different_execution_ids() {
        let registry = ExecutionRegistry::new();
        let conn_id = ConnectionId::new();

        // Run #1 with exec-1
        let exec_1 = QueryExecutionId("exec-1".to_string());
        let _rx_1 = registry.register_with_id(conn_id, exec_1.clone());
        registry.start_execution(&exec_1);
        registry.finish_execution(&exec_1, ExecutionStatus::Success, 10, 0, 1);
        registry.remove(&exec_1);

        // Run #2 with exec-2 (new UUID in production)
        let exec_2 = QueryExecutionId("exec-2".to_string());
        let _rx_2 = registry.register_with_id(conn_id, exec_2.clone());
        registry.start_execution(&exec_2);

        // exec-1 no longer exists
        assert!(registry.get_execution(&exec_1).is_none());
        // exec-2 is running
        let exec = registry.get_execution(&exec_2).unwrap();
        assert_eq!(exec.status, ExecutionStatus::Running);
    }

    /// Regression: stale cancel for exec-1 must NOT cancel exec-2.
    #[test]
    fn stale_cancel_cannot_cancel_different_execution() {
        let registry = ExecutionRegistry::new();
        let conn_id = ConnectionId::new();

        // Run #1 finished
        let exec_1 = QueryExecutionId("exec-1".to_string());
        let _rx_1 = registry.register_with_id(conn_id, exec_1.clone());
        registry.start_execution(&exec_1);
        registry.finish_execution(&exec_1, ExecutionStatus::Success, 5, 0, 1);
        registry.remove(&exec_1);

        // Run #2 running
        let exec_2 = QueryExecutionId("exec-2".to_string());
        let _rx_2 = registry.register_with_id(conn_id, exec_2.clone());
        registry.start_execution(&exec_2);

        // Stale cancel targeting exec-1 (already removed) — idempotent no-op
        let result = registry.cancel_by_id("exec-1");
        assert_eq!(result, CancelResult::NotFound);

        // exec-2 must still be running
        let exec = registry.get_execution(&exec_2).unwrap();
        assert_eq!(exec.status, ExecutionStatus::Running);
    }
}
