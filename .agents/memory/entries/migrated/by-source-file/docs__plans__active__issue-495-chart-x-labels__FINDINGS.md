---
schema_version: 1
id: "MEM-SRC-721c5046fbd1cd05"
title: "Source mirror: docs/plans/active/issue-495-chart-x-labels/FINDINGS.md"
type: pattern
scope: project
status: needs-review
mirror_kind: full-file
mirror_wave: v2-by-source-file
source_path: "docs/plans/active/issue-495-chart-x-labels/FINDINGS.md"
source_sha256: "721c5046fbd1cd05dd6a9dc59d3331679bb0186b20d6f3c23cb6ee0428602045"
mirror_body_sha256: "721c5046fbd1cd05dd6a9dc59d3331679bb0186b20d6f3c23cb6ee0428602045"
truncated: false
created: "2026-10-24"
updated: "2026-10-24"
verified_on: unknown
confidence: low
tags: [migrated, by-source-file]
---
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
