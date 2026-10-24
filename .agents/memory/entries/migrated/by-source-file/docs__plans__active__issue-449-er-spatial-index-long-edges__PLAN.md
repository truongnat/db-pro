---
schema_version: 1
id: "MEM-SRC-a304a4b2a1a9850a"
title: "Source mirror: docs/plans/active/issue-449-er-spatial-index-long-edges/PLAN.md"
type: pattern
scope: project
status: needs-review
mirror_kind: full-file
mirror_wave: v2-by-source-file
source_path: "docs/plans/active/issue-449-er-spatial-index-long-edges/PLAN.md"
source_sha256: "a304a4b2a1a9850ab1f166300ac3e3801790df84b5cb596faced581396432037"
mirror_body_sha256: "a304a4b2a1a9850ab1f166300ac3e3801790df84b5cb596faced581396432037"
truncated: false
created: "2026-10-24"
updated: "2026-10-24"
verified_on: unknown
confidence: low
tags: [migrated, by-source-file]
---
# Issue #449 — PLAN
- Issue: https://github.com/truongnat/db-pro/issues/449
- Fix: insert_edge samples cells across the span when spanning > 32 cells rather than truncating from min_corner, ensuring far endpoints and mid-spans remain queryable.
