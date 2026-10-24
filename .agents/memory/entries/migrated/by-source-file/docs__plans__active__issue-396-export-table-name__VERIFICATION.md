---
schema_version: 1
id: "MEM-SRC-b1bbe70fc884ea07"
title: "Source mirror: docs/plans/active/issue-396-export-table-name/VERIFICATION.md"
type: pattern
scope: project
status: needs-review
mirror_kind: full-file
mirror_wave: v2-by-source-file
source_path: "docs/plans/active/issue-396-export-table-name/VERIFICATION.md"
source_sha256: "b1bbe70fc884ea07a028d7fe4ab663da8a492e011903d937bc3ef14779a0224f"
mirror_body_sha256: "b1bbe70fc884ea07a028d7fe4ab663da8a492e011903d937bc3ef14779a0224f"
truncated: false
created: "2026-10-24"
updated: "2026-10-24"
verified_on: unknown
confidence: low
tags: [migrated, by-source-file]
---
# Issue #396 — VERIFICATION
- Issue: https://github.com/truongnat/db-pro/issues/396
- Fix: export_source_table_name only populates the export table name when the result is from a browsed table/view, preventing arbitrary query results from defaulting to an explorer-selected table.
