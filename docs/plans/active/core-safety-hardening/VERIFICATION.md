# Verification

## Automated evidence

- `cargo fmt --all -- --check` — PASS.
- `git diff --check` — PASS.
- `cargo check --workspace` — PASS.
- `cargo clippy --workspace --all-targets -- -D warnings` — PASS.
- `cargo test -p db-pro-core backup_factory_receives_ssh_configuration` — PASS.
- `cargo test -p db-pro-core --lib application::connection_service::tests::disconnect_failure_keeps_handle_for_retry -- --exact` — PASS.
- `cargo test -p db-pro-infrastructure postgres::connector::tests::postgres_operation_timeout_returns_query_timeout -- --exact` — PASS.
- `cargo test -p db-pro-infrastructure postgres::connector::tests` — PASS: 2
  PostgreSQL SQL-builder/timeout tests; transaction and batch rollback are also
  covered by the live fixture below.
- `cargo test -p db-pro-infrastructure postgres::user_manager::tests` — PASS: 3
  identifier/privilege validation tests.
- `cargo test -p db-pro-core application::connection_service::tests` — PASS: 29
  connection lifecycle/update/delete/connectivity/duplicate-cleanup/cache-invalidation tests.
- `cargo test -p db-pro-core domain::safety::tests` — PASS: 25 safety classifier
  and policy tests.
- `cargo test -p db-pro-core application::export_service::tests` — PASS: 6 export
  serialization, validation, and read-only policy tests.
- `cargo test -p db-pro-core execute_multi_routes_mutating_cte_through_transaction_while_preserving_rows` — PASS.
- `cargo test -p db-pro-core execute_multi_routes_select_then_update` — PASS.
- `cargo test -p db-pro-infrastructure backup::pg_dump::tests` — PASS: 2 tests,
  including existing-destination rejection and external command timeout.
- `cargo test -p db-pro-infrastructure backup::sqlite_backup::tests` — PASS: 2
  tests, including no-overwrite publish behavior.
- `cargo test -p db-pro-core backup_uses_persisted_custom_secret_reference -- --nocapture` — PASS.
- `cargo test -p db-pro-core restore_uses_persisted_custom_secret_reference -- --nocapture` — PASS.
- `cargo test -p db-pro-core application::backup_service::tests` — PASS: 5 tests,
  including SQLite backup/restore without a database secret.
- `cargo test -p db-pro-infrastructure ssh::tunnel::tests -- --nocapture` — PASS: 2
  command-construction tests for key and password authentication modes.
- `cargo test -p db-pro-core --no-fail-fast` — PASS: 218 core unit tests,
  including SSH metadata redaction and connection lifecycle hydration coverage.
- `cargo test -p db-pro-core application::sql_policy::tests` — PASS: 9 lexical
  boundary and statement-splitting tests.
- `DATABASE_URL=postgres://dbpro:dbpro_test@127.0.0.1:15434/dbpro_fixture cargo test -p db-pro-infrastructure --test pg_integration --offline -- --ignored` — PASS: 14/14 against an isolated temporary `postgres:18.2` fixture; the container was removed after the run.
- External PostgreSQL command timeout regression — PASS on Unix via
  `external_command_timeout_returns_query_timeout`.
- `cargo test --workspace` — PASS: 218 core unit, 28 SQLite integration, 48
  infrastructure unit, 14 PostgreSQL integration tests ignored, plus all runtime,
  native, UI, schema regression, and doc tests passed.
- Targeted regression `sqlite_query_timeout_interrupts_vm_and_actor_recovers` — PASS.
- Targeted regression `sqlite_transaction_timeout_waits_for_rollback_before_returning` — PASS
  against the in-memory SQLite provider; the post-timeout count was zero.
- Targeted regression `sqlite_execute_batch_timeout_waits_for_rollback_before_returning` — PASS
  against the in-memory SQLite provider; the post-timeout count was zero and the actor recovered.
- `cargo build --release --locked -p db-pro-core -p db-pro-infrastructure` — PASS.
- `bash .skills/clean-code/scripts/clean-code-scan.sh --diff` — PASS: 9 pass,
  7 heuristic warning groups, 0 blocking failures. Remaining warnings are existing
  long modules, ignored SQLite actor send results, and unrelated native/UI helpers.
- `cargo check --workspace` after SSH readiness changes — PASS.
- Full current gate — PASS: `cargo fmt --all -- --check`, `cargo clippy --workspace
  --all-targets -- -D warnings`, `cargo test --workspace`, release build for
  `db-pro-core` and `db-pro-infrastructure`, and `git diff --check`.

The multi-statement regression proves that a row-producing data-modifying CTE is
sent through `execute_transaction` with query-result routing preserved. Live
PostgreSQL transaction failure, transaction timeout rollback, batch statement
failure rollback, and batch timeout rollback are covered by the isolated fixture
run above. SQLite has equivalent
batch and transaction timeout/rollback coverage against its in-memory provider.

Source-only security check:

- `rg -n "StrictHostKeyChecking" crates/infrastructure` — no matches.

The PostgreSQL backup test proves the core factory preserves `ssh_tunnel`; actual
SSH-tunnel backup execution remains pending. The live fixture proves PostgreSQL
introspection/query behavior plus transaction and batch rollback after statement
failure and timeout. The backup destination test proves an existing PostgreSQL
artifact is rejected before external command execution.

## Provider matrix

| Provider | Automated | Live provider | Notes |
|---|---|---|---|
| PostgreSQL | Unit policy/timeout coverage PASS | PASS (isolated `postgres:18.2`, 14/14) | Live introspection/query/transaction and batch rollback pass; SSH backup/tunnel execution pending |
| SQLite | Integration timeout/recovery PASS | PASS (in-memory provider) | Native UI runtime evidence is outside this core-only slice |

## Scope check

- UI/native files: source diff contains no files under `crates/ui` or `crates/native-app`.
- Release build: PASS for `db-pro-core` and `db-pro-infrastructure`; the native
  application release gate is outside this core-only hardening slice.
