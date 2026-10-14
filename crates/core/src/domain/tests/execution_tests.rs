//! Tests for `../execution.rs` (moved out of the source file).
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
fn finish_rejects_non_terminal_status() {
    let mut exec = new_execution();
    exec.start();

    assert!(!exec.finish(ExecutionStatus::Created));
    assert_eq!(exec.status, ExecutionStatus::Running);
    assert!(exec.finished_at.is_none());
}

#[test]
fn succeed_requires_running_state() {
    let mut created = new_execution();
    created.succeed(42, 3, 1);
    assert_eq!(created.status, ExecutionStatus::Created);
    assert_eq!(created.rows_returned, 0);

    let mut finished = new_execution();
    finished.start();
    assert!(finished.finish(ExecutionStatus::TimedOut));
    finished.succeed(42, 3, 1);
    assert_eq!(finished.status, ExecutionStatus::TimedOut);
    assert_eq!(finished.rows_returned, 0);
    assert_eq!(finished.rows_affected, 0);
    assert_eq!(finished.statement_count, 0);
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
