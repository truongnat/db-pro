# DB Pro — Native UI Architecture

Status: ratified; supersedes the archived React/Vite frontend architecture
Aligned with `docs/08-technology-decisions.md`, `docs/09-architecture-decisions.md`, and
`docs/10-egui-native-migration-plan.md`.

## Direction

The product UI is **native Rust** using `eframe` + `egui` 0.29. There is no WebView, no
DOM, no CSS, no Node runtime, and no JavaScript build step.

The React 19 / TypeScript / Vite frontend that previously ran inside a Tauri 2 system
WebView was **archived on 2026-09-11** under `_archive/frontend/`, along with its React-era
ER renderer benchmark harness under `_archive/bench/`. They are retained as behavioral
references for parity work only. Do not add features to them and do not treat them as a
fallback UI. See `_archive/README.md`.

## Stack

- `eframe` / `egui` 0.29 for the shell, panels, tabs, dialogs, and painting
- `DbProTheme` (`crates/ui/src/theme.rs`) as the single source of design tokens, mapped to
  `egui::Visuals`
- `lucide-icons` as the single icon set
- Native SQL editor and native virtualized result grid (no Monaco, no TanStack Virtual)
- Typed `UiCommand` / `UiEvent` task bridge for all backend interaction
- `db-pro-runtime` as the only service graph the UI talks to

## Component ownership

All UI code lives in `crates/ui` and is owned by the product. Views compose shared
widgets from `crates/ui/src/components.rs` and must not create one-off visual variants
without adding a reusable widget or documenting a feature variant.

Styling, tokens, density, and states are DB Pro code. A widget must read semantic tokens
from `DbProTheme` instead of hard-coding colors, and a token must not be duplicated across
views.

## State boundary

- `AppState` (`crates/ui/src/app_state.rs`) holds all display state, split into
  sub-states rather than one large struct: connection, explorer, workspace, query, schema,
  grid, overlay, settings, agent, diagnostics.
- State changes are one-directional:
  `UserIntent → UiCommand → service → UiEvent → reducer → repaint`.
- `UiCommand` leaves the UI thread through the task bridge; the runtime worker handles it
  off-thread and replies with bounded `UiEvent`s.
- Workers must never mutate display state directly. Only the reducer on the UI thread
  applies events.
- Row data never streams into an unbounded global store. Batches are bounded by row and
  byte limits and carry a request ID so a stale result cannot overwrite a newer one.
- The UI thread must never block: query, introspection, export, backup, connect, and test
  all run as async tasks.

## UI states required

Every data-facing surface defines loading, empty, error, disabled, focused, selected,
editing, and permission/restricted states. Destructive actions require a confirmation
surface showing the connection, target, operation class, and SQL preview.

## Module layout

```text
crates/ui/src/
  lib.rs                  public exports
  app.rs                  eframe::App implementation, frame orchestration
  app_state.rs            AppState sub-states
  components.rs           shared widgets
  theme.rs                DbProTheme tokens → egui::Visuals
  events.rs               UiEvent definitions
  runtime.rs              task bridge (UiCommand → worker → UiEvent)
  navigation_view.rs      activity rail / sidebar navigation
  workspace_view.rs       tab model + central workspace
  connection_view.rs      connection list / editor / dialogs
  query_view.rs           SQL editor, toolbar, history
  result_grid.rs          grid model, virtualization, hit-testing
  result_grid_view.rs     grid rendering and interaction
  explorer_view.rs        schema explorer tree
  schema_object_view.rs   DB object workbench
  table_view.rs           table data surface
  table_editor_view.rs    typed cell editing and staged mutations
  diagram_view.rs         ER diagram painter
  agent_view.rs           agent panel surfaces
  agent_state.rs          agent execution state
  agent.rs                agent domain glue
  palette_view.rs         command palette / quick open
crates/native-app/src/
  main.rs                 native binary entry, window/bootstrap, worker wiring
  translate.rs            UiCommand/UiEvent ↔ domain translation
```

## Runtime verification

UI changes require runtime evidence, not source inspection. The visual acceptance gate
requires screenshots of the affected surface at 1280×800, 1440×900, and 1920×1080, in
normal plus loading/error/empty states. The full gate and the rejected patterns are
defined in `docs/10-egui-native-migration-plan.md`.
