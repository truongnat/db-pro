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
3. Workspace shell state and navigation reducer.
4. Query document/session state and output lifecycle.
5. Table/data editor state and mutation effects.
6. Agent state and task lifecycle.
7. Remove remaining feature fields from `DbProApp`; leave it as composition,
   persistence, event pump, and top-level orchestration only.
8. Add architecture checks so new feature code cannot reach another feature's
   internals or reintroduce raw control paths.

## Non-goals

- No provider behavior changes.
- No SQL semantics changes.
- No React/frontend restoration.
- No speculative service or microservice split.

## Completion criteria

- Every feature state has one owner and a public transition surface.
- UI event handling is feature-dispatched and testable without egui painting.
- `DbProApp` contains no feature-specific draft/result collection once the
  corresponding feature migration is complete.
- Core behavior tests cover open/edit/duplicate/close, stale request guards,
  and error/loading/success transitions.
- Native build, workspace tests, clippy, formatting, and runtime evidence are
  recorded truthfully.
