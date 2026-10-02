# rs-ui Integration Plan

> Deferred by owner on 2026-10-02. DB Pro executable integration/dependencies have been removed and the existing egui UI restored. This document describes the historical migration plan. Current evidence: `docs/plans/active/restore-egui-ui/`.

## Current DB Pro UI architecture

- `crates/native-app/src/main.rs` launches the native `DbProApp` through eframe.
- `crates/ui/src/app.rs` composes product state and views. Connection, query,
  schema, table, and workspace state remain DB Pro-owned.
- `crates/ui/src/runtime.rs` exposes the typed `UiCommand` / `UiEvent` protocol
  and `TaskBridge`; `db-pro-runtime` owns services and background work.
- The native UI uses egui for window input, layout, and painting. The archived
  React/Tauri UI is reference material only.

## Boundary and reusable state

Keep connection lifecycle/catalog, query documents and results, schema models,
workspace tabs, persistence, commands, task bridge, and provider services as-is.
rs-ui types belong only in adapters under `crates/ui`; domain, infrastructure,
and runtime crates must not depend on rs-ui.

The existing egui views still own much of their own interaction and painting.
Adapters may hold temporary UI state such as focus and active resize gestures,
but must not mirror DB Pro business state. UI actions return through existing
view actions and state transitions.

## Dependency and host seam

`crates/ui/Cargo.toml` uses local paths to the sibling `ui-core` and
`ui-runtime` crates. The adapter in `crates/ui/src/native_runtime_shell.rs`
builds rs-ui layout nodes, owns shell and scroll state, and retains a separate
Explorer semantic tree. The egui views consume its geometry and offsets, paint
the content, and sync scrollbar offsets back into the adapter. Explorer
connection/database/schema/Tables/table rows mirror stable IDs, selection,
expansion, bounds, and focus to TreeItem semantics; existing DB Pro state and
click/action paths still own behavior. The current rs-ui checkout is modified and
uncommitted, so this local dependency is a development setup, not a reproducible
release pin.

The eframe host currently uses egui's glow renderer. rs-ui's renderer uses wgpu
and its window crate uses winit. They are not part of the first host adapter;
renderer replacement requires a separate compatibility and performance spike.

## Migration order

1. Host seam and shell geometry.
2. Sidebar resize/focus, split-pane geometry, tabs, and normalized actions.
3. Schema tree adapter using stable DB Pro model keys, existing selection, and
   rs-ui focus traversal. Painting, expand state, and viewport clipping remain
   in egui for this incremental slice.
4. Result grid adapter preserving typed cells and requesting only visible data.
5. Search/filter input adapters.
6. Evaluate SQL editor migration separately; retain the current editor until a
   measured case supports replacing it.
7. Remove old egui surface code only after the corresponding rs-ui path passes
   runtime, accessibility, and performance checks.

## Verification and risks

Each stage records targeted tests, workspace checks, native evidence, and what
those checks do not prove in
`docs/plans/active/rs-ui-runtime-integration/VERIFICATION.md`. Before large
surface migrations, capture baseline and after measurements for startup, idle
memory, frame/scroll time, and relevant rendering workloads. The result grid
must retain typed values, virtualize visible rows and columns, and report
materialization and timing counts; do not stringify the full result eagerly.

Runtime completion also requires connection, schema, query, tab, result-scroll,
resize, and keyboard-focus smoke checks. PostgreSQL and SQLite evidence must be
reported separately for any database-facing change. No database-facing behavior
is changed by the host/shell stages.
