# Terminate Session Identity — Checklist

## Planning
- [x] Issue evidence and PID-reuse failure recorded
- [x] Scope and non-goals explicit (#481 and #456 excluded)
- [x] PostgreSQL/SQLite behavior accounted for

## Implementation
- [x] Snapshot identity is carried through confirmation and runtime command
- [x] Application service rejects missing identity for cancel and terminate
- [x] PostgreSQL cancel/termination match PID and `backend_start`
- [x] Lock blocker lookup uses PostgreSQL `pg_blocking_pids()` rather than resource-only joins
- [x] SQLite adapter signature remains compatible and still reports unsupported

## Tests and review
- [x] Application tests cover missing, forwarded, and stale identity behavior
- [x] PostgreSQL adapter tests cover cancel/termination identity predicates/no-row result and `pg_blocking_pids()` lookup
- [x] Focused test/check results independently verified and recorded
- [x] Final diff and call-site audit complete

## Delivery
- [ ] Rebased on latest `origin/main`
- [ ] Committed, pushed, and draft PR linked with `Fixes #501` and `Fixes #482`
