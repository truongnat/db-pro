---
schema_version: 1
id: "MEM-SRC-c133f786783651a3"
title: "Source mirror: docs/plans/active/issue-396-export-table-name/PLAN.md"
type: pattern
scope: project
status: needs-review
mirror_kind: full-file
mirror_wave: v2-by-source-file
source_path: "docs/plans/active/issue-396-export-table-name/PLAN.md"
source_sha256: "c133f786783651a306be06b16d28267a166de61e14b30eea0f47c908bc2a2a96"
mirror_body_sha256: "c133f786783651a306be06b16d28267a166de61e14b30eea0f47c908bc2a2a96"
truncated: false
created: "2026-10-24"
updated: "2026-10-24"
verified_on: unknown
confidence: low
tags: [migrated, by-source-file]
---
# Issue #396 — PLAN
- Issue: https://github.com/truongnat/db-pro/issues/396
- Fix: export_source_table_name only populates the export table name when the result is from a browsed table/view, preventing arbitrary query results from defaulting to an explorer-selected table.
