# Native Core Architecture — Verification

Source checkpoint: `b881f1f4`.

## Current change

- Connection dialog state aggregate added under `crates/ui/src/connection/state.rs`.
- Connection dialog view, form, advanced panels, events and workspace actions
  now address the aggregate instead of individual `DbProApp` fields.
- Connection lifecycle state now owns active/pending/error/request state.
- `ConnectionCatalogState` now owns the saved-connection read model and its
  replacement/lookup operations.
- `WorkspaceShellState` now owns shell navigation, panel visibility/geometry,
  welcome lifecycle and pending navigation state; panel resize values are
  clamped through state setters.
- `QuerySessionState` now owns query documents, active selection, selected text,
  save/close request tracking and Save As lifecycle.
- `QueryOutputState` now owns the active output tab and per-document output-tab
  overrides.
- `TableDataState` now owns grid projection/layout, filtering/sorting,
  selection, cell editor, inspector and insert-row interaction state.
- `TableState` now owns table metadata, table view, introspection/DDL/data
  requests, paging, filters, metadata searches, details and row reload state.
- `TableMutationState` now owns staged changes, mutation requests, retries and
  conflict/apply state.
- `AgentState` now owns provider settings, composer input and agent sessions;
  the saved-task scheduler remains in `DbProApp`.
- `SchemaExplorerState` now owns schema loading, selection, navigation cache,
  pinned/recent tables and schema-object view state.
- `QueryEditorState` now owns editor overlays, visual-builder drafts,
  diagnostics caches, problem filters and query history.
- `WorkspaceFilesState`, `DiagramState`, `DatabaseOperationsState`,
  `PaletteState`, `QueryExecutionPolicyState`, `QueryLibraryState`,
  `SavedTaskState`, `WorkspaceSessionState`, `OverlayState`, `FeedbackState`,
  `PreferencesState` and `WelcomeState` now own their feature state.
- `ConnectionLifecycleState` now also owns connection status and fallback name;
  `SchemaExplorerState` owns persisted explorer pane heights.
- Runtime event dispatch now lives in `crates/ui/src/event_router.rs`; feature
  transition handlers remain independently callable from the router.
- Agent and table event handlers now live in `agent_events.rs` and
  `table_events.rs`; the legacy module retains connection/schema/operation
  reducers for the next split.
- Unit tests for the extracted aggregates: 27 passed, 0 failed.
- `cargo check -p db-pro-ui`: PASS.
- `cargo fmt --all`: executed.
- `cargo clippy -p db-pro-ui --all-targets -- -D warnings`: PASS.
- `cargo test -p db-pro-ui --lib`: 573 passed, 0 failed.
- `cargo fmt --all -- --check`: PASS.
- `cargo check --workspace`: PASS.
- `cargo clippy --workspace --all-targets -- -D warnings`: PASS.
- `cargo test --workspace --no-fail-fast`: 1247 passed, 0 failed, 42 ignored.
- `cargo build --release --locked -p db-pro-native`: PASS.
- `bash .skills/clean-code/scripts/clean-code-scan.sh rust --diff --ratchet --ci`: 11 pass, 5 warnings, 0 failures; warnings are ratcheted size/cast/clone heuristics.

## Not yet proven

- Native screenshot/runtime evidence for all affected states.
- Feature-level event reducers still share the legacy handler module and are
  the next architectural slice.
- Native screenshot/runtime evidence for all affected states.
