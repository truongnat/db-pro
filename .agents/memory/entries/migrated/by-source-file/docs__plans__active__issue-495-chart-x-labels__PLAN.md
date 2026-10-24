---
schema_version: 1
id: "MEM-SRC-c302a4ca5ea977b4"
title: "Source mirror: docs/plans/active/issue-495-chart-x-labels/PLAN.md"
type: pattern
scope: project
status: needs-review
mirror_kind: full-file
mirror_wave: v2-by-source-file
source_path: "docs/plans/active/issue-495-chart-x-labels/PLAN.md"
source_sha256: "c302a4ca5ea977b4adbf8269fbe7b5cd4d0985390815f73a17ca5184ed2d28fb"
mirror_body_sha256: "c302a4ca5ea977b4adbf8269fbe7b5cd4d0985390815f73a17ca5184ed2d28fb"
truncated: false
created: "2026-10-24"
updated: "2026-10-24"
verified_on: unknown
confidence: low
tags: [migrated, by-source-file]
---
# #495 — Chart X-axis labels jump and pile at the left edge

## Goal
X-axis category labels must be stable frame-to-frame and positioned at the
data index of each label's first occurrence.

## Root cause
1. `unique_labels` was built via `HashSet` → nondeterministic iteration
   order → labels jump every frame.
2. Tick position divided the **unique-list** index by `points.len()` → when
   unique labels < point count, every label sat in the left sliver.

## Fix
`crates/ui/src/chart/mod.rs`:
- `x_axis_labels(points)` → `Vec<(String, usize)>` of first-appearance
  labels paired with their first data index (no new dep).
- `evenly_spread_indices(count, max)` → `max` indices spread over `0..count`
  including first and last; empty for `count == 0` / `max == 0`.
- Draw loop positions each label at `data_idx / points.len()` inside
  `chart_rect` (same frame as the bars it labels), capped at 8 labels.

## Tests
`chart/tests/mod_tests.rs` (new `mod tests` wiring): order/data-index pairs,
empty points, spread indices include first+last, bounds, `len < ticks` and
`count == 0` edges.
