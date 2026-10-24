---
schema_version: 1
id: "MEM-SRC-76a8cfbb472216e1"
title: "Source mirror: docs/plans/active/issue-505-filtered-selection-arrows/VERIFICATION.md"
type: pattern
scope: project
status: needs-review
mirror_kind: full-file
mirror_wave: v2-by-source-file
source_path: "docs/plans/active/issue-505-filtered-selection-arrows/VERIFICATION.md"
source_sha256: "76a8cfbb472216e1fc85168bb1b8ab5816b948c6fcf542c492d41b4e2971ddd1"
mirror_body_sha256: "76a8cfbb472216e1fc85168bb1b8ab5816b948c6fcf542c492d41b4e2971ddd1"
truncated: false
created: "2026-10-24"
updated: "2026-10-24"
verified_on: unknown
confidence: low
tags: [migrated, by-source-file]
---
# Verification — #505

- `cargo check -p db-pro-app`: clean.
- `cargo test -p db-pro-app --lib`: 591 passed, 0 failed.
  - `result_grid::tests::keyboard_selection_reenters_projection_from_filtered_out_row` — new:
    ArrowDown → first visible row, ArrowUp → last, Home/End unchanged fallback.
  - `result_grid::tests::keyboard_selection_moves_within_projection_when_row_is_visible` — new.
  - `app::tests::grid_keyboard_navigation_*` — existing behavior preserved.
