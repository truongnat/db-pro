---
schema_version: 1
id: "MEM-PLAN-issue-458-sqlserver-introspect-m-8fc3125eb85a"
title: "Plan archive: Issue 458 Sqlserver Introspect Max Rows"
type: bugfix
scope: project
status: needs-review
created: "2026-10-24"
updated: "2026-10-24"
verified_on: "unknown"
confidence: medium
tags: [migrated, docs-plans, bugfix, issue-458-sqlserver-introspect-max-rows]
mirror_wave: v1-plan-evidence-excerpt
mirror_excerpt_sha256: "8fc3125eb85aff4086ae69d502b34da7914c81baefd62aa7642bf2edbae81f30"
applies_to:
  repo: "db-pro"
  stack: "see sources"
  versions: "see plan state at migration"
sources:
  - "docs/plans/active/issue-458-sqlserver-introspect-max-rows/PLAN.md"
  - "docs/plans/active/issue-458-sqlserver-introspect-max-rows/FINDINGS.md"
  - "docs/plans/active/issue-458-sqlserver-introspect-max-rows/VERIFICATION.md"
related: []
supersedes: null
---

# Tóm tắt

Migrate từ `docs/plans/active/issue-458-sqlserver-introspect-max-rows/`. Loại suy luận: `bugfix`.

Entry tự động từ `docs/plans/`. Đối chiếu code hiện tại trước khi đặt `status: active`.

## Nội dung gốc (rút gọn)

## Source: `PLAN.md`

# Issue #458 — PLAN
- Issue: https://github.com/truongnat/db-pro/issues/458
- Fix: SqlServerConnector::query_uncapped bypasses user query max_rows cap so catalog introspection never truncates large databases with >500 tables/columns.


## Source: `FINDINGS.md`

# Issue #458 — FINDINGS
- Issue: https://github.com/truongnat/db-pro/issues/458
- Fix: SqlServerConnector::query_uncapped bypasses user query max_rows cap so catalog introspection never truncates large databases with >500 tables/columns.


## Source: `VERIFICATION.md`

# Issue #458 — VERIFICATION
- Issue: https://github.com/truongnat/db-pro/issues/458
- Fix: SqlServerConnector::query_uncapped bypasses user query max_rows cap so catalog introspection never truncates large databases with >500 tables/columns.

