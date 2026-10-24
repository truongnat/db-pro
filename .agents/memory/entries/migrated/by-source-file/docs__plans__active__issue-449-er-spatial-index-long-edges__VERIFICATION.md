---
schema_version: 1
id: "MEM-SRC-453b6531d5ecfc3c"
title: "Source mirror: docs/plans/active/issue-449-er-spatial-index-long-edges/VERIFICATION.md"
type: pattern
scope: project
status: needs-review
mirror_kind: full-file
mirror_wave: v2-by-source-file
source_path: "docs/plans/active/issue-449-er-spatial-index-long-edges/VERIFICATION.md"
source_sha256: "453b6531d5ecfc3cd1797451a730129e5784f49eb790aa5ae55744616ae2d920"
mirror_body_sha256: "453b6531d5ecfc3cd1797451a730129e5784f49eb790aa5ae55744616ae2d920"
truncated: false
created: "2026-10-24"
updated: "2026-10-24"
verified_on: unknown
confidence: low
tags: [migrated, by-source-file]
---
# Issue #449 — VERIFICATION
- Issue: https://github.com/truongnat/db-pro/issues/449
- Fix: insert_edge samples cells across the span when spanning > 32 cells rather than truncating from min_corner, ensuring far endpoints and mid-spans remain queryable.
