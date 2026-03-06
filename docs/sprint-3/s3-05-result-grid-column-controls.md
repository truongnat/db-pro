# Sprint 3 - S3-05 Result Grid Column Controls

## Goal

Add stable column controls for large result sets: resize and basic show/hide.

## Delivered

- Added dedicated column layout controller:
  - `src/features/query/useColumnLayoutController.ts`
  - responsibilities:
    - visibility state (`show/hide`)
    - width state with clamped bounds
    - schema-change safe pruning
    - reset/show-all flows
- Added column visibility panel component:
  - `src/features/query/ColumnVisibilityPanel.tsx`
  - features:
    - show/hide per column
    - hidden count indicator
    - show all
    - reset widths/layout
- Upgraded result grid integration:
  - `src/features/query/QueryResultGrid.tsx`
  - table now renders only visible columns
  - copy/export page follows visible columns
  - added layout hints/status updates
- Upgraded table renderer for resize:
  - `src/features/query/VirtualizedResultTable.tsx`
  - `colgroup` width application
  - drag-resize handle on header edge
  - pointer-based resize lifecycle (move/up/cancel)
- Added column utilities reused by selection/copy path:
  - `src/features/query/grid-selection.ts`

## Manual UX Check (1k+ rows)

1. Run paged query returning large dataset.
2. Drag header edge on multiple columns:
   - widths update immediately
   - horizontal scroll remains stable
   - no selection/copy regression
3. Hide several columns via panel:
   - table rerenders with only visible columns
   - copy page / export CSV uses visible column subset
4. Show all + reset widths:
   - layout returns to deterministic defaults.

## Validation

- `npm run -s build` => PASS
- `cargo check --manifest-path src-tauri/Cargo.toml` => PASS
- `cargo test --manifest-path src-tauri/Cargo.toml` => PASS
- `cargo run --manifest-path /Users/truongdq/Documents/GitHub/agentic-sdlc/Cargo.toml -- workflow check` => PASS (`checked=27 errors=0 warnings=0`)

Validated on 2026-03-06.
