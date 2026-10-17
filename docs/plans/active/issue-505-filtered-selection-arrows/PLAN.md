# #505 — Arrow keys after selection leaves the visible projection

## Goal
When `selected_row` is filtered/sorted out of `visible_rows`,
`grid_keyboard_selection` defaulted `row_position` to 0 → ArrowDown returned
`visible_rows[1]` (skipping the first row) and ArrowUp returned `None`.

## Scope
- `result_grid/mod.rs`: `row_position` is now `Option<usize>` from a
  `position()` lookup (drops the per-keypress `HashMap`).
  - ArrowDown: `Some(p)` → next row (clamped); `None` → `visible_rows[0]`.
  - ArrowUp: `Some(p)` → previous row; `None` → `visible_rows.last()`.
  - Left/Right/Home/End: unchanged fallback (`unwrap_or(0)` → first visible row,
    column navigation only).

## Non-goals
- No wrap-around navigation; clamping semantics preserved.

## Acceptance
- ArrowDown with filtered-out selection → first visible row; ArrowUp → last.
- `cargo test -p db-pro-app` + `cargo check -p db-pro-app`.
