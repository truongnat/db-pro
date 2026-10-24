---
schema_version: 1
id: "MEM-SRC-68a4d342d7d82100"
title: "Source mirror: docs/plans/active/issue-501-backend-start-guard/PLAN.md"
type: pattern
scope: project
status: needs-review
mirror_kind: full-file
mirror_wave: v2-by-source-file
source_path: "docs/plans/active/issue-501-backend-start-guard/PLAN.md"
source_sha256: "68a4d342d7d821003547c74c82709dcaddb2f7dbcf5bbf8cc4d581704c9edf94"
mirror_body_sha256: "68a4d342d7d821003547c74c82709dcaddb2f7dbcf5bbf8cc4d581704c9edf94"
truncated: false
created: "2026-10-24"
updated: "2026-10-24"
verified_on: unknown
confidence: low
tags: [migrated, by-source-file]
---
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
