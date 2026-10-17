# #481 verification

- `cancel_reports_backends_pg_cancel_backend_refused` — `cancel_failures(vec![123,456])`
  errors and the message names both pids and "cancel".
- `cancel_with_no_refused_backends_succeeds` — empty refused list → `Ok(())`.
- `owned_cancel_reads_pg_cancel_backend_result` — shipped `cancel` body contains
  `query_scalar("SELECT pg_cancel_backend($1)")` → `fetch_one(&pool)` →
  `cancel_failures(refused)`, and no `.execute(&pool)`.
- `cargo check -p db-pro-infrastructure` — clean.
- `cargo test -p db-pro-infrastructure --lib` — 167 passed / 0 failed, including the three
  tests above.
