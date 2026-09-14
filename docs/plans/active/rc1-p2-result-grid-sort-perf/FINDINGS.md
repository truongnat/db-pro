# FINDINGS — Result Grid Sorting Allocation & Performance Optimization (#238)

## Problem Evidence
- `filtered_sorted_indexes` (`crates/ui/src/result_grid.rs:97`) runs on every frame during grid render (`result_grid_view.rs:76`).
- During sorting, `compare_ui_cells` called `cell_text(cell)`, allocating a new `String` for every cell comparison. For N rows, sorting requires O(N log N) comparisons, creating millions of string allocations per frame on 100k rows.
- `compare_ui_cells` attempted `DateTime::parse_from_rfc3339`, `NaiveDateTime::parse_from_str`, and `NaiveDate::parse_from_str` on every pair of `UiCell::Text` values, even for plain text strings like `"customer-123"`.
- `filtered_sorted_indexes` called `cell_text(cell).to_lowercase().contains(&filter)` for every cell, creating two `String` allocations per cell per frame during filtering.

## Solution Architecture
1. `cell_text_as_str(&UiCell) -> &str` borrows string representations from `UiCell` variants (`UiCell::Null` -> `"NULL"`, `UiCell::Boolean(true)` -> `"true"`, `UiCell::Boolean(false)` -> `"false"`, text/number/json/bytes -> `value.as_str()`).
2. `looks_like_iso_temporal` checks `bytes.len() >= 10 && bytes[0].is_ascii_digit() && bytes[4] == b'-'` before attempting expensive `chrono` date/time parsers.
3. `cell_contains_filter` uses zero-allocation ASCII case-insensitive window matching (`s.as_bytes().windows(len).any(|w| w.eq_ignore_ascii_case(needle))`).
4. `compare_ui_cells` uses `cell_text_as_str` for fallback cell comparisons, avoiding all heap allocations.
