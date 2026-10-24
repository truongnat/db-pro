---
schema_version: 1
id: "MEM-SRC-be6a66b7d73a3938"
title: "Source mirror: docs/plans/active/issue-449-er-spatial-index-long-edges/CHECKLIST.md"
type: pattern
scope: project
status: needs-review
mirror_kind: full-file
mirror_wave: v2-by-source-file
source_path: "docs/plans/active/issue-449-er-spatial-index-long-edges/CHECKLIST.md"
source_sha256: "be6a66b7d73a39384f1469af9257cab46d101014a1af0b919e2b591368b21e35"
mirror_body_sha256: "be6a66b7d73a39384f1469af9257cab46d101014a1af0b919e2b591368b21e35"
truncated: false
created: "2026-10-24"
updated: "2026-10-24"
verified_on: unknown
confidence: low
tags: [migrated, by-source-file]
---
# Issue #449 — CHECKLIST
- Issue: https://github.com/truongnat/db-pro/issues/449
- Fix: insert_edge samples cells across the span when spanning > 32 cells rather than truncating from min_corner, ensuring far endpoints and mid-spans remain queryable.
