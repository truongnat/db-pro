---
schema_version: 1
id: "MEM-SRC-1f5b5bec7f29bd1e"
title: "Source mirror: docs/plans/active/issue-505-filtered-selection-arrows/FINDINGS.md"
type: pattern
scope: project
status: needs-review
mirror_kind: full-file
mirror_wave: v2-by-source-file
source_path: "docs/plans/active/issue-505-filtered-selection-arrows/FINDINGS.md"
source_sha256: "1f5b5bec7f29bd1efd6fd983c95e84347e0caaff4a0d3ff1b9824b646e41b2dd"
mirror_body_sha256: "1f5b5bec7f29bd1efd6fd983c95e84347e0caaff4a0d3ff1b9824b646e41b2dd"
truncated: false
created: "2026-10-24"
updated: "2026-10-24"
verified_on: unknown
confidence: low
tags: [migrated, by-source-file]
---
# Findings — #505

- `grid_keyboard_selection` built a `HashMap<usize, usize>` per keypress and
  did `row_positions.get(&selected_row).copied().unwrap_or(0)` — a filtered-out
  selection silently became position 0, so ArrowDown yielded index 1 (skipping
  row 0) and ArrowUp saturated at `None`.
- Fix: `visible_rows.iter().position(..)` → `Option`; the Arrow arms branch on
  it explicitly. Also removes a per-keypress HashMap allocation.
- Existing tests in `app_tests.rs` (visible-row navigation) still pass; new
  coverage in `result_grid/tests/mod_tests.rs`.
