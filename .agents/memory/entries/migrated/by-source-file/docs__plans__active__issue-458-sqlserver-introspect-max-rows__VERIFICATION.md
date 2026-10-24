---
schema_version: 1
id: "MEM-SRC-589e3c7b94c57c10"
title: "Source mirror: docs/plans/active/issue-458-sqlserver-introspect-max-rows/VERIFICATION.md"
type: pattern
scope: project
status: needs-review
mirror_kind: full-file
mirror_wave: v2-by-source-file
source_path: "docs/plans/active/issue-458-sqlserver-introspect-max-rows/VERIFICATION.md"
source_sha256: "589e3c7b94c57c1032597c2381d771ac559f48fd27ae056c0d4270fc66dd86f8"
mirror_body_sha256: "589e3c7b94c57c1032597c2381d771ac559f48fd27ae056c0d4270fc66dd86f8"
truncated: false
created: "2026-10-24"
updated: "2026-10-24"
verified_on: unknown
confidence: low
tags: [migrated, by-source-file]
---
# Issue #458 — VERIFICATION
- Issue: https://github.com/truongnat/db-pro/issues/458
- Fix: SqlServerConnector::query_uncapped bypasses user query max_rows cap so catalog introspection never truncates large databases with >500 tables/columns.
