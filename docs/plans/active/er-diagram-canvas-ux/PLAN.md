# ER Diagram Canvas UX

State: IMPLEMENTING
Branch: `main` (owner override 2026-09-30 — no worktree)
Owner asks (2026-10-07): redesign the ER action row to a compact icon row;
the diagram is ugly, cannot drag, cannot navigate.

## Problem

Current canvas (`crates/ui/src/diagram_canvas_view.rs`) wraps the painter in
`egui::ScrollArea::both()` **and** applies a separate `pan` offset on top of the
viewport transform. The scroll area and the pan state fight each other: dragging
is routed to scrollbars, `pan` shifts content inside a scroll rect sized only to
`world_size * zoom`, and there is no wheel zoom, no node drag, no fit-on-open.

Visual issues: table titles truncated at a fixed 18 chars (`main.HumanResourc…`),
schema prefix wastes the title budget, edges share a single mid-x bend so
parallel edges and their labels stack on each other.

## Scope

1. **Infinite canvas** — remove `ScrollArea`; drag empty space pans; mouse wheel
   zooms toward the pointer; pinch/`zoom_delta` supported.
2. **Node drag** — grab a table card to move it; incident edge bboxes update
   live; spatial index rebuilt on drop.
3. **Auto fit** — fit-to-view on first graph ready, and debounced fit to the
   search-neighborhood subset after typing pauses.
4. **Minimap** — bottom-right overlay; click/drag navigates the viewport.
5. **Compact toolbar** — no `ER DIAGRAM` section label; left = context chips
   (tables · relationships · arranging), search + hop controls for large
   schemas; right = icon buttons (reset layout, design mode).
6. **Persisted layout** — manual node positions + zoom/pan per connection saved
   to `dbpro.native.er-layouts-v1` via `eframe::Storage`, restored on next open
   (falls back to auto-fit when absent).
7. **Visual polish** — two-line node header (name + `schema · n cols`) with
   pixel-measured truncation; edge lane offsets so parallel edges fan out;
   hovered/selected node keeps its edges accent, the rest dim.

## Out of scope

- Design Mode mutation semantics (existing panel kept, toggle restyled).
- PostgreSQL-specific diagram behavior (none — diagram is provider-agnostic;
  both providers feed the same `UiTableSummary`).
- Full force-directed layout (grid layout retained; manual positions override).

## Evidence plan

- Unit: node drag updates `world_rect` + incident edge bbox; wheel zoom anchors
  pointer; overrides survive worker rebuild; snapshot serde round-trip.
- `cargo build --release --locked -p db-pro-native`
- Captures via `capture` feature (`DB_PRO_CAPTURE_DIAGRAM` + seeded fixture) at
  1280×800, 1440×900, 1920×1080 dark + one light.
