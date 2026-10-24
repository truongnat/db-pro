---
schema_version: 1
id: "MEM-SRC-6f53beb2db6af1e7"
title: "Source mirror: docs/plans/active/issue-501-backend-start-guard/VERIFICATION.md"
type: pattern
scope: project
status: needs-review
mirror_kind: full-file
mirror_wave: v2-by-source-file
source_path: "docs/plans/active/issue-501-backend-start-guard/VERIFICATION.md"
source_sha256: "6f53beb2db6af1e7d6d6b0423bdd80640e10fc53690141e4c58564acf57cd8bf"
mirror_body_sha256: "6f53beb2db6af1e7d6d6b0423bdd80640e10fc53690141e4c58564acf57cd8bf"
truncated: false
created: "2026-10-24"
updated: "2026-10-24"
verified_on: unknown
confidence: low
tags: [migrated, by-source-file]
---
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
