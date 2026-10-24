---
schema_version: 1
id: "MEM-SRC-ad5b7922936255c7"
title: "Source mirror: docs/plans/active/issue-458-sqlserver-introspect-max-rows/CHECKLIST.md"
type: pattern
scope: project
status: needs-review
mirror_kind: full-file
mirror_wave: v2-by-source-file
source_path: "docs/plans/active/issue-458-sqlserver-introspect-max-rows/CHECKLIST.md"
source_sha256: "ad5b7922936255c77ce088bea8bc3e2c3bc41440b563c936ec62f0d9c880ec8b"
mirror_body_sha256: "ad5b7922936255c77ce088bea8bc3e2c3bc41440b563c936ec62f0d9c880ec8b"
truncated: false
created: "2026-10-24"
updated: "2026-10-24"
verified_on: unknown
confidence: low
tags: [migrated, by-source-file]
---
# Issue #458 — CHECKLIST
- Issue: https://github.com/truongnat/db-pro/issues/458
- Fix: SqlServerConnector::query_uncapped bypasses user query max_rows cap so catalog introspection never truncates large databases with >500 tables/columns.
