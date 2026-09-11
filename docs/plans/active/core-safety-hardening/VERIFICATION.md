# Verification

## Automated evidence

- `cargo fmt --all -- --check` — PASS.
- `git diff --check` — PASS.
- `cargo check --workspace` — PASS.
- `cargo clippy --workspace --all-targets -- -D warnings` — PASS.
- `cargo test -p db-pro-core -p db-pro-infrastructure` — PASS: 190 core unit
  tests, 39 infrastructure unit tests, 26 SQLite integration tests, 10 PostgreSQL
  integration tests ignored.
- `cargo test -p db-pro-core backup_factory_receives_ssh_configuration` — PASS.
- `cargo test --workspace` — PASS: all executed workspace tests passed; 10
  PostgreSQL integration tests remain ignored.
- Targeted regression `sqlite_query_timeout_interrupts_vm_and_actor_recovers` — PASS.
- `cargo build --release --locked -p db-pro-core -p db-pro-infrastructure` — PASS.
- `cargo check --workspace` after SSH readiness changes — PASS.

Source-only security check:

- `rg -n "StrictHostKeyChecking" crates/infrastructure` — no matches.

The PostgreSQL backup test proves the core factory preserves `ssh_tunnel`; actual
SSH and PostgreSQL command execution remains provider/runtime evidence pending.

## Provider matrix

| Provider | Automated | Live provider | Notes |
|---|---|---|---|
| PostgreSQL | Unit policy coverage PASS | Pending | Live Explain/mutation behavior still needs a real PostgreSQL provider |
| SQLite | Integration timeout/recovery PASS | Pending | Native UI/provider runtime evidence still pending |

## Scope check

- UI/native files: source diff contains no files under `crates/ui` or `crates/native-app`.
- Release build: pending; it is outside this core-only hardening slice.
