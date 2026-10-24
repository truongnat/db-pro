---
schema_version: 1
id: "MEM-PLAN-issue-495-chart-x-labels-8364c21bc59f"
title: "Plan archive: Issue 495 Chart X Labels"
type: bugfix
scope: project
status: needs-review
created: "2026-10-24"
updated: "2026-10-24"
verified_on: "unknown"
confidence: medium
tags: [migrated, docs-plans, bugfix, issue-495-chart-x-labels]
mirror_wave: v1-plan-evidence-excerpt
mirror_excerpt_sha256: "8364c21bc59feb6c83db5778f2b96f4fdfd3f60fc3e6dabf2751546d42d36763"
applies_to:
  repo: "db-pro"
  stack: "see sources"
  versions: "see plan state at migration"
sources:
  - "docs/plans/active/issue-495-chart-x-labels/PLAN.md"
  - "docs/plans/active/issue-495-chart-x-labels/FINDINGS.md"
  - "docs/plans/active/issue-495-chart-x-labels/VERIFICATION.md"
related: []
supersedes: null
---

# Tóm tắt

Migrate từ `docs/plans/active/issue-495-chart-x-labels/`. Loại suy luận: `bugfix`.

Entry tự động từ `docs/plans/`. Đối chiếu code hiện tại trước khi đặt `status: active`.

## Nội dung gốc (rút gọn)

## Source: `PLAN.md`

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


## Source: `FINDINGS.md`

# #495 Findings

## Evidence
- `chart/mod.rs:86-91`: `points.iter().map(label).collect::<HashSet<_>>()`
  then `.into_iter().collect::<Vec>()` — HashSet order is randomized per
  process/iteration → label order unstable frame to frame.
- `:98-105`: `idx = i * step` indexes `unique_labels` but `px` divides by
  `points.len().max(1)` — with duplicates, unique count < point count so all
  labels compressed into the left fraction of the axis.
- `draw_bars` (`:195-210`) places bar i at `i / points.len() * width` —
  so the correct label position is the data index of first occurrence.

## Decision
`Vec` + linear `contains`-style dedup preserves first-appearance order with
no new dependency (IndexSet not present). Position from the recorded
`data_idx`. Also switched label positioning to `chart_rect` (was outer
`rect` including the 60px left margin) so labels align with bars.

## Severity
P2 — mislabeled/jittering axis; cosmetic but confusing.


## Source: `VERIFICATION.md`

# #495 Verification

- `cargo check -p db-pro-ui` → Finished, no errors (also `db-pro-app` for #510 caller).
- `cargo test -p db-pro-ui` → 687 passed, 0 failed.
- `cargo clippy -p db-pro-ui` → no warnings in touched files.
- `cargo fmt -p db-pro-ui -p db-pro-app -- --check` → clean.
- `bash scripts/check-ui-architecture.sh` → PASS.

## New regression tests
- `x_axis_labels_keep_first_appearance_order_and_data_index`
- `x_axis_labels_empty_points`
- `evenly_spread_indices_include_first_and_last`

