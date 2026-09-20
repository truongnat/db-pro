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
   `0fc757d8`; mutation effects remain.
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
9. Migrate native Explorer rendering in vertical slices: connection/database/
   schema/table/schema-object row views now emit typed intents; folder,
   workspace and runtime reducers remain in progress.
10. Continue the same intent/reducer boundary through the remaining large
    native surfaces. The Agent workflow/settings/header/confirmation/context
    slices are now explicit; monitoring, security, settings sections and
    remaining table/query surfaces still require migration.

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
