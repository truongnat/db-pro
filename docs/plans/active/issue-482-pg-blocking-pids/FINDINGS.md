# Findings — #482

- Old query joined `pg_locks waiting` × `pg_locks blocking` on all resource
  columns + `waiting.granted = false` + `blocking.granted = true`. Two sessions
  holding/for-awaiting *compatible* modes on the same resource satisfy that
  predicate — false blockers.
- `waiting.granted = false` also misses fast-path locks (never appear ungranted
  in pg_locks) and prepared-transaction blockers (no pid in pg_locks at all).
- `pg_blocking_pids(pid)` (PG ≥ 9.6) returns the authoritative array, including
  group leaders for parallel queries.
- `MonitorLock.blocker_pid` is `Option<i64>` and every consumer renders a single
  blocker → `[1]` keeps the shape.
- Column order of `list_locks` SELECT unchanged: `locked_pid, blocker_pid,
  relation, lock_type, mode, granted, waiting_ms` — row mapping untouched.
