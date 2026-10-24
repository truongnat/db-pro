---
schema_version: 1
id: "MEM-PLAN-clippy-boolean-simplification-25e4febe00af"
title: "Plan archive: Clippy Boolean Simplification"
type: decision
scope: project
status: needs-review
created: "2026-10-24"
updated: "2026-10-24"
verified_on: "unknown"
confidence: medium
tags: [migrated, docs-plans, decision, clippy-boolean-simplification]
mirror_wave: v1-plan-evidence-excerpt
mirror_excerpt_sha256: "25e4febe00af6bf1c48f8985642dcd9ab01c6a03453f12073573fd6a95d2faec"
applies_to:
  repo: "db-pro"
  stack: "see sources"
  versions: "see plan state at migration"
sources:
  - "docs/plans/active/clippy-boolean-simplification/PLAN.md"
  - "docs/plans/active/clippy-boolean-simplification/FINDINGS.md"
  - "docs/plans/active/clippy-boolean-simplification/VERIFICATION.md"
related: []
supersedes: null
---

# Tóm tắt

Migrate từ `docs/plans/active/clippy-boolean-simplification/`. Loại suy luận: `decision`.

Entry tự động từ `docs/plans/`. Đối chiếu code hiện tại trước khi đặt `status: active`.

## Nội dung gốc (rút gọn)

## Source: `PLAN.md`

# Clippy Boolean Simplification Fix

## Problem
In `crates/app/src/runtime/operation_events.rs`, `take_current_connection_request` contained a nonminimal boolean expression `!pending.as_ref().is_some_and(...)` which triggered Clippy warning/error `-D clippy::nonminimal_bool`:

```text
error: this boolean expression can be simplified
  --> crates/app/src/app/../runtime/operation_events.rs:12:8
   |
12 | ...if !pending.as_ref().is_some_and(|(pending_id, _)| *pending_id == request_id.0) {
   |       ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ help: try: `pending.as_ref().is_none_or(|(pending_id, _)| *pending_id != request_id.0)`
```

## Solution
Simplify the boolean check to `pending.as_ref().is_none_or(|(pending_id, _)| *pending_id != request_id.0)`.

## Scope
- `crates/app/src/runtime/operation_events.rs`

## Verification
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo test -p db-pro-app`


## Source: `FINDINGS.md`

## Analysis
The nonminimal boolean `!pending.as_ref().is_some_and(...)` was introduced in a recent commit. Replacing it with `pending.as_ref().is_none_or(...)` satisfies Clippy while preserving identical logic semantics.


## Source: `VERIFICATION.md`

## Verification Results
- `cargo clippy --workspace --all-targets -- -D warnings`: PASS
- `cargo test -p db-pro-app`: PASS (675 passed)

