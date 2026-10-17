# Verification — #482

- `cargo check -p db-pro-infrastructure` — clean (with the rest of the touched
  workspace crates).
- `cargo test -p db-pro-infrastructure monitoring` — `monitoring_tests` pass.
- Not run against a live PostgreSQL: no runtime fixture available; correctness
  rests on `pg_blocking_pids` being the documented server-side answer
  (https://www.postgresql.org/docs/current/functions-info.html) plus the
  SQL-shape contract tests.
