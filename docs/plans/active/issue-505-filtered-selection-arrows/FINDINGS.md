# Findings — #505

- `grid_keyboard_selection` built a `HashMap<usize, usize>` per keypress and
  did `row_positions.get(&selected_row).copied().unwrap_or(0)` — a filtered-out
  selection silently became position 0, so ArrowDown yielded index 1 (skipping
  row 0) and ArrowUp saturated at `None`.
- Fix: `visible_rows.iter().position(..)` → `Option`; the Arrow arms branch on
  it explicitly. Also removes a per-keypress HashMap allocation.
- Existing tests in `app_tests.rs` (visible-row navigation) still pass; new
  coverage in `result_grid/tests/mod_tests.rs`.
