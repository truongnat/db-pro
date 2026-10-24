---
schema_version: 1
id: "MEM-PLAN-issue-371-history-clear-confirm-d5662fd9c393"
title: "Plan archive: Issue 371 History Clear Confirm"
type: bugfix
scope: project
status: needs-review
created: "2026-10-24"
updated: "2026-10-24"
verified_on: "unknown"
confidence: medium
tags: [migrated, docs-plans, bugfix, issue-371-history-clear-confirm]
mirror_wave: v1-plan-evidence-excerpt
mirror_excerpt_sha256: "d5662fd9c3934562bfe8764e91f6017d9b2124825635e77551f2a8f53004b2a0"
applies_to:
  repo: "db-pro"
  stack: "see sources"
  versions: "see plan state at migration"
sources:
  - "docs/plans/active/issue-371-history-clear-confirm/PLAN.md"
  - "docs/plans/active/issue-371-history-clear-confirm/FINDINGS.md"
  - "docs/plans/active/issue-371-history-clear-confirm/VERIFICATION.md"
related: []
supersedes: null
---

# Tóm tắt

Migrate từ `docs/plans/active/issue-371-history-clear-confirm/`. Loại suy luận: `bugfix`.

Entry tự động từ `docs/plans/`. Đối chiếu code hiện tại trước khi đặt `status: active`.

## Nội dung gốc (rút gọn)

## Source: `PLAN.md`

# Issue #371: DDL history Clear has no confirm; Open-in-editor spams documents

Status: VERIFIED-FIXED (already fixed in 9fe8706e)

No code change required. Two-step confirm (`history_clear_confirm` + Confirm clear / Keep history) in draw_execution_history; `OpenSqlAsQuery` reuses a document already holding the same SQL or an empty active doc.


## Source: `FINDINGS.md`

# Findings — #371

schema_workbench_secondary_view.rs:130-162, schema_workbench.rs:522-553.


## Source: `VERIFICATION.md`

# Verification — #371

`cargo check -p db-pro-app` — clean. `cargo test -p db-pro-app` — 593 passed, 0 failed (includes schema-workbench/event regressions).


