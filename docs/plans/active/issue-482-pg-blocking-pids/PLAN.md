# Issue #482 — blocker detection must use `pg_blocking_pids`

## Problem
`list_locks` (`crates/infrastructure/src/postgres/monitoring.rs`) computed
`blocker_pid` with a LATERAL join on `pg_locks` matching only resource identity +
`granted` flag. It never checked lock-mode compatibility, ignored fast-path
locks and prepared-xact/group blockers, and `LIMIT 1` picked an arbitrary pid.
Result: compatible locks were reported as blockers and the UI could terminate an
innocent session.

## Fix
Replace the LATERAL subquery with `(pg_catalog.pg_blocking_pids(a.pid))[1]::bigint
AS blocker_pid` — PostgreSQL's authoritative blocker list (mode conflict,
fast-path, group aware). Row shape unchanged: one optional `blocker_pid` per
lock row (the view displays a single blocker per row).

## Files
- `crates/infrastructure/src/postgres/monitoring.rs` — extract `LIST_LOCKS_SQL`
  const; swap subquery for `pg_blocking_pids`; declare `mod tests`.
- `crates/infrastructure/src/postgres/tests/monitoring_tests.rs` — SQL contract
  tests (new file).
