---
schema_version: 1
id: "MEM-PLAN-issue-350-open-sql-connection-0a9d25fac4ee"
title: "Plan archive: Issue 350 Open Sql Connection"
type: bugfix
scope: project
status: needs-review
created: "2026-10-24"
updated: "2026-10-24"
verified_on: "unknown"
confidence: medium
tags: [migrated, docs-plans, bugfix, issue-350-open-sql-connection]
mirror_wave: v1-plan-evidence-excerpt
mirror_excerpt_sha256: "0a9d25fac4ee3545d9430b17a7e7f6629e9bc04c4fc66b76e1855a2403dd04a8"
applies_to:
  repo: "db-pro"
  stack: "see sources"
  versions: "see plan state at migration"
sources:
  - "docs/plans/active/issue-350-open-sql-connection/PLAN.md"
  - "docs/plans/active/issue-350-open-sql-connection/FINDINGS.md"
  - "docs/plans/active/issue-350-open-sql-connection/VERIFICATION.md"
related: []
supersedes: null
---

# Tóm tắt

Migrate từ `docs/plans/active/issue-350-open-sql-connection/`. Loại suy luận: `bugfix`.

Entry tự động từ `docs/plans/`. Đối chiếu code hiện tại trước khi đặt `status: active`.

## Nội dung gốc (rút gọn)

## Source: `PLAN.md`

# Issue #350: Open in SQL Editor uses query context connection, not plan's connection

Status: VERIFIED-FIXED (already fixed in 9fe8706e)

No code change required. `OpenSql` handler binds the new document to `workbench.preview_connection_id` (captured at plan time in `run_plan`) and sets the document schema.


## Source: `FINDINGS.md`

# Findings — #350

schema_workbench_actions.rs:21-38. `preview_connection_id` stored at :102-103. Handler calls `set_document_connection(index, Some(connection_id))` and `set_document_schema`. The handling lives in schema_workbench_actions.rs (my file), not events_query_dispatch.


## Source: `VERIFICATION.md`

# Verification — #350

`cargo check -p db-pro-app` — clean. `cargo test -p db-pro-app` — 593 passed, 0 failed (includes schema-workbench/event regressions).


