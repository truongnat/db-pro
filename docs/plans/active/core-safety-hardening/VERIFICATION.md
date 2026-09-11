# Verification

## Automated evidence

- `cargo fmt --all -- --check` — PASS.
- `git diff --check` — PASS.
- `cargo check --workspace` — PASS.
- `cargo clippy --workspace --all-targets -- -D warnings` — PASS.
- `cargo test -p db-pro-core -p db-pro-infrastructure` — PASS: 203 core unit
  tests, 44 infrastructure unit tests, 27 SQLite integration tests, 10 PostgreSQL
  integration tests ignored.
- `cargo test -p db-pro-core backup_factory_receives_ssh_configuration` — PASS.
- `cargo test -p db-pro-core --lib application::connection_service::tests::disconnect_failure_keeps_handle_for_retry -- --exact` — PASS.
- `cargo test -p db-pro-infrastructure postgres::connector::tests::postgres_operation_timeout_returns_query_timeout -- --exact` — PASS.
- `cargo test -p db-pro-infrastructure postgres::connector::tests` — PASS: 2
  PostgreSQL SQL-builder/timeout tests; transaction rollback timeout behavior is
  covered in the implementation and remains live-provider pending.
- `cargo test -p db-pro-infrastructure postgres::user_manager::tests` — PASS: 3
  identifier/privilege validation tests.
- `cargo test -p db-pro-core application::connection_service::tests` — PASS: 22
  connection lifecycle/update/delete/connectivity/duplicate-cleanup tests.
- `cargo test -p db-pro-core domain::safety::tests` — PASS: 25 safety classifier
  and policy tests.
- `cargo test -p db-pro-core application::export_service::tests` — PASS: 6 export
  serialization, validation, and read-only policy tests.
- `cargo test -p db-pro-core execute_multi_routes_mutating_cte_through_transaction_while_preserving_rows` — PASS.
- `cargo test -p db-pro-core execute_multi_routes_select_then_update` — PASS.
- External PostgreSQL command timeout regression — PASS on Unix via
  `external_command_timeout_returns_query_timeout`.
- `cargo test --workspace` — PASS: 203 core unit, 44 infrastructure unit, 27
  SQLite integration, 10 PostgreSQL integration tests ignored, plus all runtime,
  native, UI, schema regression, and doc tests passed.
- Targeted regression `sqlite_query_timeout_interrupts_vm_and_actor_recovers` — PASS.
- Targeted regression `sqlite_transaction_timeout_waits_for_rollback_before_returning` — PASS
  against the in-memory SQLite provider; the post-timeout count was zero.
- `cargo build --release --locked -p db-pro-core -p db-pro-infrastructure` — PASS.
- `bash .skills/clean-code/scripts/clean-code-scan.sh --diff` — PASS: 9 pass,
  7 heuristic warning groups, 0 blocking failures. Remaining warnings are existing
  long modules, ignored SQLite actor send results, and unrelated native/UI helpers.
- `cargo check --workspace` after SSH readiness changes — PASS.
- Full current gate — PASS: `cargo fmt --all -- --check`, `cargo clippy --workspace
  --all-targets -- -D warnings`, `cargo test --workspace`, release build for
  `db-pro-core` and `db-pro-infrastructure`, and `git diff --check`.

The multi-statement regression proves that a row-producing data-modifying CTE is
sent through `execute_transaction` with query-result routing preserved. The live
rollback effect remains provider evidence pending.

Source-only security check:

- `rg -n "StrictHostKeyChecking" crates/infrastructure` — no matches.

The PostgreSQL backup test proves the core factory preserves `ssh_tunnel`; actual
SSH and PostgreSQL command execution remains provider/runtime evidence pending. The
PostgreSQL timeout wrapper is unit-tested; live timeout behavior still needs a real
provider.

## Provider matrix

| Provider | Automated | Live provider | Notes |
|---|---|---|---|
| PostgreSQL | Unit policy coverage PASS | Pending | Live Explain/mutation behavior still needs a real PostgreSQL provider |
| SQLite | Integration timeout/recovery PASS | PASS (in-memory provider) | Native UI runtime evidence is outside this core-only slice |

## Scope check

- UI/native files: source diff contains no files under `crates/ui` or `crates/native-app`.
- Release build: PASS for `db-pro-core` and `db-pro-infrastructure`; the native
  application release gate is outside this core-only hardening slice.
