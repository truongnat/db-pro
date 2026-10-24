---
schema_version: 1
id: "MEM-SRC-3ca53c48de25c321"
title: "Source mirror: docs/plans/completed/rc1-p2-result-grid-sort-perf/FINDINGS.md"
type: pattern
scope: project
status: needs-review
mirror_kind: full-file
mirror_wave: v2-by-source-file
source_path: "docs/plans/completed/rc1-p2-result-grid-sort-perf/FINDINGS.md"
source_sha256: "3ca53c48de25c321865fb5c028230dc18004344c20f97cd965aa989f30cde693"
mirror_body_sha256: "3ca53c48de25c321865fb5c028230dc18004344c20f97cd965aa989f30cde693"
truncated: false
created: "2026-10-24"
updated: "2026-10-24"
verified_on: unknown
confidence: low
tags: [migrated, by-source-file]
---
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
