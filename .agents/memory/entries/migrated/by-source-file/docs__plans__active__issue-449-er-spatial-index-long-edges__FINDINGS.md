---
schema_version: 1
id: "MEM-SRC-54143ccdb4356540"
title: "Source mirror: docs/plans/active/issue-449-er-spatial-index-long-edges/FINDINGS.md"
type: pattern
scope: project
status: needs-review
mirror_kind: full-file
mirror_wave: v2-by-source-file
source_path: "docs/plans/active/issue-449-er-spatial-index-long-edges/FINDINGS.md"
source_sha256: "54143ccdb435654089d5233e53c5c9e91e0069936d6e96ecfa349bff65391d94"
mirror_body_sha256: "54143ccdb435654089d5233e53c5c9e91e0069936d6e96ecfa349bff65391d94"
truncated: false
created: "2026-10-24"
updated: "2026-10-24"
verified_on: unknown
confidence: low
tags: [migrated, by-source-file]
---
# Issue #449 — FINDINGS
- Issue: https://github.com/truongnat/db-pro/issues/449
- Fix: insert_edge samples cells across the span when spanning > 32 cells rather than truncating from min_corner, ensuring far endpoints and mid-spans remain queryable.
