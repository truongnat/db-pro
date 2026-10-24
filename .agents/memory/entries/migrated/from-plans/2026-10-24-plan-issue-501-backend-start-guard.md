---
schema_version: 1
id: "MEM-PLAN-issue-501-backend-start-guard-e82e297c67cc"
title: "Plan archive: Issue 501 Backend Start Guard"
type: bugfix
scope: project
status: needs-review
created: "2026-10-24"
updated: "2026-10-24"
verified_on: "unknown"
confidence: medium
tags: [migrated, docs-plans, bugfix, issue-501-backend-start-guard]
mirror_wave: v1-plan-evidence-excerpt
mirror_excerpt_sha256: "e82e297c67ccc9b9396a74779623e593ec4d4cc89b97848177327c231d8f5a1c"
applies_to:
  repo: "db-pro"
  stack: "see sources"
  versions: "see plan state at migration"
sources:
  - "docs/plans/active/issue-501-backend-start-guard/PLAN.md"
  - "docs/plans/active/issue-501-backend-start-guard/FINDINGS.md"
  - "docs/plans/active/issue-501-backend-start-guard/VERIFICATION.md"
related: []
supersedes: null
---

# Tóm tắt

Migrate từ `docs/plans/active/issue-501-backend-start-guard/`. Loại suy luận: `bugfix`.

Entry tự động từ `docs/plans/`. Đối chiếu code hiện tại trước khi đặt `status: active`.

## Nội dung gốc (rút gọn)

## Source: `PLAN.md`

# Issue #501 — cancel/terminate must verify `backend_start`

## Problem
`ConfirmTerminate`/`ConfirmCancelBackend` carried only the pid. Between the
session snapshot and the confirm click the pid can be recycled by a new client,
so the signal hits the wrong session.

## Fix
Carry the snapshot's `backend_start` (`pg_stat_activity.backend_start::text`,
already selected in `LIST_SESSIONS_SQL` → `MonitorSession.backend_start`)
through the whole pipeline and verify it inside the signal statement itself:

```
SELECT pg_terminate_backend(a.pid)          -- or pg_cancel_backend
FROM pg_catalog.pg_stat_activity a
WHERE a.pid = $1 AND a.backend_start = $2::timestamptz
```

- Atomic check-and-signal: no re-query window. Comparing as `timestamptz` (not
  text) avoids false mismatches when the TimeZone differs between connections.
- No row / `false` → `Ok(false)`, surfaced via `MonitoringActionCompleted` as
  before.
- App layer refuses `backend_start: None` before dispatch ("session start time
  is unknown — refresh and retry"); service rejects an empty string so no
  caller can bypass the guard.

## Plumbing (signature change end to end)
`MonitoringConfirmationAction::{ConfirmTerminate,ConfirmCancelBackend}` →
`UiCommand::Monitoring{Cancel,Terminate}Backend` (new `expected_backend_start:
String`) → `translate_cmd` → `RuntimeCommand` → `MonitoringApi` →
`MonitoringService::{cancel,terminate}_backend` →
`MonitoringPort::{cancel,terminate}_backend(handle, pid, expected_backend_start)`
→ `PostgresMonitoringPort` guarded SQL; SQLite keeps returning `Unsupported`.

## Files
- `crates/app/src/monitoring/monitoring_confirmation_view.rs`
- `crates/app/src/monitoring/monitoring_activity_view.rs`
- `crates/app/src/monitoring/tests/monitoring_activity_view_tests.rs`
- `crates/core/src/ports/monitoring.rs`
- `crates/core/src/application/monitoring_service.rs`
- `crates/core/src/application/tests/monitoring_service_tests.rs`
- `crates/infrastructure/src/postgres/monitoring.rs`
- `crates/infrastructure/src/postgres/tests/monitoring_tests.rs`
- `crates/infrastructure/src/sqlite/monitoring.rs`
- `crates/ui/src/runtime/runtime_protocol.rs`
- `crates/runtime/src/api.rs`
- `crates/runtime/src/worker.rs`
- `crates/native-app/src/translate_cmd.rs`


## Source: `FINDINGS.md`

# Findings — #501

- `MonitorSession.backend_start` already exists and `LIST_SESSIONS_SQL` already
  returns `backend_start::text` — no query change needed upstream.
- Race surface: snapshot → user reads dialog → clicks confirm → runtime spawns
  `pg_terminate_backend(pid)`. A fast client reconnect can reuse the pid.
- Server-side guard is the only correct level: any UI/service-side re-check is
  still check-then-act. The WHERE clause folds the verification into the same
  statement as the signal.
- `pg_cancel_backend`/`pg_terminate_backend` only accept a pid; filtering
  `pg_stat_activity` rows first means a mismatched `backend_start` returns zero
  rows → `Ok(false)` (refused), never a signal to the wrong backend.
- `backend_start` for `client backend` rows is never NULL, but the code treats
  it as missing-information anyway and refuses rather than silently signalling.
- Struct-variant payloads on `MonitoringConfirmationAction` dropped the `Copy`
  derive; no other site relied on it.


## Source: `VERIFICATION.md`

# Verification — #501

- `cargo check` clean for db-pro-app, db-pro-core, db-pro-infrastructure,
  db-pro-runtime, db-pro-ui, db-pro-native.
- `cargo test -p db-pro-app` monitoring tests: confirm actions dispatch
  `UiCommand::{MonitoringCancelBackend,MonitoringTerminateBackend}` carrying
  `expected_backend_start`; `backend_start: None` dispatches nothing and sets
  feedback.
- `cargo test -p db-pro-core` monitoring_service: positive-pid validation kept,
  `cancel`/`terminate` reject empty backend_start, mock asserts the arg reaches
  the port.
- `cargo test -p db-pro-infrastructure` monitoring: SQL contract asserts
  `a.pid = $1 AND a.backend_start = $2::timestamptz` in both statements.
- Live-PG negative path (mismatched backend_start → Ok(false), no signal) rests
  on SQL semantics; no DB fixture in this repo's unit layer.

