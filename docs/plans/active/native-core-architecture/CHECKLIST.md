# Native Core Architecture — Checklist

## Foundation

- [x] Connection dialog aggregate introduced.
- [x] Connection dialog default/open/edit/duplicate/close transitions have unit tests.
- [x] Existing connection UI behavior compiles against the aggregate.
- [x] Connection lifecycle state separated from `DbProApp`.
- [x] Saved-connection read model separated from `DbProApp`.
- [x] Connection dialog and catalog storage are private behind feature APIs.

## Remaining migrations

- [x] Workspace shell/navigation aggregate.
- [x] Query document/session aggregate.
- [x] Table/data interaction aggregate (grid/editor state).
- [x] Table metadata/request aggregate (introspection, paging, filters and table view state).
- [x] Table mutation/change-set effects aggregate.
- [x] Agent workspace aggregate.
- [x] Query editor/diagnostics/history aggregate.
- [x] Schema explorer aggregate.
- [x] Saved-task scheduler and task lifecycle aggregate.
- [x] Query library, named workspace session and overlay aggregates.
- [x] Preferences, welcome and shared feedback aggregates.
- [x] Diagram, workspace files, database operations, palette and query execution policy aggregates.
- [x] Database management catch-all split into named feature aggregates and schema comparison state.
- [x] Feature aggregate fields scoped to the app boundary with an architecture guard.
- [x] Runtime event dispatch table isolated from feature handlers.
- [x] Agent and table runtime handlers split into feature event modules.
- [x] Connection, schema and operation reducers split out of `events.rs`.
- [x] Legacy agent command/event path removed; agent runtime uses one workflow contract.
- [x] `DbProApp` reduced to composition root (event pump and cross-feature orchestration only).
- [x] Architecture boundary check in CI (`scripts/check-ui-architecture.sh`).

## Gates

- [x] `cargo fmt --all -- --check` (PASS)
- [x] `cargo check --workspace`
- [x] `cargo clippy -p db-pro-ui --all-targets -- -D warnings`
- [x] `cargo test -p db-pro-ui --lib` (578 passed)
- [x] `cargo test --workspace --no-fail-fast` (1250 passed, 0 failed, 42 ignored)
- [x] `cargo build --release --locked -p db-pro-native`
- [x] `cargo build --release --locked -p db-pro-native --features capture`
- [x] New Connection modal runtime capture at logical `1280x800` (centered
      card, separated header, right-aligned close, sticky footer).
- [x] Loading Welcome and New Connection error state captures at logical
      `1280x800`.
- [ ] Native runtime evidence for affected surfaces.
