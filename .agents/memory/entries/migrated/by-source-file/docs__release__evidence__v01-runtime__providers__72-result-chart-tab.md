---
schema_version: 1
id: "MEM-SRC-e3c67635f4a76746"
title: "Source mirror: docs/release/evidence/v01-runtime/providers/72-result-chart-tab.md"
type: pattern
scope: project
status: needs-review
mirror_kind: full-file
mirror_wave: v2-by-source-file
source_path: "docs/release/evidence/v01-runtime/providers/72-result-chart-tab.md"
source_sha256: "e3c67635f4a76746e403a6a94c956fcd21cc8d6948603385e44f57e2489a5342"
mirror_body_sha256: "e3c67635f4a76746e403a6a94c956fcd21cc8d6948603385e44f57e2489a5342"
truncated: false
created: "2026-10-24"
updated: "2026-10-24"
verified_on: unknown
confidence: low
tags: [migrated, by-source-file]
---
# Result chart tab mounted (#248 progress)

## Claim

The existing chart engine is reachable from the query output surface: a **Chart** tab
stores `ChartConfig` on `QueryDocument` (survives tab switches) and renders through
`ChartEngine` / `ChartRenderer`.

## Landed

- `OutputTab::Chart` in query + bottom navigation panels
- `QueryDocument.chart_config`
- Type / X / Y / aggregation controls; numeric-only Y picker
- Unit tests: numeric column detection, downsample, config clone

## Not yet (issue stays open)

- Image export / copy chart
- Aggregate-from-database query generation
- 10k-row responsiveness evidence beyond `max_points` downsample unit test
- Full acceptance audit for #248

## Tests

```bash
cargo test -p db-pro-ui chart_view --lib
```
