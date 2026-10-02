# rs-ui Runtime Integration

State: IMPLEMENTING
Branch: `main` (repository owner workflow override)

## Goal

Adopt the separate rs-ui runtime incrementally as the UI layout and behavior foundation for DB Pro while preserving the existing `DbProApp`, task bridge, domain state, database services, and egui window host.

## Scope

- Pin the rs-ui core and runtime crates to an exact Git revision.
- Create a thin DB Pro shell layout adapter backed by `UiTree`.
- Use rs-ui-computed sidebar geometry in the existing egui host.
- Route sidebar resizing and focused keyboard adjustment through rs-ui behavior
  while DB Pro remains the source of persisted panel width.
- Route sidebar wheel input and retained offset through rs-ui `ScrollState`,
  keeping egui for content painting and scrollbar interaction during migration.
- Mirror the visible Explorer connection/database/schema/Tables/table hierarchy
  into rs-ui `Tree` / `TreeItem` semantics with stable model-derived keys,
  selection, expansion, focus and keyboard traversal while preserving existing
  DB Pro actions and schema state.
- Keep the renderer boundary explicit and track the next migration stages.

## Non-goals

- Replacing the eframe window host or the current renderer.
- Rewriting product state, commands, connection/query/schema logic, or editor.
- Rewriting the result-grid painter or moving database/projection logic into rs-ui.
- Migrating search/filter inputs or the SQL editor in this stage.
- Copying rs-ui source into DB Pro.

## Current architecture and boundary

DB Pro's shipped UI is `db-pro-ui` (`crates/ui`) hosted by `db-pro-native` (`crates/native-app`) through eframe/egui. `DbProApp` owns product UI state and communicates through the existing typed command/event bridge. The DB Pro runtime and domain crates remain independent of UI rendering.

The rs-ui workspace provides `ui-core` geometry/display-list types and `ui-runtime` layout, interaction, focus, scroll, and accessibility behavior. Its renderer is a separate wgpu renderer and its window crate uses winit. This first adapter consumes only the renderer-independent core/runtime boundary; egui remains the host and painter while migration is incremental.

## Reuse and coupling

- Reuse `DbProApp`, existing workspace/connection/query/schema state, command/event handling, persistence, and provider services unchanged.
- Keep rs-ui types inside `crates/ui` adapters. Do not add rs-ui dependencies to core, infrastructure, or runtime.
- Keep current tab actions and sidebar content implementations; adapters return
  into existing DB Pro state/action handling.

## Migration order

1. Host integration and shell geometry adapter.
2. Sidebar/split/tab geometry adapter, then normalized input/action routing.
3. Schema tree adapter.
4. Virtual result grid adapter with typed-cell preservation and measurements.
5. Search/filter inputs.
6. Decide separately whether SQL editor migration is warranted.
7. Remove egui shell pieces only after runtime and visual evidence.

## Risks

- Historical blocker: rs-ui `db3cf2bfed3d29f0e1d19963462488ed9157f1ea` lacked behavior/resize APIs used by DB Pro. That finding applies only to that SHA. The current exact pin is `f6e798d6cfa966b5344cf6a9de6c634563258eec`.
- eframe currently uses the glow renderer; rs-ui's renderer targets wgpu. Switching rendering backends is outside this adapter and needs an isolated technical spike before any full renderer migration.
- The current host adapter rebuilds a small retained tree for the shell layout request. Measure before expanding that pattern to high-frequency surfaces.
- Explorer rows currently use egui for painting, expansion state, and row
  viewport clipping. The adapter mirrors visible rows into rs-ui and routes
  ArrowUp/ArrowDown focus traversal through its behavior runtime; it does not
  replace Explorer rendering or expose rs-ui semantics through a native OS
  accessibility backend. Keyboard traversal can only target rows rendered in
  the current viewport.
- Stage 4 uses rs-ui `VirtualGrid` and `ScrollState` to calculate the vertical
  row window from egui's scroll offset while egui remains the scroll host and
  painter. DB Pro retains filtered projection, typed cells, column order,
  variable widths, selection, and persisted width state. Horizontal
  virtualization is deferred because the pinned `VirtualGrid` API only accepts
  one fixed column extent. SelectionModel and Resizable now handle behavior;
  DB Pro remains the source of truth for selected rows/cells and widths.
- Stage 4 source/tests and build gates pass against the exact Git pin. Native
  result-grid runtime evidence remains outstanding.

## Acceptance

- The DB Pro UI crate depends only on rs-ui core/runtime crates at the same exact Git revision.
- The shell adapter uses rs-ui layout results and rejects viewports that cannot fit a usable main surface.
- Existing domain state and command/task bridge remain unchanged.
- Explorer semantics represent the existing visible hierarchy and clicks
  continue through DB Pro's existing selection and action paths.
- Stage 4 must preserve the existing projection, typed cells, column widths,
  selection, clipboard, and egui cell painter while measuring visible work.
- Targeted tests and Rust checks are recorded honestly.
- Runtime screenshots and end-to-end smoke evidence remain required before marking this integration complete.
