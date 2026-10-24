---
schema_version: 1
id: "MEM-SRC-586921b81c64f13b"
title: "Source mirror: docs/plans/active/issue-458-sqlserver-introspect-max-rows/FINDINGS.md"
type: pattern
scope: project
status: needs-review
mirror_kind: full-file
mirror_wave: v2-by-source-file
source_path: "docs/plans/active/issue-458-sqlserver-introspect-max-rows/FINDINGS.md"
source_sha256: "586921b81c64f13b1ddc87f0ba80ee0e11021bc1a670175291e38ac707e599c6"
mirror_body_sha256: "586921b81c64f13b1ddc87f0ba80ee0e11021bc1a670175291e38ac707e599c6"
truncated: false
created: "2026-10-24"
updated: "2026-10-24"
verified_on: unknown
confidence: low
tags: [migrated, by-source-file]
---
# Issue #458 — FINDINGS
- Issue: https://github.com/truongnat/db-pro/issues/458
- Fix: SqlServerConnector::query_uncapped bypasses user query max_rows cap so catalog introspection never truncates large databases with >500 tables/columns.
