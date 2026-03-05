# Sprint 3 - S3-04 Result Grid Selection Upgrade

## Goal

Add deterministic multi-cell range selection and robust copy semantics for TSV/CSV from result grid.

## Delivered

- Added grid selection domain utilities:
  - `src/features/query/grid-selection.ts`
  - responsibilities:
    - range normalization
    - range hit-testing
    - selection summary
    - keyboard cell movement
    - TSV/CSV payload generation from selected ranges
- Rebuilt result table selection model:
  - `src/features/query/VirtualizedResultTable.tsx`
  - supports:
    - single-cell select (click)
    - rectangular range select (`Shift+click`, `Shift+Arrow`)
    - active-cell navigation (`Arrow` keys)
    - virtualization-compatible selection rendering
    - range copy shortcuts:
      - `Ctrl/Cmd+C`: copy TSV (selection without header, full page with header fallback)
      - `Ctrl/Cmd+Shift+C`: copy CSV (selection/header-aware)
- Updated result grid UX:
  - `src/features/query/QueryResultGrid.tsx`
  - added selection badge (`Selection RxC`)
  - added visible copy shortcut hint

## Clipboard Output Matrix

1. Single cell selected + `Ctrl/Cmd+C`:
   - output: single cell value (TSV text).
2. Range selected (e.g. 2x3) + `Ctrl/Cmd+C`:
   - output: 2 lines x 3 columns (TSV).
3. Range selected + `Ctrl/Cmd+Shift+C`:
   - output: CSV text with selected columns + selected rows.
4. No selection + `Ctrl/Cmd+C`:
   - output: full page TSV with header row.
5. No selection + `Ctrl/Cmd+Shift+C`:
   - output: full page CSV with header row.

## Validation

- `npm run -s build` => PASS
- `cargo check --manifest-path src-tauri/Cargo.toml` => PASS
- `cargo test --manifest-path src-tauri/Cargo.toml` => PASS
- `cargo run --manifest-path /Users/truongdq/Documents/GitHub/agentic-sdlc/Cargo.toml -- workflow check` => PASS (`checked=27 errors=0 warnings=0`)

Validated on 2026-03-06.
