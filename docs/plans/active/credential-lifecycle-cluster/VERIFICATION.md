# Verification — Credential Lifecycle Cluster

Baseline: `beaa1882a6253d902748c27fba53b99d9969b635` (`origin/main` before implementation).

## Executed

- `cargo test -p db-pro-infrastructure failed_write_removes_partial_pgpass_file` — PASS, 1 passed / 0 failed.
- `cargo test -p db-pro-infrastructure backup::pg_dump::tests` — PASS, 8 passed / 0 failed.
- `cargo check --workspace` — PASS.
- `cargo +nightly fmt --all -- --check` — PASS.
- `git diff --check` — PASS.
- `cargo clippy -p db-pro-infrastructure --all-targets -- -D warnings` — BLOCKED by 3 existing `clippy::collapsible_match` diagnostics in untouched `crates/core/src/domain/saved_task.rs:153,158,165`. No diagnostics point to this change.

The first focused test invocation during development emitted an unused-import warning from the test; that import was removed. The final focused test invocation above passed without that warning.

## Not run

- `cargo test --workspace` — not run; focused crate tests cover the changed behavior.
- Live PostgreSQL/SQLite runtime — not applicable; this change only guards the lifecycle of the temporary PostgreSQL `PGPASSFILE` and changes no provider behavior.

## Manual regression shape

The regression writer writes a prefix to a real temporary file and then returns an I/O error. The test verifies the helper returns an error and the path no longer exists. The writer is dropped before the guard attempts removal, which also supports platforms that prohibit deleting an open file.
