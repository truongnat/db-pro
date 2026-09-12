# Verification: Table Details Workspace

## Quality Gates
- [x] `cargo fmt --all -- --check`
- [x] `cargo check --workspace` (offline)
- [x] `cargo clippy --workspace --all-targets -- -D warnings` (offline)
- [x] `cargo test --workspace` (273 core, 62 infrastructure, 32 SQLite integration, 122 UI, 3 native, 4 runtime, 21 tauri tests; 18 PostgreSQL integration tests remain ignored without a live target)
- [x] `cargo build --release --locked -p db-pro-native` (release build)
- [x] Performance scan: native binary 20.9 MB, workspace check/clippy pass
- [ ] Runtime verification of the affected Table Data grid at 1280x800, 1440x900, and 1920x1080, including loading/error/empty states

## Current Slice Evidence

- Selection model tests cover filtered/sorted Shift range selection and non-empty Cmd/Ctrl toggle behavior.
- Focused UI tests cover rectangular cell selection and selecting the complete current grid; the grid also has TSV-with-header, JSON, and INSERT SQL copy actions.
- Native translation test covers comparison and `IS NULL` filter mapping.
- A release binary smoke launch completed schema introspection against the configured local PostgreSQL connection. Interactive grid gestures were not collected because the native app was not accessible to the scripted click harness in this run.
- ChangeSet tests cover final-value merge, update-to-delete supersession, no-op reversion, and cell reversion.
- Core table mutation tests cover parameterized statement construction/order and propagation of a confirmed rollback failure.
- Detailed transaction failures retain `statement_index` through core, runtime, and native UI events; the grid highlights the mapped row/cell, preserves the ChangeSet after rollback, and exposes rollback/error details with reload/discard/keep-local actions.
- Tables without a primary key now show a safety warning and disable row update/delete while leaving inserts available; row identity continues to use original server-side composite PK values.
- The provider transaction path is implemented for PostgreSQL and SQLite with rollback on statement failure, timeout, and affected-rows zero; live PostgreSQL execution remains pending.
- SQLite integration test `sqlite_parameterized_table_mutations_roll_back_and_reject_zero_rows` passes, covering actual rollback and zero-row failure handling.
- The grid now builds a visible row/column coordinate lookup once per render, preserving O(1) selection checks per cell.
