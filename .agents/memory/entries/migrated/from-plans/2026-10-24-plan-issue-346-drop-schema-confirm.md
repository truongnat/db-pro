---
schema_version: 1
id: "MEM-PLAN-issue-346-drop-schema-confirm-70af7e940c0b"
title: "Plan archive: Issue 346 Drop Schema Confirm"
type: bugfix
scope: project
status: needs-review
created: "2026-10-24"
updated: "2026-10-24"
verified_on: "unknown"
confidence: medium
tags: [migrated, docs-plans, bugfix, issue-346-drop-schema-confirm]
mirror_wave: v1-plan-evidence-excerpt
mirror_excerpt_sha256: "70af7e940c0b1c164fd746124764c0fd0b0ef06631fcd71a6f52c56f912a0d8f"
applies_to:
  repo: "db-pro"
  stack: "see sources"
  versions: "see plan state at migration"
sources:
  - "docs/plans/active/issue-346-drop-schema-confirm/PLAN.md"
  - "docs/plans/active/issue-346-drop-schema-confirm/FINDINGS.md"
  - "docs/plans/active/issue-346-drop-schema-confirm/VERIFICATION.md"
related: []
supersedes: null
---

# Tóm tắt

Migrate từ `docs/plans/active/issue-346-drop-schema-confirm/`. Loại suy luận: `bugfix`.

Entry tự động từ `docs/plans/`. Đối chiếu code hiện tại trước khi đặt `status: active`.

## Nội dung gốc (rút gọn)

## Source: `PLAN.md`

# Issue #346: plan_drop_schema opens confirm even when plan fails / SQL empty

Status: VERIFIED-FIXED (already fixed in 9fe8706e)

No code change required. `plan_drop_schema` (schema_workbench_actions.rs:56-63) now sets `apply_confirmation = true` only when `preview_error.is_none() && !preview_sql.trim().is_empty()`.


## Source: `FINDINGS.md`

# Findings — #346

Confirmed the guard exists at schema_workbench_actions.rs:59-63. Failed plans set `preview_error` and clear `preview_sql` in `plan_workbench_action`, so the dialog cannot open for failed/empty plans.


## Source: `VERIFICATION.md`

# Verification — #346

`cargo check -p db-pro-app` — clean. `cargo test -p db-pro-app` — 593 passed, 0 failed (includes schema-workbench/event regressions).


