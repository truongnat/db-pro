# #481 — `cancel` ignores `pg_cancel_backend` boolean

## Goal
`PostgresConnector::cancel` must fail when `pg_cancel_backend($1)` returns `false`
(backend already gone or role lacks `pg_signal_backend`). Today `.execute()` hides the
result, so the UI reports Stop succeeded while the statement keeps running.

## Scope
- `crates/infrastructure/src/postgres/connector.rs` `cancel` (~L497): switch to
  `sqlx::query_scalar::<_, bool>` + `fetch_one`; collect refused pids; return one
  `DbError::Internal("could not cancel backend(s) 123, 456")`. Empty refused list → `Ok(())`.
- New pure helper `cancel_failures(refused: Vec<i32>) -> Result<(), DbError>` for testability.
- Tests in `crates/infrastructure/src/postgres/tests/connector_tests.rs` (per-repo `#[path]` split).

## Non-goals
- No retry/force-terminate escalation; `pg_terminate_backend` stays out of scope.
- No live-PG test; error aggregation is covered by the extracted pure fn plus a
  source-shape assertion on the shipped `cancel` body.

## Acceptance
- cancel reports failure naming every refused pid; `Ok(())` when none refused / none active.
