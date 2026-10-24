---
schema_version: 1
id: "MEM-PLAN-issue-440-stale-schema-loaded-c86661a28ef1"
title: "Plan archive: Issue 440 Stale Schema Loaded"
type: bugfix
scope: project
status: needs-review
created: "2026-10-24"
updated: "2026-10-24"
verified_on: "unknown"
confidence: medium
tags: [migrated, docs-plans, bugfix, issue-440-stale-schema-loaded]
mirror_wave: v1-plan-evidence-excerpt
mirror_excerpt_sha256: "c86661a28ef1697ce8a46cda850040e1a855b4ca56fa328e7986d03657743a1d"
applies_to:
  repo: "db-pro"
  stack: "see sources"
  versions: "see plan state at migration"
sources:
  - "docs/plans/active/issue-440-stale-schema-loaded/PLAN.md"
  - "docs/plans/active/issue-440-stale-schema-loaded/FINDINGS.md"
  - "docs/plans/active/issue-440-stale-schema-loaded/VERIFICATION.md"
related: []
supersedes: null
---

# Tóm tắt

Migrate từ `docs/plans/active/issue-440-stale-schema-loaded/`. Loại suy luận: `bugfix`.

Entry tự động từ `docs/plans/`. Đối chiếu code hiện tại trước khi đặt `status: active`.

## Nội dung gốc (rút gọn)

## Source: `PLAN.md`

# Issue #440: Stale SchemaLoaded response applies after disconnect

Status: FIXED in this change

Changed `on_schema_loaded` guard from `is_some_and(|e| e != request_id)` to `schema_request != Some(request_id)` — a response whose pending request was cleared by disconnect/reset is now dropped.


## Source: `FINDINGS.md`

# Findings — #440

`schema_events.rs:48-53` previously only rejected when a *different* request id was pending; `None` (post-disconnect) fell through and applied the stale schema.
`request_schema_introspection` (app/mod.rs:434) always sets `schema_request` before dispatch, so a legit response always has `Some(request_id)` pending. `handle_schema_request_failure` already used exact-match (`!= Some`).
Updated two app tests that sent SchemaLoaded without a pending request. Added `schema_loaded_after_disconnect_is_dropped` regression test.
Same `is_some_and` shape exists in `connection/connection_events.rs:75-80` (`on_connected`) — outside my file ownership; flagged to parent.


## Source: `VERIFICATION.md`

# Verification — #440

`cargo check -p db-pro-app` — clean. `cargo test -p db-pro-app` — 593 passed, 0 failed (includes schema-workbench/event regressions).
New regression test `schema_loaded_after_disconnect_is_dropped` passes.


