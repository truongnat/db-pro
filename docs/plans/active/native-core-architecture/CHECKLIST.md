# Native Core Architecture — Checklist

## Foundation

- [x] Connection dialog aggregate introduced.
- [x] Connection dialog default/open/edit/duplicate/close transitions have unit tests.
- [x] Existing connection UI behavior compiles against the aggregate.
- [ ] Connection lifecycle/session state separated from `DbProApp`.

## Remaining migrations

- [ ] Workspace shell/navigation aggregate.
- [ ] Query document/session aggregate.
- [ ] Table/data editor aggregate.
- [ ] Agent/task aggregate.
- [ ] Runtime event dispatch split by feature.
- [ ] `DbProApp` reduced to composition root.
- [ ] Architecture boundary check in CI.

## Gates

- [x] `cargo fmt --all -- --check` (PASS)
- [x] `cargo check --workspace`
- [x] `cargo clippy -p db-pro-ui --all-targets -- -D warnings`
- [x] `cargo test -p db-pro-ui --lib` (550 passed)
- [x] `cargo test --workspace --no-fail-fast` (1222 passed, 0 failed, 42 ignored)
- [x] `cargo build --release --locked -p db-pro-native`
- [ ] Native runtime evidence for affected surfaces.
