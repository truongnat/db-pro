# Native Core Architecture — Checklist

## Foundation

- [x] Connection dialog aggregate introduced.
- [x] Connection dialog default/open/edit/duplicate/close transitions have unit tests.
- [x] Existing connection UI behavior compiles against the aggregate.
- [x] Connection lifecycle state separated from `DbProApp`.
- [x] Saved-connection read model separated from `DbProApp`.

## Remaining migrations

- [x] Workspace shell/navigation aggregate.
- [x] Query document/session aggregate.
- [x] Table/data interaction aggregate (grid/editor state).
- [x] Table metadata/request aggregate (introspection, paging, filters and table view state).
- [x] Table mutation/change-set effects aggregate.
- [x] Agent workspace aggregate.
- [x] Query editor/diagnostics/history aggregate.
- [x] Schema explorer aggregate.
- [ ] Saved-task scheduler and remaining task lifecycle aggregate.
- [ ] Runtime event dispatch split by feature.
- [ ] `DbProApp` reduced to composition root.
- [ ] Architecture boundary check in CI.

## Gates

- [x] `cargo fmt --all -- --check` (PASS)
- [x] `cargo check --workspace`
- [x] `cargo clippy -p db-pro-ui --all-targets -- -D warnings`
- [x] `cargo test -p db-pro-ui --lib` (562 passed)
- [x] `cargo test --workspace --no-fail-fast` (1230 passed, 0 failed, 42 ignored)
- [x] `cargo build --release --locked -p db-pro-native`
- [ ] Native runtime evidence for affected surfaces.
