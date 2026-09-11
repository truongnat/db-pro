# Verification

## Automated evidence

- `cargo fmt --all -- --check` — PASS.
- `git diff --check` — PASS.
- `cargo check --workspace` — PASS.
- `cargo clippy --workspace --all-targets -- -D warnings` — PASS.
- `cargo test -p db-pro-core -p db-pro-infrastructure` — PASS: 189 core unit
  tests, 39 infrastructure unit tests, 25 SQLite integration tests, 10 PostgreSQL
  integration tests ignored.
- `cargo test --workspace` — PASS: all executed workspace tests passed; 10
  PostgreSQL integration tests remain ignored.
- Targeted regression `sqlite_query_timeout_interrupts_vm_and_actor_recovers` — PASS.

Source-only security check:

- `rg -n "StrictHostKeyChecking" crates/infrastructure` — no matches.

## Provider matrix

| Provider | Automated | Live provider | Notes |
|---|---|---|---|
| PostgreSQL | Unit policy coverage PASS | Pending | Live Explain/mutation behavior still needs a real PostgreSQL provider |
| SQLite | Integration timeout/recovery PASS | Pending | Native UI/provider runtime evidence still pending |

## Scope check

- UI/native files: source diff contains no files under `crates/ui` or `crates/native-app`.
- Release build: pending; it is outside this core-only hardening slice.
