# Verification

## Automated evidence

- `cargo fmt --all -- --check` — PASS.
- `git diff --check` — PASS.
- `cargo check --workspace` — PASS.
- `cargo clippy --workspace --all-targets -- -D warnings` — PASS.
- `cargo test -p db-pro-core -p db-pro-infrastructure` — PASS: 192 core unit
  tests, 44 infrastructure unit tests, 26 SQLite integration tests, 10 PostgreSQL
  integration tests ignored.
- `cargo test -p db-pro-core backup_factory_receives_ssh_configuration` — PASS.
- `cargo test -p db-pro-core --lib application::connection_service::tests::disconnect_failure_keeps_handle_for_retry -- --exact` — PASS.
- `cargo test -p db-pro-infrastructure postgres::connector::tests::postgres_operation_timeout_returns_query_timeout -- --exact` — PASS.
- `cargo test -p db-pro-infrastructure postgres::user_manager::tests` — PASS: 3
  identifier/privilege validation tests.
- External PostgreSQL command timeout regression — PASS on Unix via
  `external_command_timeout_returns_query_timeout`.
- `cargo test --workspace` — PASS: all executed workspace tests passed; 10
  PostgreSQL integration tests remain ignored.
- Targeted regression `sqlite_query_timeout_interrupts_vm_and_actor_recovers` — PASS.
- `cargo build --release --locked -p db-pro-core -p db-pro-infrastructure` — PASS.
- `cargo check --workspace` after SSH readiness changes — PASS.
- Full current gate — PASS: `cargo fmt --all -- --check`, `cargo clippy --workspace
  --all-targets -- -D warnings`, `cargo test --workspace`, release build for
  `db-pro-core` and `db-pro-infrastructure`, and `git diff --check`.

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
| SQLite | Integration timeout/recovery PASS | Pending | Native UI/provider runtime evidence still pending |

## Scope check

- UI/native files: source diff contains no files under `crates/ui` or `crates/native-app`.
- Release build: PASS for `db-pro-core` and `db-pro-infrastructure`; the native
  application release gate is outside this core-only hardening slice.
