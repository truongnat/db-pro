---
schema_version: 1
id: "MEM-SRC-75d58cf3df25bd89"
title: "Source mirror: docs/plans/active/clippy-boolean-simplification/PLAN.md"
type: pattern
scope: project
status: needs-review
mirror_kind: full-file
mirror_wave: v2-by-source-file
source_path: "docs/plans/active/clippy-boolean-simplification/PLAN.md"
source_sha256: "75d58cf3df25bd893e84c81514a40a15c2a26dfc5af3052df8dc480101914ae5"
mirror_body_sha256: "75d58cf3df25bd893e84c81514a40a15c2a26dfc5af3052df8dc480101914ae5"
truncated: false
created: "2026-10-24"
updated: "2026-10-24"
verified_on: unknown
confidence: low
tags: [migrated, by-source-file]
---
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
