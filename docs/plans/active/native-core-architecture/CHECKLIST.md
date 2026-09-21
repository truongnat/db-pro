# Native Core Architecture — Checklist

## Foundation

- [x] Connection dialog aggregate introduced.
- [x] Connection dialog default/open/edit/duplicate/close transitions have unit tests.
- [x] Existing connection UI behavior compiles against the aggregate.
- [x] Connection lifecycle state separated from `DbProApp`.
- [x] Saved-connection read model separated from `DbProApp`.
- [x] Connection dialog and catalog storage are private behind feature APIs.
- [x] Connection lifecycle request flags and fallback naming are private behind feature APIs.
- [x] Connection connected status is private behind lifecycle APIs.
- [x] Active connection identity is private behind lifecycle accessors.
- [x] Pending connection request/target and failure storage are private behind lifecycle APIs.
- [x] Connection dialog view/form/advanced panels use an explicit feature view context.
- [x] Connection status/schema helpers use explicit state inputs instead of `DbProApp` methods.
- [x] Connection lifecycle reducers use explicit state inputs instead of `DbProApp` methods.
- [x] Connection deletion confirmation uses an explicit feature context.
- [x] Query-folder deletion confirmation has its own feature module.
- [x] Connection catalog, lifecycle and dialog are composed under one feature aggregate at the root.

## Remaining migrations

- [x] Workspace shell/navigation aggregate.
- [x] Query document/session aggregate.
- [x] Table/data interaction aggregate (grid/editor state).
- [x] Table metadata/request aggregate (introspection, paging, filters and table view state).
- [x] Table mutation/change-set effects aggregate.
- [x] Agent workspace aggregate.
- [x] Agent workflow reducer, patch safety, result projection, settings,
      header, confirmation and quick-action intent boundaries.
- [x] Settings keybindings, diagnostics, general/session and editor intent
      boundaries.
- [x] Query editor/diagnostics/history aggregate.
- [x] Schema explorer aggregate.
- [x] Saved-task scheduler and task lifecycle aggregate.
- [x] Query library, named workspace session and overlay aggregates.
- [x] Preferences, welcome and shared feedback aggregates.
- [x] Diagram, workspace files, database operations, palette and query execution policy aggregates.
- [x] Query execution preparation owns destructive gating, parameter binding,
      history and document-running transitions behind an explicit context.
- [x] Query Explain capability validation, ANALYZE confirmation and request
      transitions use an explicit feature context.
- [x] Saved-query command preparation and document request tracking use an
      explicit feature context; filesystem-backed workspace saves remain at the
      filesystem boundary.
- [x] Schema Workbench mutation-request planning is owned by
      `SchemaWorkbenchState`, with the root limited to orchestration.
- [x] Closing a Table workspace uses the same feature reset transition as table
      navigation, including filters, sorts, caches and mutation dialogs.
- [x] Agent and Saved Task command paths use the central dispatch adapter; the
      architecture guard catches multiline direct channel sends.
- [x] Explorer schema-change, disconnect and schema-object navigation paths
      reset table state through the canonical feature transition.
- [x] Query status, table DDL, insert, metadata, relations, mutation-dialog,
      conflict and data-toolbar surfaces use feature-owned contexts/actions.
- [x] Database management catch-all split into named feature aggregates and schema comparison state.
- [x] Feature aggregate fields scoped to the app boundary with an architecture guard.
- [x] Runtime event dispatch table isolated from feature handlers.
- [x] Agent and table runtime handlers split into feature event modules.
- [x] Connection, schema and operation reducers split out of `events.rs`.
- [x] Legacy agent command/event path removed; agent runtime uses one workflow contract.
- [ ] `DbProApp` reduced to composition root (event pump and cross-feature orchestration only).
      Explorer connection/schema/table/schema-object rendering still has root
      adapters under active migration.
- [x] Architecture boundary check in CI (`scripts/check-ui-architecture.sh`).

## Gates

- [x] `cargo fmt --all -- --check` (PASS)
- [x] `cargo check --workspace`
- [x] `cargo clippy -p db-pro-ui --all-targets -- -D warnings`
- [x] UI test suite through the workspace gate (654 passed at `bdf2e363`)
- [x] `cargo test --workspace --no-fail-fast --quiet` (PASS at `bdf2e363`; 0 failed)
- [x] `cargo build --release --locked -p db-pro-native`
- [x] `cargo build --release --locked -p db-pro-native --features capture`
- [x] New Connection modal runtime capture at logical `1280x800` (centered
      card, separated header, right-aligned close, sticky footer).
- [x] Explorer intent-boundary slices for connection, database, schema, table
      and schema-object rows.
- [x] Loading Welcome and New Connection error state captures at logical
      `1280x800`.
- [x] Latest native release binary rebuilt on `5f6e7870`; runtime launch is
      recorded below, but no new screenshot was collected for this checkpoint.
- [ ] Native runtime evidence for affected surfaces.
