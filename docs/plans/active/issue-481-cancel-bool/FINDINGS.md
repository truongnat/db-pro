# #481 findings

- `connector.rs:497-503` ran `sqlx::query("SELECT pg_cancel_backend($1)").execute(&pool)`
  inside a pid loop and discarded the row — `false` (permission/backend-exited) surfaced
  as `Ok(())`.
- `pg_cancel_backend` returns a single `bool` column; `query_scalar` + `fetch_one`
  exposes it directly. No rows-affected semantics needed.
- `DbError::Internal(String)` is the existing convention for infra-internal failures
  (e.g. `rollback_batch`); used for the aggregated refusal error.
- Refused pids collected across the loop so one error names all of them, matching the
  issue's "could not cancel backend(s) 123, 456" shape. A transport error still aborts
  early via `?`.
- `pids` snapshot comes from `entry.active_backends` — pids may already be stale by the
  time cancel runs, which is exactly why false results must be reported.
