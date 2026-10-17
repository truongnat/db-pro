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
