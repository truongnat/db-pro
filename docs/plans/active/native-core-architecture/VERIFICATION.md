# Native Core Architecture — Verification

Source checkpoint: `0070d374`.

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
- Query-document lifecycle transitions now run through an explicit
  `QueryDocumentContext`; opening, duplication, closing and fallback-tab
  behavior receive their feature aggregates directly, while query execution
  remains a composition-root command decision.
- Query-document switching now uses the same context, so active cursor and
  selected-text synchronization is kept beside the document transition.
- Active query text edits, document connection/schema binding and prediction
  cancellation now use the same context; the root retains only cross-aggregate
  result-grid invalidation and command-level orchestration.
- Query connection/schema/capability resolution now uses a read-only
  `QueryConnectionContext` over the query session, connection catalog/lifecycle
  and schema explorer instead of embedding the lookup algorithm in the root.
- Table-editor value generation and typed parsing now live in the pure
  `table_editor_values.rs` module; UUID, numeric/decimal, JSON, temporal and
  binary validation no longer depends on `DbProApp`.
- Table mutation capability checks, staged-value lookup/revert and discard
  transitions now use `TableMutationContext`; the root keeps only reload and
  runtime-command orchestration.
- Staged-change transaction planning and retry-target filtering now live in
  `TableMutationState::build_apply_plan`; `apply_staged_changes` only performs
  boundary validation, command dispatch and request lifecycle updates.
- Primary-key row-reload filter construction now lives in
  `TableMutationState::row_reload_filters`, with composite-key metadata
  coverage in the state tests.
- Insert and duplicate-row mapping now live in pure functions in
  `table_editor_values.rs`; identity/generated-column handling, required-field
  validation and typed parsing are covered by focused tests.
- Synthetic-data plan construction now lives in `synthetic_data.rs`; table
  lookup, numeric input validation, inferred generators and bounded FK seed
  pools are covered by focused tests. The native capture adapter also rejects
  framebuffer dimensions that cannot be represented by PNG dimensions instead
  of truncating them.
- Masking preview construction now lives in `masking.rs`; requested-column
  parsing, stable fallback headers, sample rows and mask-rule output are covered
  by focused tests.
- PostgreSQL RLS/table-policy preview planning now lives in `security_rls.rs`
  with a shared quote dialect and explicit request structs; missing identity,
  role parsing and generated SQL are covered by focused tests.
- Schema compare keyed data-diff request validation and effect construction now
  live in `SchemaCompareState`; tests cover required target/table/key fields and
  normalized schema/key payloads.
- Monitoring state and snapshot/workload/session-control command planning now
  live in `monitoring_state.rs`; tests cover bounded workload requests and
  explicit confirmation flags for destructive commands.
- The architecture guard now freezes `monitoring_state.rs` as an explicit-state
  module that may not depend on the composition-root type.
- Audit filter construction and selected/bookmarked export planning now live in
  `audit_state.rs`; tests cover bounded load effects and export preconditions.
- The architecture guard now freezes `audit_state.rs` as an explicit-state
  module that may not depend on the composition-root type.
- The former `database_feature_states.rs` catch-all was removed; each remaining
  database-management aggregate now has an explicit state module and the guard
  checks those modules for composition-root dependencies.
- PostgreSQL settings, FDW, logical replication and event-trigger command
  payload construction now lives in the owning state modules; focused FDW
  coverage checks copied form values and explicit confirmation.
- Security role, membership, privilege and RLS-inspection command construction
  now lives in `SecurityState`; focused coverage checks the RLS boundary's
  required schema/table invariant.
- RLS preview application now also builds its `ExecuteDdl` effect in
  `SecurityState`, with coverage proving empty preview SQL cannot dispatch.
- Saved-query and query-folder refresh effects now build in `QueryLibraryState`,
  with focused coverage for both command identities.
- Table metadata and DDL effects now build in `TableState`; table-data request
  effects build in `TableDataQueryState`, keeping paging/filter/sort and row
  reload lifecycle out of metadata state. Focused coverage verifies empty DDL
  is rejected at the state boundary.
- Migration apply and schema-workbench DDL effects now build in their owning
  aggregates; focused coverage verifies both apply paths reject missing plans.
- Query-folder creation and saved-query save/rename/delete effects now build in
  `QueryLibraryState`; focused coverage checks folder normalization and the
  empty-folder precondition.
- Backup/restore and file-picker effects now build in `OverlayState`; focused
  coverage checks both path-bearing effects and their connection identity.
- Palette and explorer connection switching now use the lifecycle-owned
  `Connect` effect builder; focused coverage checks request and connection
  identity preservation.
- The architecture guard now freezes `table_editor_context.rs` and
  `table_editor_values.rs` as explicit-state modules that may not depend on
  the composition-root type.
- `VisualQueryBuilderState` now owns visual-builder form inputs, the
  `VisualQueryModel`, validation errors, SQL preview and state transitions for
  table/join/column/filter/order/import operations; the view retains only egui
  rendering plus active-document and feedback adapters.
- The architecture guard also freezes `visual_query_builder_state.rs` as an
  explicit-state module that may not depend on the composition-root type.
- `QueryOutputState` now owns the active output tab and per-document output-tab
  overrides.
- `TableDataState` now owns grid projection/layout, filtering/sorting,
  selection, cell editor, inspector and insert-row interaction state.
- `TableState` now owns table metadata, table view, introspection/DDL requests,
  metadata searches and details. `TableDataQueryState` owns paged data,
  filters, sorts, data requests and row reload state.
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
- Agent workflow event routing now also lives in `agent_events.rs`; document,
  session and run identity checks are performed against `AgentState` there,
  while the `DbProApp` method is only a thin composition-root adapter.
- Agent document snapshots and UI-to-core context conversion now live in the
  pure `agent_context.rs` mapper; it receives explicit query/schema inputs and
  no longer depends on `DbProApp`.
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
- Result-grid keyboard navigation target calculation now lives on
  `TableDataState`; the root handles only egui input and commit-edit effects.
- Result-grid layout scope, persistence and restore transitions now live on
  `TableDataState`; the former `grid_layout.rs` root facade is deleted and
  table-opening flows pass an explicit layout scope into the state owner.
- Result-grid row-identity derivation and cache rebuilding now live on
  `TableDataState`; table-editor code keeps mutation orchestration while the
  grid identity algorithm has one state owner.
- Result-grid identity lookup now also lives on `TableDataState`; grid views
  pass table metadata explicitly and no longer depend on a root lookup facade.
- Mutation-error clearing now lives on `TableMutationState`; table-editor
  views keep feedback and request orchestration but no longer implement the
  target-matching state transition.
- Selected-row projection now lives on `TableDataState`; clipboard code only
  consumes the state-owned indexes for copy/export operations.
- Table metadata primary-key and column-write-policy projections now live on
  `TableState`; connection mutability remains an explicit lifecycle concern at
  the composition boundary.
- Active query buffer-version projection now lives on `QuerySessionState`; query
  dispatchers consume the state API instead of a root helper.
- Active query running-request projection now also lives on
  `QuerySessionState`; query cancellation/dispatch checks read the owning
  session directly.
- Explain-plan/request and result-count projections now live on
  `QuerySessionState`; output, agent and navigation surfaces read query state
  directly without root projection facades.
- Active query messages projection now lives on `QuerySessionState`; output
  and navigation surfaces consume the session-owned message slice directly.
- Active query result projection now lives on `QuerySessionState`; output,
  navigation, palette and agent surfaces consume the session-owned result.
- Active query text read projection now lives on `QuerySessionState`; callers
  read the session directly, while the root setter remains only for prediction
  cancellation plus text mutation orchestration.
- Active output-tab read projection now lives on `QueryOutputState`; output
  rendering reads the tab by active document identity, while tab mutations
  remain explicit orchestration transitions.
- Active output-tab mutation now also lives on `QueryOutputState`; query views
  pass the active document identity directly and the root tab setter is gone.
- Per-document output-tab routing now lives on `QueryOutputState`; the active
  document condition is evaluated by the feature state, not by query view code.
- Row-identity matching now uses the published `table_events` function
  directly; the composition root no longer exposes a forwarding helper.
- Toast mutations now live on `FeedbackState`; error/success/info notifications
  no longer route through `DbProApp` wrappers.
- Activity-bar rendering now lives in an explicit renderer that owns only
  workspace-shell state and returns navigation intents; it no longer
  implements a `DbProApp` method. The architecture guard enforces this seam.
- Named workspace-session capture, restore, duplication and persistence now
  run through `WorkspaceSessionContext` with explicit aggregate dependencies;
  the session module no longer implements `DbProApp` methods.
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
- Post-refactor release framebuffer capture: PASS at logical `1280x800`
  (`/tmp/db-pro-native-post-refactor.png`); Welcome/empty state, activity rail,
  sidebar, query tabs and status bar were inspected after the state-owner
  changes.
- Latest query-state release framebuffer capture: PASS at logical `1280x800`
  (`/tmp/db-pro-native-post-query-state-refactor.png`); the same Welcome/empty
  surface was inspected after the active-query projection extractions.
- Unit tests for the extracted aggregates are included in the UI test suite.
- Query-document context tests cover explicit connection/schema binding and
  document-owned output-tab cleanup during close.
- `cargo check -p db-pro-ui`: PASS.
- `cargo fmt --all`: executed.
- `cargo clippy -p db-pro-ui --all-targets -- -D warnings`: PASS.
- `cargo test -p db-pro-ui --lib`: 632 passed, 0 failed.
- `cargo fmt --all -- --check`: PASS.
- `cargo check --workspace`: PASS.
- `cargo clippy --workspace --all-targets -- -D warnings`: PASS.
- `cargo test --workspace --no-fail-fast`: 1304 passed, 0 failed, 42 ignored;
  all workspace doc-tests passed with 0 tests.
- `cargo build --release --locked -p db-pro-native`: PASS.
- `cargo build --release --locked -p db-pro-native --features capture`: PASS.
- `bash .skills/clean-code/scripts/clean-code-scan.sh rust --diff --ratchet --ci`: 12 pass, 4 warnings, 0 failures; warnings are ratcheted size/file/clone heuristics.
- Synthetic/transfer harness extraction: `navigation_view.rs` reduced from
  3503 to 3021 lines; the 482-line implementation now lives in
  `transfer_harness_view.rs`.
- Post-extraction focused verification: `cargo check -p db-pro-ui` PASS,
  `cargo test -p db-pro-ui --lib` 632 passed, architecture guard PASS,
  `cargo clippy -p db-pro-ui --all-targets -- -D warnings` PASS, and
  `git diff --check` PASS.
- Table mutation-dialog extraction: `table_editor_view.rs` reduced from 2643
  to 2146 lines; the 504-line discard/pending/conflict slice now lives in
  `table_mutation_dialogs_view.rs`. Focused clippy, architecture guard, clean
  scan and 632 UI tests all PASS; clean scan remains 12 pass, 4 ratcheted
  warnings, 0 failures.
- Insert-row workflow extraction: `table_editor_view.rs` reduced from 2146 to
  1795 lines; the 357-line open/duplicate/submit/dialog slice now lives in
  `table_insert_row_view.rs`. Focused clippy, architecture guard, clean scan
  and 632 UI tests all PASS; clean scan remains 12 pass, 4 ratcheted warnings,
  0 failures.
- Security activity extraction: `navigation_view.rs` reduced from 3021 to 2438
  lines; the 586-line roles/RLS/policy slice now lives in
  `security_activity_view.rs`. Focused clippy, architecture guard, clean scan
  and 632 UI tests all PASS; clean scan remains 12 pass, 4 ratcheted warnings,
  0 failures.
- Management activity extraction: `navigation_view.rs` reduced from 2438 to
  971 lines; the 1470-line monitoring/audit/settings/FDW/replication/event
  trigger slice now lives in `database_management_view.rs`. Focused clippy,
  architecture guard, clean scan and 632 UI tests all PASS; clean scan remains
  12 pass, 4 ratcheted warnings, 0 failures.
- Final source checkpoint: `426c8790` on `main`; worktree clean after the
  management extraction.
- Final full-gate rerun at this checkpoint: `cargo fmt --all -- --check`,
  `cargo check --workspace`, `cargo clippy --workspace --all-targets -- -D
  warnings`, `cargo test --workspace --no-fail-fast` (1304 passed, 0 failed,
  42 ignored), `cargo build --release --locked -p db-pro-native`, and the
  capture-feature release build all PASS.
- Final runtime evidence: `/tmp/db-pro-native-core-current.png`, captured
  from the release binary at logical `1280x800` with the New Connection modal
  open; centered card, separated header, right-aligned close control, scroll
  body and footer were visually inspected.
- Management view topology extraction: deleted the 1490-line aggregate
  `database_management_view.rs` and split it into seven feature-owned view
  modules. Monitoring snapshot rendering is now split into health/local,
  sessions, server stats, workload and confirmation methods. Focused check,
  632 UI tests, clippy, architecture guard and clean scan all PASS; clean scan
  reports 14 pass, 2 ratcheted warnings, 0 failures.
- Runtime event/lifecycle extraction: `app.rs` reduced from 1187 to 734 lines;
  runtime transitions live in `runtime_event_handlers.rs` (231 lines) and the
  `eframe::App` adapter plus persistence/frame helpers live in
  `app_lifecycle.rs` (253 lines). Focused clippy, architecture guard and 632 UI
  tests PASS; clean scan is 16 pass, 0 warnings, 0 failures.
- Table editor state aggregation: `DbProApp` now owns one `TableEditorState`
  aggregate instead of separate `table_state`, `table_data` and
  `table_mutation` root fields. Consumers use `self.table.state`,
  `self.table.data` or `self.table.mutation`; the architecture allowlist was
  updated to require the aggregate. Focused UI tests (632 passed), focused
  clippy, architecture guard and clean scan PASS; clean scan reports 12 pass,
  4 ratcheted baseline warnings, 0 failures.
- Current release runtime capture: PASS at logical `1280x800`
  (`/tmp/db-pro-native-table-data-query.png`) after rebuilding both
  `db-pro-native` release variants. The New Connection surface still shows a
  centered modal card, separated header, right-aligned close action, scrollable
  body and footer; the state-owner refactor did not regress the visual surface.
- Query state aggregation: `DbProApp` now owns one `QueryFeatureState`
  aggregate instead of separate query session/editor/output/execution/library
  root fields. Focused UI tests (632 passed), focused clippy, architecture
  guard and clean scan PASS; clean scan reports 12 pass, 4 ratcheted baseline
  warnings, 0 failures.
- Storage hydration extraction: startup key parsing now lives in
  `app_storage.rs` behind `NativeStorageContext` and
  `NativeStorageDependencies`; `app_state.rs` only sequences restore phases.
  Focused compile, clippy, architecture guard and 632 UI tests PASS; clean
  scan reports 15 pass, 1 ratcheted warning, 0 failures.
- Table data-query boundary extraction: `TableState` no longer owns result,
  paging, filters, sorts or row-reload lifecycle. Those concerns now live in
  `TableDataQueryState`; table reducers and views receive the explicit state
  boundary. Focused UI tests (632 passed), clippy, architecture guard and
  clean scan PASS; clean scan reports 12 pass, 4 ratcheted warnings, 0 failures.
- Source checkpoint `96d52bd7`: full release gate rerun on `main` passed
  (`cargo fmt --all -- --check`, workspace check/clippy, workspace tests;
  1304 passed, 42 ignored, 0 failed), both native release builds passed, and
  the deterministic capture at `/tmp/db-pro-native-table-data-query.png`
  passed at logical `1280x800`.
- Table-data behavior ownership follow-up: filter-operator compatibility and
  table-query reset/invalidate/page transitions now live in
  `TableDataQueryState`; focused UI tests (632 passed), clippy, fmt and
  architecture guard PASS.
- Final merged `main` checkpoint `5332876c`: after merging the concurrent SQL
  Server PR, workspace format/check/clippy/tests and both native release builds
  passed again; final modal capture is `/tmp/db-pro-native-final-main.png` at
  logical `1280x800`.
- Grid layout ownership follow-up: `TableDataState` now owns column ordering,
  visibility, movement, auto-sizing and width calculation; `result_grid_view`
  no longer exposes those as `DbProApp` methods. Focused UI tests (632 passed),
  clippy, fmt and architecture guard PASS.
- Table editing ownership follow-up: cell-edit buffers, inspector state,
  discard confirmations and insert-row form state now live in
  `TableEditingState`, separate from grid projection/layout/selection state.
  Focused UI tests (632 passed), clippy, architecture guard and diff checks
  PASS.
- Design Mode action ownership follow-up: ER foreign-key draft parsing,
  mutation-plan preview and query-runtime apply orchestration now live in
  `diagram_design_actions.rs`; `diagram_view.rs` retains the Design Mode UI
  surface and diagram rendering. Focused UI check, clippy, 632 UI tests,
  architecture guard and diff checks PASS.
- ER canvas interaction ownership follow-up: empty state, canvas surface,
  pan handling and click-through table navigation now live in
  `diagram_canvas_view.rs`; `diagram_view.rs` coordinates schema candidates,
  toolbar and Design Mode. Focused UI check, clippy, 632 UI tests,
  architecture guard, clean scan and diff checks PASS.
- Current source checkpoint: `6b258257` on `main`. Full regression gate PASS:
  `cargo fmt --all -- --check`, workspace check, workspace clippy with
  `-D warnings`, workspace tests (`0 failed`), native release build and
  capture-feature release build.
- Current runtime evidence: `/tmp/db-pro-native-core-final.png`, captured from
  the rebuilt release binary at logical `1280x800` with the
  New Connection modal open. Visual inspection confirms the centered card,
  separated header, right-aligned close control, scrollable body and footer.
- Design Mode panel ownership follow-up: draft table/column/FK form rendering
  now lives in `diagram_design_panel_view.rs`; `diagram_view.rs` only
  coordinates whether the panel is shown. Focused UI tests (632 passed),
  clippy, architecture guard and clean scan (14 pass, 2 ratcheted warnings)
  PASS. One timing-sensitive chart performance assertion exceeded its budget
  once at 300.7ms and passed on the isolated rerun and subsequent full UI run;
  no product test failure remains.
- Database-management state ownership follow-up: audit, event-trigger, FDW,
  masking, monitoring, PostgreSQL settings, replication, routine, security,
  synthetic-data and transfer state now cross the single
  `DatabaseManagementState` aggregate through `DbProApp.management`. Focused
  UI tests (632 passed), clippy, architecture guard, clean scan and diff
  checks PASS.
- Schema workspace state ownership follow-up: explorer, schema workbench,
  schema compare and ER diagram state now cross the single
  `SchemaWorkspaceState` aggregate through `DbProApp.schema`; storage and
  reducer contexts still receive explicit child dependencies. Focused UI
  tests (632 passed), clippy, architecture guard and diff checks PASS.
- Schema comparison view ownership follow-up: schema-compare sidebar,
  migration/data-compare surface and keyed data-diff dispatch now live in
  `schema_compare_view.rs`; `navigation_view.rs` is reduced to shell and
  activity surfaces. Focused UI tests (632 passed), clippy, architecture
  guard, clean scan (15 pass, 1 ratcheted warning) and diff checks PASS.
- Transfer activity ownership follow-up: backup/restore entry point,
  synthetic seed, masking preview, transfer harness controls and job history
  now live in `transfer_activity_view.rs`; `navigation_view.rs` is reduced to
  shell/status/output and schema/diagram navigation. Focused UI tests (632
  passed), clippy, architecture guard, clean scan (15 pass, 1 ratcheted
  warning) and diff checks PASS.
- Shell chrome ownership follow-up: top bar, status bar and output dock now
  live in `shell_chrome_view.rs`; `navigation_view.rs` is reduced to the
  remaining navigation activity composition. Focused UI tests (632 passed),
  clippy, architecture guard, clean scan (15 pass, 1 ratcheted warning) and
  diff checks PASS.
- Query tool ownership follow-up: query action menu/editor actions now live
  in `query_actions_view.rs`, while find overlays/bars live in
  `query_search_view.rs`; `query_view.rs` is below the 800-line file boundary.
  Focused UI tests (632 passed), clippy, architecture guard, clean scan (15
  pass, 1 ratcheted warning) and diff checks PASS.
- Palette ownership follow-up: static Quick Open/Command catalog builders now
  live in `palette_catalog.rs`, and palette action routing lives in
  `palette_actions.rs`; `palette_view.rs` is reduced to indexing/filtering and
  dialog coordination. Focused UI tests (632 passed), UI clippy, architecture
  guard, clean scan (14 pass, 2 ratcheted warnings) and diff checks PASS.
- Table editor ownership follow-up: data-grid/paging rendering now lives in
  `table_data_view.rs`, while row editing and staged mutation lifecycle live in
  `table_mutation_actions.rs`; `table_editor_view.rs` is reduced to DDL and
  table-request/filter coordination. Focused UI tests (632 passed), UI clippy,
  architecture guard and diff checks PASS; clean scan has no failures and only
  ratcheted legacy function-size/clone warnings.

## Not yet proven

- Native screenshot/runtime evidence for the requested `1440x900` and
  `1920x1080` logical heights remains host-limited: macOS capture clamps both
  to a logical height of `838`. The required normal/loading/error/empty states
  are now captured at exact logical `1280x800`.
