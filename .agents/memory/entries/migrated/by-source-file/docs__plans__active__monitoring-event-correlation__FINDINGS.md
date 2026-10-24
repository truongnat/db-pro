---
schema_version: 1
id: "MEM-SRC-38f71e0ee26f27f4"
title: "Source mirror: docs/plans/active/monitoring-event-correlation/FINDINGS.md"
type: pattern
scope: project
status: needs-review
mirror_kind: full-file
mirror_wave: v2-by-source-file
source_path: "docs/plans/active/monitoring-event-correlation/FINDINGS.md"
source_sha256: "38f71e0ee26f27f4276ad343cb2c9a59535583fc6d50d32ad3401fe13d2dac08"
mirror_body_sha256: "38f71e0ee26f27f4276ad343cb2c9a59535583fc6d50d32ad3401fe13d2dac08"
truncated: false
created: "2026-10-24"
updated: "2026-10-24"
verified_on: unknown
confidence: low
tags: [migrated, by-source-file]
---
# Findings

## #489 — idle monitoring refresh

`request_monitoring_poll_if_due` ran only during Monitor drawing. With no repaint source active, an idle egui context could remain asleep past the five-second poll interval. `prepare_frame` now schedules a repaint for the remaining interval while Monitor is active, connected, and auto-refresh is enabled. The next draw dispatches the poll and resets the timestamp.

## #473 — stale monitoring/security responses

`UiEvent` already carried request IDs, but the event router discarded them; loaded events also lacked the originating connection ID. Monitoring snapshots/workload and security users/privileges/memberships now retain pending `(request_id, connection_id)` pairs, and successful response events are accepted only when both match and the connection remains active.
