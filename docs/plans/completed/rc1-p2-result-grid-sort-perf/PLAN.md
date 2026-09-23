# PLAN — Result Grid Sorting Allocation & Performance Optimization (#238)

State: COMPLETED

## Overview
When a user sorts or filters rows in the Result Grid (`crates/ui/src/result_grid_view.rs`), `filtered_sorted_indexes` is evaluated on every frame render.
Previously, `compare_ui_cells` called `cell_text(cell)` on every pair comparison during sorting, allocating millions of heap `String`s for large grids.
In addition, `compare_ui_cells` attempted three expensive `chrono` date/time parsers on every text string comparison without checking if the string resembles a date/timestamp.
Row filtering also allocated `String`s per cell for `to_lowercase()`.

This plan eliminates heap allocations during sorting and filtering comparisons, adds a fast date/time heuristic check, and verifies that sorting and filtering remain correct and fast.

## Steps
1. Create plan documentation in `docs/plans/active/rc1-p2-result-grid-sort-perf/`.
2. Implement `cell_text_as_str(&UiCell) -> &str` in `crates/ui/src/result_grid.rs` to provide borrowed string slices with 0 allocations.
3. Add `looks_like_iso_temporal` heuristic check in `crates/ui/src/result_grid.rs` to skip chrono parsing for non-date string values.
4. Refactor `compare_ui_cells` and `filtered_sorted_indexes` to use zero-allocation string comparison and ASCII case-insensitive window matching.
5. Add unit tests for `cell_text_as_str`, `looks_like_iso_temporal`, `compare_ui_cells`, and `filtered_sorted_indexes`.
6. Run quality gates and verify.
