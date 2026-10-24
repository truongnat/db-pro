---
schema_version: 1
id: "MEM-PLAN-issue-357-refresh-ddl-edits-e4953e327480"
title: "Plan archive: Issue 357 Refresh Ddl Edits"
type: bugfix
scope: project
status: needs-review
created: "2026-10-24"
updated: "2026-10-24"
verified_on: "unknown"
confidence: medium
tags: [migrated, docs-plans, bugfix, issue-357-refresh-ddl-edits]
mirror_wave: v1-plan-evidence-excerpt
mirror_excerpt_sha256: "e4953e327480c8c850488d66adff09687e13e09abc883eb2f7d7b4b26ad33c78"
applies_to:
  repo: "db-pro"
  stack: "see sources"
  versions: "see plan state at migration"
sources:
  - "docs/plans/active/issue-357-refresh-ddl-edits/PLAN.md"
  - "docs/plans/active/issue-357-refresh-ddl-edits/FINDINGS.md"
  - "docs/plans/active/issue-357-refresh-ddl-edits/VERIFICATION.md"
related: []
supersedes: null
---

# Tóm tắt

Migrate từ `docs/plans/active/issue-357-refresh-ddl-edits/`. Loại suy luận: `bugfix`.

Entry tự động từ `docs/plans/`. Đối chiếu code hiện tại trước khi đặt `status: active`.

## Nội dung gốc (rút gọn)

## Source: `PLAN.md`

# Issue #357: Refresh DDL wipes edits without confirm; Open-in-Query can overwrite active buffer

Status: VERIFIED-FIXED (already fixed in edbcf7ae)

No code change required (file belongs to a sibling area anyway). Refresh now checks `table_ddl_dirty()` and raises `table_ddl_discard_confirm` instead of dropping the buffer; OpenInQuery uses `open_query_document_with` (new document).


## Source: `FINDINGS.md`

# Findings — #357

table/table_ddl_view.rs:49-63. Note: file is owned by a table-fixes sibling; verified read-only.


## Source: `VERIFICATION.md`

# Verification — #357

`cargo check -p db-pro-app` — clean. `cargo test -p db-pro-app` — 593 passed, 0 failed (includes schema-workbench/event regressions).


