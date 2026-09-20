# Native Core Architecture — Verification

Source checkpoint: `623d6331`.

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
- `WorkspaceFilesState`, `DiagramState`, named database-management aggregates
  from `database_feature_states.rs`, `SchemaCompareState`,
  `PaletteState`, `QueryExecutionPolicyState`, `QueryLibraryState`,
  `SavedTaskState`, `WorkspaceSessionState`, `OverlayState`, `FeedbackState`,
  `PreferencesState` and `WelcomeState` now own their feature state.
- `ConnectionLifecycleState` now also owns connection status and fallback name;
  `SchemaExplorerState` owns persisted explorer pane heights.
- Connection dialog fields are private to the `connection` feature module, and
  saved-connection storage is private behind catalog read-model methods
  (`iter`, `get`, `find`, `len`, `is_empty`).
- Connection lifecycle request flags and fallback naming are private behind
  lifecycle methods; tests use explicit lifecycle setup APIs rather than
  production field access.
- Connection connected status is private behind `is_connected` and
  `set_connected` lifecycle APIs.
- Active connection identity is private behind lifecycle accessors; consumers
  no longer read or mutate the storage field directly.
- Pending request/target and connection failure storage are private behind
  lifecycle APIs; production consumers no longer access those storage fields
  directly.
- Connection dialog rendering is now driven by `ConnectionDialogView<'a>` with
  explicit state/runtime/feedback dependencies; `view.rs`, `form_fields.rs`
  and `advanced_panels.rs` no longer implement methods on `DbProApp`.
- Active connection, schema and statusbar helpers are pure functions in
  `connection_status.rs`; the module no longer implements methods on
  `DbProApp`, and the architecture guard enforces that boundary.
- Connection lifecycle event reducers are pure functions in
  `connection_events.rs`; the root wrapper only performs follow-up runtime
  orchestration after the reducer returns an explicit transition result.
- Connection deletion confirmation now lives in
  `connection/delete_dialog.rs` and receives explicit overlay/catalog/
  lifecycle/runtime/feedback dependencies; it no longer implements a
  `DbProApp` method.
- Query-folder deletion confirmation now lives in
  `query_folder_delete_dialog.rs`; the old mixed connection/folder confirmation
  module was removed.
- `DbProApp` now composes one `ConnectionFeatureState` aggregate containing
  catalog, lifecycle and dialog sub-states; the architecture allowlist rejects
  the former three root fields.
- `DbProApp` now composes one `WorkspaceFeatureState` aggregate containing
  shell/navigation, local-file activity and named-session sub-states; the
  architecture allowlist rejects the former `workspace_files` and
  `workspace_sessions` root fields. Existing shell field access is preserved
  through a typed `Deref` facade while file/session ownership remains explicit
  under `workspace.files` and `workspace.sessions`.
- Schema event handling now lives in explicit-state reducers in
  `schema_events.rs`; the root wrapper only performs the follow-up table-info
  request returned by `SchemaLoadedTransition`. Stale request rejection and
  missing-table reconciliation are covered by reducer tests, and the
  architecture guard rejects `DbProApp` references in the reducer module.
- Agent provider/workflow event handling now lives in explicit-state reducers
  in `agent_events.rs`; provider configuration failure is request-scoped and
  reducer tests cover stale configuration events and provider readiness. Toast
  emission is owned by `FeedbackState`, not an app-only helper, and the
  architecture guard rejects `DbProApp` references in the agent reducer.
- Table event handling now lives in explicit-state reducers in
  `table_events.rs`; metadata/data/row-reload transitions return typed cache
  invalidation and staged-apply effects, while the root only executes those
  follow-ups. Existing table mutation, reload and stale-request tests remain
  green, and the architecture guard rejects `DbProApp` references in the
  table reducer.
- Saved-query and query-folder read-model replacement now lives in
  `query_library_events.rs`; the root keeps only thin adapters for event
  routing, and reducer tests cover replacement semantics.
- SQL prediction ready/failed handling now lives in
  `query_prediction_events.rs`; stale request and document-version checks stay
  in the query-session reducer boundary, while the root only adapts runtime
  event payloads.
- Saved-query completion now lives in `query_save_events.rs`; the reducer
  returns an explicit close-document transition and the root performs only the
  resulting tab orchestration.
- Query-history retention now lives in `query_history_events.rs`; the reducer
  owns the 500-entry cap and only receives `QueryEditorState` plus a history
  record.
- Query queued feedback now lives in `query_queue_events.rs`; the reducer
  receives only `FeedbackState` and the request identity.
- Database-management event state transitions now live in
  `management_events.rs` for monitoring, audit, pg settings, FDW, replication,
  event triggers, security and data compare. `operation_events.rs` retains
  only composition-root orchestration and cross-feature follow-ups.
- File-picker state transitions now live in `file_picker_events.rs`, and DDL
  completion now returns a typed refresh transition from `ddl_events.rs`.
  Workspace opening and schema/RLS requests remain explicit root side effects.
- Explain completion and query cancellation now live in
  `query_execution_events.rs`, including output-tab selection, document
  cleanup and cancellation history.
- Single-statement query completion now lives in `query_result_events.rs`
  behind an explicit `QueryResultContext`, including grid invalidation,
  history, output selection and active-document presentation state.
- Multi-statement query completion now lives in
  `query_multi_result_events.rs` behind the same explicit state boundary;
  diagnostics, history status and result presentation remain request-scoped.
- Query-local failure handling now lives in `query_failure_events.rs`; the
  root only routes failures to other feature reducers before invoking the
  explicit query failure context. The old mixed `events_query.rs` module is
  deleted.
- Recent-table MRU ownership now lives on `SchemaExplorerState`; workspace,
  palette, explorer and tests use the state API instead of a root facade.
- Palette open lifecycle now lives on `PaletteState`; navigation, welcome,
  sidebar, query shortcuts and tests no longer call a `DbProApp` palette
  mutation facade.
- New-connection dialog opening now lives on `ConnectionFeatureState`; all
  shell entry points call the feature transition directly.
- Workspace close/refresh lifecycle now lives on `WorkspaceFilesState`; only
  the native folder-picker command remains in the composition root.
- Workspace search/replace, task, refactor, context, schema snapshot and drift
  transitions now live on `WorkspaceFilesState`; the root only composes the
  schema input needed by snapshot/drift operations.
- Workspace Git status/stage/unstage/diff/commit transitions and external-file
  change detection now live on `WorkspaceFilesState`; query documents are
  passed in as an explicit snapshot at the view boundary.
- Schema compare snapshot, diff and migration-plan transitions now live on
  `SchemaCompareState`; only migration apply remains root orchestration because
  it allocates a request and dispatches provider work.
- Transaction policy transitions now live on `QueryExecutionPolicyState` and
  return explicit SQL effects; the root only dispatches the returned effect.
- The capture-only native entrypoint now uses the feature-owned new-connection
  helper, so the capture-feature release build stays aligned with the dialog
  lifecycle migration.
- Named-session store mutations and persistence now live on
  `WorkspaceSessionState`; capture/restore of cross-feature layout remains
  explicit composition-root orchestration.
- Query output-tab override and active-tab mutations now live on
  `QueryOutputState`; the root only resolves the active document identity.
- Grid projection epoch and row-identity cache invalidation now live on
  `TableDataState`; query/result reducers and table event orchestration call
  that explicit state API.
- Query document collection invariants now live on `QuerySessionState`; add,
  select, remove, keep-one, truncate-right and reset operations no longer
  mutate the document vector and active index ad hoc in `DbProApp` helpers.
- Active query text, explain state, running request, result selection/count and
  message projections now live on `QuerySessionState`; the root keeps only
  cancellation, connection lookup and grid-invalidation orchestration.
- Query-document connection/schema metadata changes and prediction
  invalidation now live on `QuerySessionState`; the root only dispatches the
  returned prediction-cancel command.
- Result-grid row/cell selection and mutation-error matching now live on
  `TableDataState` and `TableMutationState`; grid cells/views no longer call
  selection helpers through `DbProApp`.
- Runtime event dispatch now lives in `crates/ui/src/event_router.rs`; feature
  transition handlers remain independently callable from the router.
- Agent and table event handlers now live in `agent_events.rs` and
  `table_events.rs`.
- Connection, schema and operation event handlers now live in their own
  feature event modules; `events.rs` contains only the event pump and tests.
- `event_router.rs` is now a pure event-to-handler dispatch table; database
  operation, agent, query and feature-failure transitions no longer mutate
  state inline in the router.
- Runtime event application is bounded to `64` events per egui frame; a full
  batch schedules another repaint. The native adapter uses a bounded
  `sync_channel(256)` and retries asynchronously when the UI queue is full.
- Runtime command sends are centralized through the dispatch adapter; closed
  command boundaries are logged and surfaced as a user-visible runtime error.
- Legacy `RunAgent`/`ExecuteAgentTool` commands and ignored tool completion
  events were removed; the runtime now exposes one agent workflow command/event
  contract.
- Aggregate fields are scoped to the app boundary; the architecture guard
  rejects crate-public state fields in feature state modules and connection
  state modules.
- `scripts/check-ui-architecture.sh`: PASS; it allowlists the composition-root
  fields, rejects event handlers in `events.rs`, rejects direct state access in
  `event_router.rs`, requires bounded event draining, and rejects feature code
  bypassing the command dispatch adapter.
- Shared dialog layout now reserves an explicit chrome budget, centers the card
  inside the safe viewport, gives the body its own scroll budget, and renders a
  full-width separated header with the close action aligned to the right.
- Deterministic native capture of the affected New Connection modal: PASS at
  logical `1280x800` (`/tmp/db-pro-evidence-core-error-latest.png`).
  The inspected framebuffer shows balanced vertical margins, a separated
  header, right-aligned close action, independently scrolling body and sticky
  footer.
- Deterministic state captures at logical `1280x800`: normal Welcome state
  (`/tmp/db-pro-evidence-core-normal-1280x800.png`), loading Welcome state
  (`/tmp/db-pro-evidence-core-loading-1280x800.png`) and New Connection error
  state (`/tmp/db-pro-evidence-core-error-1280x800.png`). The error alert is
  visible immediately below the separated header instead of being hidden at the
  end of the scroll body.
- Release runtime smoke: PASS; `target/release/db-pro-native` launched from
  the verified HEAD and rendered the Welcome/empty state in a `1440x870` DB Pro
  window. Capture was inspected from the native window after startup settled.
- Unit tests for the extracted aggregates are included in the UI test suite.
- `cargo check -p db-pro-ui`: PASS.
- `cargo fmt --all`: executed.
- `cargo clippy -p db-pro-ui --all-targets -- -D warnings`: PASS.
- `cargo test -p db-pro-ui --lib`: 596 passed, 0 failed.
- `cargo fmt --all -- --check`: PASS.
- `cargo check --workspace`: PASS.
- `cargo clippy --workspace --all-targets -- -D warnings`: PASS.
- `cargo test --workspace --no-fail-fast`: 1268 passed, 0 failed, 42 ignored.
- `cargo build --release --locked -p db-pro-native`: PASS.
- `cargo build --release --locked -p db-pro-native --features capture`: PASS.
- `bash .skills/clean-code/scripts/clean-code-scan.sh rust --diff --ratchet --ci`: 11 pass, 5 warnings, 0 failures; warnings are ratcheted size/cast/clone heuristics.

## Not yet proven

- Native screenshot/runtime evidence for the requested `1440x900` and
  `1920x1080` logical heights remains host-limited: macOS capture clamps both
  to a logical height of `838`. The required normal/loading/error/empty states
  are now captured at exact logical `1280x800`.
