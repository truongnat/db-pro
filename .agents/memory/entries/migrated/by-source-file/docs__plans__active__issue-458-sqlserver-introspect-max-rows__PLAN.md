---
schema_version: 1
id: "MEM-SRC-547abe4b5772313f"
title: "Source mirror: docs/plans/active/issue-458-sqlserver-introspect-max-rows/PLAN.md"
type: pattern
scope: project
status: needs-review
mirror_kind: full-file
mirror_wave: v2-by-source-file
source_path: "docs/plans/active/issue-458-sqlserver-introspect-max-rows/PLAN.md"
source_sha256: "547abe4b5772313f4548aa1e3f3091b69fae49616e33be0ec2583cb35c89e899"
mirror_body_sha256: "547abe4b5772313f4548aa1e3f3091b69fae49616e33be0ec2583cb35c89e899"
truncated: false
created: "2026-10-24"
updated: "2026-10-24"
verified_on: unknown
confidence: low
tags: [migrated, by-source-file]
---
# Issue #458 — PLAN
- Issue: https://github.com/truongnat/db-pro/issues/458
- Fix: SqlServerConnector::query_uncapped bypasses user query max_rows cap so catalog introspection never truncates large databases with >500 tables/columns.
