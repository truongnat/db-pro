# Verification — #505

- `cargo check -p db-pro-app`: clean.
- `cargo test -p db-pro-app --lib`: 591 passed, 0 failed.
  - `result_grid::tests::keyboard_selection_reenters_projection_from_filtered_out_row` — new:
    ArrowDown → first visible row, ArrowUp → last, Home/End unchanged fallback.
  - `result_grid::tests::keyboard_selection_moves_within_projection_when_row_is_visible` — new.
  - `app::tests::grid_keyboard_navigation_*` — existing behavior preserved.
