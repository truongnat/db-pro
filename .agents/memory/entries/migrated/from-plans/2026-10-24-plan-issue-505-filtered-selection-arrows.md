---
schema_version: 1
id: "MEM-PLAN-issue-505-filtered-selection-arr-fe052d49e522"
title: "Plan archive: Issue 505 Filtered Selection Arrows"
type: bugfix
scope: project
status: needs-review
created: "2026-10-24"
updated: "2026-10-24"
verified_on: "unknown"
confidence: medium
tags: [migrated, docs-plans, bugfix, issue-505-filtered-selection-arrows]
mirror_wave: v1-plan-evidence-excerpt
mirror_excerpt_sha256: "fe052d49e52288620010c199ae6216570b1af3e34932791e25e03c0836aca382"
applies_to:
  repo: "db-pro"
  stack: "see sources"
  versions: "see plan state at migration"
sources:
  - "docs/plans/active/issue-505-filtered-selection-arrows/PLAN.md"
  - "docs/plans/active/issue-505-filtered-selection-arrows/FINDINGS.md"
  - "docs/plans/active/issue-505-filtered-selection-arrows/VERIFICATION.md"
related: []
supersedes: null
---

# Tóm tắt

Migrate từ `docs/plans/active/issue-505-filtered-selection-arrows/`. Loại suy luận: `bugfix`.

Entry tự động từ `docs/plans/`. Đối chiếu code hiện tại trước khi đặt `status: active`.

## Nội dung gốc (rút gọn)

## Source: `PLAN.md`

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


## Source: `FINDINGS.md`

# Findings — #505

- `grid_keyboard_selection` built a `HashMap<usize, usize>` per keypress and
  did `row_positions.get(&selected_row).copied().unwrap_or(0)` — a filtered-out
  selection silently became position 0, so ArrowDown yielded index 1 (skipping
  row 0) and ArrowUp saturated at `None`.
- Fix: `visible_rows.iter().position(..)` → `Option`; the Arrow arms branch on
  it explicitly. Also removes a per-keypress HashMap allocation.
- Existing tests in `app_tests.rs` (visible-row navigation) still pass; new
  coverage in `result_grid/tests/mod_tests.rs`.


## Source: `VERIFICATION.md`

# Verification — #505

- `cargo check -p db-pro-app`: clean.
- `cargo test -p db-pro-app --lib`: 591 passed, 0 failed.
  - `result_grid::tests::keyboard_selection_reenters_projection_from_filtered_out_row` — new:
    ArrowDown → first visible row, ArrowUp → last, Home/End unchanged fallback.
  - `result_grid::tests::keyboard_selection_moves_within_projection_when_row_is_visible` — new.
  - `app::tests::grid_keyboard_navigation_*` — existing behavior preserved.

