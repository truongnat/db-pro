---
schema_version: 1
id: "MEM-SRC-cbd770510419e6e0"
title: "Source mirror: docs/plans/active/issue-396-export-table-name/CHECKLIST.md"
type: pattern
scope: project
status: needs-review
mirror_kind: full-file
mirror_wave: v2-by-source-file
source_path: "docs/plans/active/issue-396-export-table-name/CHECKLIST.md"
source_sha256: "cbd770510419e6e0fe5915cb2ac6297978825716eed81c09d016eedb38818fac"
mirror_body_sha256: "cbd770510419e6e0fe5915cb2ac6297978825716eed81c09d016eedb38818fac"
truncated: false
created: "2026-10-24"
updated: "2026-10-24"
verified_on: unknown
confidence: low
tags: [migrated, by-source-file]
---
# Issue #396 — CHECKLIST
- Issue: https://github.com/truongnat/db-pro/issues/396
- Fix: export_source_table_name only populates the export table name when the result is from a browsed table/view, preventing arbitrary query results from defaulting to an explorer-selected table.
