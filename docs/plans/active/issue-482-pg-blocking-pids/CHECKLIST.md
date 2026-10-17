# Checklist — #482

- [x] `LIST_LOCKS_SQL` uses `pg_blocking_pids(a.pid)`, resource-join subquery removed
- [x] Row shape/`blocker_pid` Option semantics preserved for the view
- [x] Contract test asserts `pg_blocking_pids` present, `blocking.granted` gone
- [x] `cargo check -p db-pro-infrastructure`
- [x] `cargo test -p db-pro-infrastructure monitoring`
