---
schema_version: 1
id: "MEM-PLAN-monitoring-event-correlation-2cfd05a6a12d"
title: "Plan archive: Monitoring Event Correlation"
type: decision
scope: project
status: needs-review
created: "2026-10-24"
updated: "2026-10-24"
verified_on: "unknown"
confidence: medium
tags: [migrated, docs-plans, decision, monitoring-event-correlation]
mirror_wave: v1-plan-evidence-excerpt
mirror_excerpt_sha256: "2cfd05a6a12de5a58cc7545756dd0716a125fecf6e028f235f4dfe29daf55909"
applies_to:
  repo: "db-pro"
  stack: "see sources"
  versions: "see plan state at migration"
sources:
  - "docs/plans/active/monitoring-event-correlation/PLAN.md"
  - "docs/plans/active/monitoring-event-correlation/FINDINGS.md"
  - "docs/plans/active/monitoring-event-correlation/VERIFICATION.md"
related: []
supersedes: null
---

# Tóm tắt

Migrate từ `docs/plans/active/monitoring-event-correlation/`. Loại suy luận: `decision`.

Entry tự động từ `docs/plans/`. Đối chiếu code hiện tại trước khi đặt `status: active`.

## Nội dung gốc (rút gọn)

## Source: `PLAN.md`

# Monitoring Refresh and Connection-Bound Responses — Plan

State: REVIEW — implementation and focused validation complete; see `VERIFICATION.md`.
Issues: truongnat/db-pro#489, #473, and #439.
Branch: `fix/monitoring-refresh-stale-results`.

## Goal

Wake the idle native UI when monitoring auto-refresh is due, and prevent delayed monitoring/security responses from an old request or database connection from overwriting current state.

## Scope

- Schedule an egui repaint for the next five-second monitoring poll only while Monitor is active, connected, and auto-refresh is enabled.
- Keep request ID and connection ID for monitoring snapshots/workload, security users/privileges/memberships/RLS, and connection-bound Audit, pg_settings, FDW, replication, and event-trigger inventories.
- Propagate connection ID on corresponding loaded events; apply a response only if its request ID is the latest for that surface and its connection is still active.
- Track reloads dispatched after successful management mutations so older inventory responses cannot overwrite the refreshed view.
- Clear audit page/filter/selection/export state when disconnecting or switching connections.
- Validate data-diff responses against request ID, active source connection, and requested target connection.
- Add focused regression tests for polling delay and request/connection correlation.

## Non-goals

- No polling while Monitor is inactive or auto-refresh is disabled.
- No change to database query semantics or security mutations.
- Non-connection-bound loaded event families (saved queries, folders, and table/query results) retain their existing request handling; this issue focuses on connection-bound monitoring/management surfaces.

## Acceptance

- [x] Idle Monitor schedules repaint for the next due poll.
- [x] Monitoring, security, and connection-bound management loaded responses carry request and connection identity end-to-end.
- [x] Cross-connection data-diff responses validate request, source, target, and active connection.
- [x] Stale request IDs, response connections, and inactive connections are rejected.
- [x] Focused tests, cross-crate compile, format and diff checks pass.


## Source: `FINDINGS.md`

# Findings

## #489 — idle monitoring refresh

`request_monitoring_poll_if_due` ran only during Monitor drawing. With no repaint source active, an idle egui context could remain asleep past the five-second poll interval. `prepare_frame` now schedules a repaint for the remaining interval while Monitor is active, connected, and auto-refresh is enabled. The next draw dispatches the poll and resets the timestamp.

## #473 — stale monitoring/security responses

`UiEvent` already carried request IDs, but the event router discarded them; loaded events also lacked the originating connection ID. Monitoring snapshots/workload and security users/privileges/memberships now retain pending `(request_id, connection_id)` pairs, and successful response events are accepted only when both match and the connection remains active.


## Source: `VERIFICATION.md`

# Verification

State: REVIEW — focused validation passed; full workspace suite and live database tests were not run.
Issues: truongnat/db-pro#489, #473, and #439.

## Commands

| Command | Result |
|---|---|
| `cargo test -p db-pro-app --lib monitoring` | PASS — 7 passed, 0 failed |
| `cargo test -p db-pro-app --lib connection_request_tests` | PASS — 1 passed, 0 failed |
| `cargo test -p db-pro-app --lib security_activity_view` | PASS — 2 passed, 0 failed |
| `cargo test -p db-pro-app --lib operation_events` | PASS — 1 passed, 0 failed |
| `cargo test -p db-pro-app --lib audit_state` | PASS — 2 passed, 0 failed |
| `cargo test -p db-pro-app --lib explorer_navigation` | PASS — 1 passed, 0 failed |
| `cargo check -p db-pro-app -p db-pro-runtime -p db-pro-native` | PASS |
| `cargo +nightly fmt --all -- --check` | PASS |
| `git diff --check` | PASS |

Full workspace suite and live database tests were not run.

