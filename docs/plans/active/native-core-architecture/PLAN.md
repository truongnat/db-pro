# Native Core Architecture — Plan

State: IMPLEMENTING
Branch: `main`

## Objective

Rebuild the native UI around explicit feature-owned state and one-way
application transitions. `DbProApp` remains the egui composition root, but it
must stop being the owner of every feature's mutable state and business
transition.

## Target shape

```text
egui views
  -> feature intent
  -> feature reducer / state aggregate
  -> UiCommand
  -> TaskBridge
  -> native-app adapter
  -> runtime/core/infrastructure
  -> UiEvent
  -> feature reducer
  -> repaint
```

The UI layer owns presentation state and intent mapping only. Domain rules and
provider behavior stay in `crates/core` and `crates/infrastructure`.

## Migration order

1. Connection dialog state aggregate — completed in `f8a091eb`.
2. Connection session state and lifecycle transitions — lifecycle completed in
   `13dedb61`; saved-connection read model completed in `d9ff6b00`.
3. Workspace shell state and navigation reducer — completed in `a483000a`.
4. Query document/session state and output lifecycle — completed in
   `24f8692a`.
5. Table/data editor state and mutation effects — grid/editor interaction state
   completed in `77a27f0c`; table metadata/request state completed in
   `0fc757d8`; mutation effects and the remaining table surface adapters are
    now feature-owned in `f5fc419d`; Explain validation and transitions are
    isolated in `ef33bb1c`; saved-query preparation is isolated in
    `2b711995`; Schema Workbench mutation planning is feature-owned in
    `1d8647dc`; command dispatch bypasses were closed and guarded in
    `d687de8e`; Explorer navigation reset invariants are unified in
    `bdf2e363`.
6. Agent state and query-editor/schema-explorer state — completed; workspace
   files, diagram, database operations, palette, query execution policy, query
   library, saved tasks, named sessions, overlays, feedback, preferences and
   welcome state are also extracted.
7. Keep the runtime event dispatch table isolated in `event_router.rs`; split
   feature reducers out of `events.rs` and keep `DbProApp` as composition,
   persistence, event pump, and top-level orchestration only — completed.
8. Add architecture checks so new feature code cannot reach another feature's
   internals or reintroduce raw control paths — completed by
   `scripts/check-ui-architecture.sh` and CI.
9. Migrate native Explorer rendering in vertical slices: the complete Explorer
   surface now emits typed intents through explicit view boundaries for
   toolbar, empty state, connection nodes, database/schema trees,
   schema-object folders, tables and table details. Root effect adapters remain
   in progress alongside equivalent adapters in other large surfaces.
10. Continue the same intent/reducer boundary through the remaining large
    native surfaces. The Agent workflow/settings/header/confirmation/context
    slices, large Settings sections, table surfaces and query execution
    preparation are now explicit; Saved Tasks, Settings system /
    Appearance/Backup, Welcome, Query dialogs and the shell Output Panel now
    emit through explicit surface contexts; Query output dock geometry and tab
    chrome now use a dedicated context while pane effects remain at the root;
    shell topbar navigation and chrome now use the same boundary; Explorer
    statusbar chrome now uses the same boundary; Explorer adapters,
    Agent thread rendering and panel/context presentation now use explicit
    contexts; Security confirmation, the composed Security surface, monitoring
    presentation and result-grid viewport composition now use the same
    boundary; query panel geometry, Audit, Event Trigger, FDW, Logical
    Replication, PostgreSQL settings, workspace-files, query-actions, Visual
    Query Builder, workspace secondary-tab, table-structure, query-snippets,
    table-profile, Backup Settings, saved-query confirmation, table-indexes and
    schema-definition, saved-task confirmation, result-grid value inspector,
    record inspector, inline cell editor and sidebar shell presentation are now
    explicit; native feature dialogs now use the shared Dialog primitive;
    Audit, FDW, Event Trigger, Logical Replication and PostgreSQL settings root
    adapters are now migrated to explicit contexts; Security activity is now
    migrated to `SecurityActivityContext` and Monitoring to
    `MonitoringActivityContext`; Welcome composition is co-located with the
    workspace boundary; workspace-tab composition is co-located there as well;
    the remaining root adapters still require migration.
11. Enforce the runtime command boundary as a two-phase transition: prepare
    typed command, dispatch through `TaskBridge`, then commit local pending or
    running state only after dispatch succeeds. This is now applied to the
    connection, schema, table, query and mutation paths; remaining surfaces
    still need the same audit.

## Non-goals

- No provider behavior changes.
- No SQL semantics changes.
- No React/frontend restoration.
- No speculative service or microservice split.

## Completion criteria

- Every migrated feature state has one owner and a public transition surface.
- UI event dispatch is isolated from feature handlers and remains testable
  without egui painting.
- `DbProApp` contains no feature-specific draft/result collection once the
  corresponding feature migration is complete.
- Core behavior tests cover open/edit/duplicate/close, stale request guards,
  and error/loading/success transitions.
- Native build, workspace tests, clippy, formatting, and runtime evidence are
  recorded truthfully.
