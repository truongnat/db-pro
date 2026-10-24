---
schema_version: 1
id: "MEM-PLAN-issue-381-workbench-stale-contex-014a38e8dbc7"
title: "Plan archive: Issue 381 Workbench Stale Context"
type: bugfix
scope: project
status: needs-review
created: "2026-10-24"
updated: "2026-10-24"
verified_on: "unknown"
confidence: medium
tags: [migrated, docs-plans, bugfix, issue-381-workbench-stale-context]
mirror_wave: v1-plan-evidence-excerpt
mirror_excerpt_sha256: "014a38e8dbc73f3e9d5888ecbb30ee2f569f20fd123b01966e90763a0765aafd"
applies_to:
  repo: "db-pro"
  stack: "see sources"
  versions: "see plan state at migration"
sources:
  - "docs/plans/active/issue-381-workbench-stale-context/PLAN.md"
  - "docs/plans/active/issue-381-workbench-stale-context/FINDINGS.md"
  - "docs/plans/active/issue-381-workbench-stale-context/VERIFICATION.md"
related: []
supersedes: null
---

# Tóm tắt

Migrate từ `docs/plans/active/issue-381-workbench-stale-context/`. Loại suy luận: `bugfix`.

Entry tự động từ `docs/plans/`. Đối chiếu code hiện tại trước khi đặt `status: active`.

## Nội dung gốc (rút gọn)

## Source: `PLAN.md`

# Issue #381: open_schema_workbench keeps stale preview/name/confirmation when schema context changes

Status: VERIFIED-FIXED (already fixed in 9fe8706e)

No code change required. `open_schema_workbench` detects schema/table context change and calls `clear_plan_state()` + clears `name`.


## Source: `FINDINGS.md`

# Findings — #381

schema_workbench.rs:407-430 + `clear_plan_state` :313-321 (clears preview_sql/safety/fingerprint/error/connection/pending_plan/apply_confirmation).


## Source: `VERIFICATION.md`

# Verification — #381

`cargo check -p db-pro-app` — clean. `cargo test -p db-pro-app` — 593 passed, 0 failed (includes schema-workbench/event regressions).


