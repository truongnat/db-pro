---
schema_version: 1
id: "MEM-SRC-c108c63eb37d3a56"
title: "Source mirror: docs/plans/active/monitoring-event-correlation/PLAN.md"
type: pattern
scope: project
status: needs-review
mirror_kind: full-file
mirror_wave: v2-by-source-file
source_path: "docs/plans/active/monitoring-event-correlation/PLAN.md"
source_sha256: "c108c63eb37d3a561fb5e352b2d916c7bd5362dcd504762d57387bbb225cdf9c"
mirror_body_sha256: "c108c63eb37d3a561fb5e352b2d916c7bd5362dcd504762d57387bbb225cdf9c"
truncated: false
created: "2026-10-24"
updated: "2026-10-24"
verified_on: unknown
confidence: low
tags: [migrated, by-source-file]
---
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
