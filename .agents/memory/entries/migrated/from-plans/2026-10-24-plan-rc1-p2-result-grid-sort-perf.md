---
schema_version: 1
id: "MEM-PLAN-rc1-p2-result-grid-sort-perf-5d196ca08a04"
title: "Plan archive: Rc1 P2 Result Grid Sort Perf"
type: decision
scope: project
status: needs-review
created: "2026-10-24"
updated: "2026-10-24"
verified_on: "unknown"
confidence: medium
tags: [migrated, docs-plans, decision, rc1-p2-result-grid-sort-perf]
mirror_wave: v1-plan-evidence-excerpt
mirror_excerpt_sha256: "5d196ca08a04f0a90ee88585112d0f8c0caea58dbc471501c3221b91795fe46b"
applies_to:
  repo: "db-pro"
  stack: "see sources"
  versions: "see plan state at migration"
sources:
  - "docs/plans/completed/rc1-p2-result-grid-sort-perf/PLAN.md"
  - "docs/plans/completed/rc1-p2-result-grid-sort-perf/FINDINGS.md"
  - "docs/plans/completed/rc1-p2-result-grid-sort-perf/VERIFICATION.md"
related: []
supersedes: null
---

# Tóm tắt

Migrate từ `docs/plans/completed/rc1-p2-result-grid-sort-perf/`. Loại suy luận: `decision`.

Entry tự động từ `docs/plans/`. Đối chiếu code hiện tại trước khi đặt `status: active`.

## Nội dung gốc (rút gọn)

## Source: `PLAN.md`

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


## Source: `FINDINGS.md`

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


## Source: `VERIFICATION.md`

# VERIFICATION — Result Grid Sorting Allocation & Performance Optimization (#238)

## Verification Plan

### Automated Verification
- Run `cargo test -p db-pro-ui` to verify all unit tests pass, including new tests for `cell_text_as_str`, date heuristics, typed sorting, and row filtering.
- Run workspace tests `cargo test -p db-pro-core -p db-pro-infrastructure -p db-pro-runtime -p db-pro-ui -p db-pro-native`.
- Run clippy and check gates.

### Manual / Structural Verification
- Confirm `cell_text_as_str` returns string slices without heap allocations.
- Confirm `compare_ui_cells` handles `Null`, `Boolean`, `Number`, `Text`, `Json`, `Bytes` identically to previous behavior.
- Confirm non-date strings bypass chrono parsers.

