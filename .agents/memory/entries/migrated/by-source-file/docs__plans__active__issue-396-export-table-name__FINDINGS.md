---
schema_version: 1
id: "MEM-SRC-8baf9a8f1a0ec613"
title: "Source mirror: docs/plans/active/issue-396-export-table-name/FINDINGS.md"
type: pattern
scope: project
status: needs-review
mirror_kind: full-file
mirror_wave: v2-by-source-file
source_path: "docs/plans/active/issue-396-export-table-name/FINDINGS.md"
source_sha256: "8baf9a8f1a0ec6138e80b8ddaeaa1f74c243844f9113d32aaef9aac59587fdac"
mirror_body_sha256: "8baf9a8f1a0ec6138e80b8ddaeaa1f74c243844f9113d32aaef9aac59587fdac"
truncated: false
created: "2026-10-24"
updated: "2026-10-24"
verified_on: unknown
confidence: low
tags: [migrated, by-source-file]
---
# Issue #396 — FINDINGS
- Issue: https://github.com/truongnat/db-pro/issues/396
- Fix: export_source_table_name only populates the export table name when the result is from a browsed table/view, preventing arbitrary query results from defaulting to an explorer-selected table.
