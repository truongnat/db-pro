//! Tests for `../monitoring.rs` (moved out of the source file).
use super::*;

#[test]
fn lock_query_uses_pg_blocking_pids_for_blockers() {
    // pg_blocking_pids is PostgreSQL's own blocker answer — it honors lock-mode
    // conflicts, fast-path locks, and prepared-xact/group blockers, so a
    // compatible grant can never be reported as blocking (#482).
    assert!(LIST_LOCKS_SQL.contains("pg_catalog.pg_blocking_pids(a.pid)"));
    // The old join matched any granted lock on the same resource regardless of
    // mode compatibility — it must stay gone.
    assert!(!LIST_LOCKS_SQL.contains("blocking.granted"));
    // Row shape is unchanged for the view: one optional blocker pid per row.
    assert!(LIST_LOCKS_SQL.contains("AS blocker_pid"));
    assert!(LIST_LOCKS_SQL.contains("AS locked_pid"));
}

#[test]
fn cancel_and_terminate_guard_backend_start_before_signalling() {
    // The pid's backend_start must still match the snapshot the user confirmed;
    // otherwise a recycled pid would signal an innocent session (#501). The
    // check happens inside the same statement as the signal so there is no
    // check-then-act window.
    for sql in [CANCEL_BACKEND_SQL, TERMINATE_BACKEND_SQL] {
        assert!(sql.contains("a.pid = $1"), "{sql}");
        assert!(sql.contains("a.backend_start = $2::timestamptz"), "{sql}");
        assert!(sql.contains("pg_catalog.pg_stat_activity"), "{sql}");
    }
    assert!(CANCEL_BACKEND_SQL.contains("pg_cancel_backend(a.pid)"));
    assert!(TERMINATE_BACKEND_SQL.contains("pg_terminate_backend(a.pid)"));
}

#[test]
fn sessions_query_still_exposes_backend_start_for_pid_reuse_guard() {
    // The confirm dialog re-sends this value; dropping it would silently
    // re-open the recycled-pid hole (#501).
    assert!(LIST_SESSIONS_SQL.contains("backend_start::text"));
}
