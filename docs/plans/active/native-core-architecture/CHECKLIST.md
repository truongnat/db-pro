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
- [x] Saved Tasks renderer separated into an explicit surface context with
      typed root actions; scheduler and runtime dispatch remain at the root
      boundary.
- [x] Settings system and Appearance panes separated into explicit contexts;
      settings root retains navigation and runtime/file orchestration.
- [x] Settings Backup/Restore form separated into an explicit context with
      typed runtime actions.
- [x] Welcome surface separated into an explicit context with typed start,
      connection and draft-query actions.
- [x] Query destructive/export/save-as/dirty-close dialogs separated into
      explicit contexts with typed actions.
- [x] Query output dock resize geometry and tab chrome separated into an
      explicit context; result-pane effects remain at the root adapter.
- [x] Shell topbar rendering separated into an explicit context with typed
      navigation, palette, gallery, agent, theme and quick-open actions.
- [x] Shell statusbar rendering separated into an explicit context with a
      typed output-panel toggle action.
- [x] Agent thread rendering separated into an immutable context with typed
      submit, result, retry and confirmation actions.
- [x] Agent panel shell and context-chip presentation separated into explicit
      contexts; settings/header/composer effects remain root adapters.
- [x] Security drop-role confirmation separated into an explicit context with
      typed confirm/cancel actions.
- [x] Security surface composition separated into an explicit context with a
      unified roles/details/confirmation/RLS action stream; command dispatch
      and provider effects remain at the root adapter.
- [x] Monitoring header, snapshot, sessions and workload presentation
      separated into an explicit context with typed refresh/session/workload
      actions; polling and auxiliary runtime effects remain at the root
      adapter.
- [x] Result-grid viewport composition separated into an explicit context with
      a renderer contract for header and virtualized rows; grid state and row
      effects remain at the root adapter.
- [x] Query output dock/editor height policy separated into a pure layout
      context with focused geometry tests.
- [x] Audit activity presentation separated into an explicit context with
      typed refresh/export/select/bookmark/open-query actions.
- [x] Event Trigger presentation separated into an explicit context with
      typed refresh/preview/alter/create/drop actions.
- [x] FDW presentation separated into an explicit context with typed
      refresh/preview/create/drop actions; provider command effects remain at
      the root adapter.
- [x] Logical Replication presentation separated into an explicit context with
      typed refresh/preview/create/drop actions; provider command effects
      remain at the root adapter.
- [x] PostgreSQL settings presentation separated into an explicit context with
      typed refresh/edit/reset/preview actions; command dispatch remains at the
      root adapter.
- [x] Workspace-files shell presentation separated into an explicit context
      with typed folder/root/trust/environment/tab actions; filesystem and
      navigation effects remain at the root adapter.
- [x] Query actions menu presentation separated into an explicit context with
      typed run/format/explain/save/editor/prediction actions; query effects
      remain at the root adapter.
- [x] Visual Query Builder presentation separated into an explicit context
      with explicit query-state/schema/dialect inputs and typed apply/import/
      clear intents; editor/navigation effects remain at the root adapter.
- [x] Table structure metrics, columns and column-detail presentation
      separated into an explicit context with typed column-selection/close
      actions; table state and root navigation remain at the adapter.
- [x] Query snippets presentation separated into an explicit context with a
      typed insertion action; document mutation remains at the root adapter.
- [x] Table profile grid, bounded page profiling and structure loading/error
      placeholder presentation separated into explicit surfaces; table routing
      and state remain at the root adapters.
- [x] Backup Settings card presentation separated into its feature context;
      provider capability/driver inputs and backup command effects remain at
      the settings root adapter.
- [x] Saved-query delete confirmation presentation separated into the query
      library context with typed confirm/cancel actions; command dispatch and
      overlay ownership remain at the root adapter.
- [x] Table indexes filter/table/detail presentation separated into an explicit
      context with typed select/close actions; table metadata state remains at
      the root adapter.
- [x] Schema-object definition card presentation separated into an explicit
      context; schema-object routing remains at the root adapter.
- [x] Saved-task destructive-run confirmation presentation moved into the task
      surface; root retains policy and task execution effects.
- [x] Result-grid value inspector and full-record inspector presentation
      separated into explicit surfaces with typed copy/export/apply/close and
      inspect actions; editor state and effects remain at the root adapter.
- [x] Result-grid inline cell editor and sidebar shell geometry/chrome
      separated into explicit surfaces with typed commit/cancel, navigation and
      resize actions; feature activity renderers remain at the root adapter.
- [x] Native feature dialogs use the shared Dialog primitive for centered
      layout, separated header, right-aligned close control and typed actions;
      direct feature-level egui Window construction is eliminated.
- [x] Workspace Migrations and Graph tabs separated into explicit renderers
      with typed file-open intents; file navigation remains at the root
      adapter.
- [x] Shell Output Panel separated into an explicit context for panel layout,
      tab rendering and close/resize state.
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
- [x] `DbProApp` reduced to composition root (event pump and cross-feature
      orchestration only); `app.rs` has no egui painting or feature-specific
      draft/result collection, enforced by the architecture guard. Feature
      effect adapters remain in their owning view modules; runtime evidence is
      tracked separately below.
- [x] Query result-pane shell rendering is owned by an explicit surface
      context; the root retains only snapshot preparation, grid callback,
      typed intent application and feature-specific dialog adapters.
- [x] Activity rail rendering consumes an immutable context and emits typed
      navigation intents; workspace mutation remains in the composition-root
      adapter.
- [x] Shell central-panel geometry is owned by `ShellFrameContext`; lifecycle
      only composes feature panels and applies shell actions.
- [x] Queries and History sidebar composition is owned by
      `SidebarQueriesSurfaceContext`; the root only applies the unified typed
      query/library/shortcut action stream.
- [x] Table data grid framing and metadata scroll geometry are owned by
      explicit layout surfaces; table coordinators retain grid interaction and
      runtime mutation effects only.
- [x] Query Visual Builder framing and editor-stack allocation are owned by
      `query_shell_surface_view`; query root retains editor/search/effect
      adapters only.
- [x] Agent panel composition is owned by `AgentPanelContext`; the root retains
      snapshot preparation and typed header/settings/context/thread/composer
      effect adapters.
- [x] Result-grid keyboard/edit interaction is owned by
      `ResultGridInteractionContext`; root retains only typed clipboard,
      mutation, edit and navigation effect adapters.
- [x] Saved Tasks draft and schedule lifecycle is owned by `SavedTaskState`;
      task runtime dispatch remains at the composition root.
- [x] Saved Tasks destructive-run policy and run-history recording are owned by
      `SavedTaskState`; runtime payload dispatch remains at the composition
      root.
- [x] Saved Tasks export and maintenance SQL is prepared through a provider-aware
      pure boundary; qualified identifiers are escaped, unsupported provider
      syntax is rejected, and the architecture guard blocks raw interpolation.
- [x] Workspace-files panel selection, root/environment/trust transitions,
      directory expansion, search routing and external-change dismissal are
      owned by `WorkspaceFilesState`; file activity adapters no longer mutate
      those internals directly.
- [x] Workspace search/replace/refactor inputs cross the renderer boundary as
      a `WorkspaceSearchDraft` snapshot and typed `UpdateDraft` intent.
- [x] Agent lifecycle state is isolated from composition-root adapters; runtime
      command/effect methods live in `agent_actions.rs`, and all feature
      `*_state.rs` modules are guarded against `DbProApp` dependencies.
- [x] Runtime command sends from dialogs, query documents and editor surfaces
      cross the `RuntimeCommandDispatcher` port; direct best-effort
      `TaskBridge` sends are rejected by the architecture guard.
- [x] Request-ID allocation crosses the composition-root port through
      `DbProApp::next_request_id`; direct feature access to
      `TaskBridge::next_request_id` is rejected by the architecture guard.
- [x] Query execution and saved-query contexts return typed prepared effects;
      `RunQuery`, `RunQueryMulti` and saved-query protocol mapping are isolated
      in command adapters and guarded against leaking back into feature state.
- [x] Query editor rendering returns typed runtime effects; request-id allocation,
      dispatch and prediction-request commit remain in the query composition
      adapter, while the surface owns only editor state transitions.
- [x] `TableState` owns table lifecycle and DDL validation without constructing
      `UiCommand`; table runtime protocol mapping remains in the table editor
      effect adapter and is protected by the architecture guard.
- [x] Query library read-model/draft state no longer constructs runtime
      protocol commands; saved-query/folder mapping is centralized in the query
      save adapter and all callers use that boundary.
- [x] Management read-only states no longer construct runtime protocol
      commands; audit, FDW, replication, event-trigger and PostgreSQL-settings
      mapping is centralized in activity adapters and guarded by architecture
      checks.
- [x] Audit, FDW, Event Trigger and Logical Replication activity adapters no
      longer implement `DbProApp`; they receive explicit feature state and
      runtime dependencies, and Audit cross-feature navigation is a typed
      effect applied by the root.
- [x] PostgreSQL settings activity uses an explicit context; failed SET
      dispatch preserves the edit state until the runtime accepts the command.
- [x] Monitoring state no longer constructs runtime protocol commands;
      snapshot, workload, session-control, maintenance and reset mapping is
      centralized in the monitoring activity adapter and architecture-guarded.
- [x] Security state no longer constructs runtime protocol commands; role,
      privilege, membership and RLS mapping is centralized in the security
      activity adapter and architecture-guarded.
- [x] Overlay backup/restore state no longer constructs runtime protocol
      commands; file-picker, backup and restore mapping is centralized in the
      settings adapter and architecture-guarded.
- [x] Schema-compare state no longer constructs runtime protocol commands;
      snapshot/diff/migration planning remains pure while final command
      construction is centralized in the workspace/query adapters.
- [x] Connection lifecycle state no longer constructs runtime protocol
      commands; connection switching is centralized in the connection logic
      adapter.
- [x] Schema Workbench and Explorer connection navigation prepare typed
      requests without constructing `UiCommand`; protocol mapping stays in
      their root effect adapters and is architecture-guarded.
- [x] Confirmation and sensitive-form state commits happen only after a
      successful runtime dispatch; failed Security, connection-delete,
      query-delete, folder-delete, monitoring and restore dispatches remain
      retryable. Connection failure classification uses typed pending
      operations rather than feedback text.
- [x] Composition root no longer constructs feature runtime commands directly;
      connection loading and schema introspection use feature adapters, with a
      guard preventing `UiCommand::` from returning to `app.rs`.
- [x] App module topology is isolated in `app_modules.rs`; `app.rs` contains
      aggregate ownership/orchestration only, and the guard rejects reintroducing
      feature `#[path]` declarations into the composition root.
- [x] Settings navigation, section composition and diagnostics presentation
      are owned by `SettingsSurfaceContext`; settings root retains only
      persistence, runtime/file effects and typed action application.
- [x] Architecture boundary check in CI (`scripts/check-ui-architecture.sh`).

## Gates

- [x] `cargo fmt --all -- --check` (PASS)
- [x] `cargo check --workspace`
- [x] `cargo clippy -p db-pro-ui --all-targets -- -D warnings`
- [x] UI test suite through the workspace gate (686 passed at `6ff9f937`)
- [x] `cargo test --workspace --no-fail-fast --quiet` (PASS at `6ff9f937`; 0 failed)
- [x] `cargo build --release --locked -p db-pro-native`
- [x] `cargo build --release --locked -p db-pro-native --features capture`
- [x] New Connection modal runtime capture at logical `1280x800` (centered
      card, separated header, right-aligned close, sticky footer).
- [x] Explorer intent-boundary slices for connection, database, schema, table
      and schema-object rows.
- [x] Loading Welcome and New Connection error state captures at logical
      `1280x800`.
- [x] Latest native release binary rebuilt on `6ff9f937`; runtime launch and
      the normal Welcome capture are recorded in `VERIFICATION.md`.
- [x] Native runtime evidence for affected surfaces recorded at the required
      viewport/state matrix in `VERIFICATION.md`.
