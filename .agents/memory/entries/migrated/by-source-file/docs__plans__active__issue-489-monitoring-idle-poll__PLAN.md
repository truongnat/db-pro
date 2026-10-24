---
schema_version: 1
id: "MEM-SRC-ca43332658a8ce01"
title: "Source mirror: docs/plans/active/issue-489-monitoring-idle-poll/PLAN.md"
type: pattern
scope: project
status: needs-review
mirror_kind: full-file
mirror_wave: v2-by-source-file
source_path: "docs/plans/active/issue-489-monitoring-idle-poll/PLAN.md"
source_sha256: "ca43332658a8ce01de3db4a1621bf52811615c3d73730585d678605de93b4dfe"
mirror_body_sha256: "ca43332658a8ce01de3db4a1621bf52811615c3d73730585d678605de93b4dfe"
truncated: false
created: "2026-10-24"
updated: "2026-10-24"
verified_on: unknown
confidence: low
tags: [migrated, by-source-file]
---
# Issue #489 — monitoring auto-refresh stalls while idle

## Problem
`request_monitoring_poll_if_due` ran inside `MonitoringActivityContext::draw` —
it only executes when a frame is painted. egui sleeps when idle, so after one
poll the 5s deadline expired with nobody awake to notice.

## Fix
In `DbProApp::prepare_frame` (`app_lifecycle.rs`), when the Monitor activity is
shown, polling is enabled, and a connection is active:

```rust
let remaining = monitoring_poll_remaining(state.monitoring_last_poll, now);
ctx.request_repaint_after(remaining.max(Duration::from_millis(50)));
```

`monitoring_poll_remaining` / `MONITORING_POLL_INTERVAL` live in
`monitoring_activity_view.rs` next to the poll itself so cadence can't drift;
`request_monitoring_poll_if_due` reuses it.

Gates: `Activity::Monitor` (a hidden monitor never polls and would otherwise
spin at the 50ms clamp forever), `monitoring_poll`, connected + active
connection id (mirrors `draw_monitor_activity`).

## Other auto-refresh loops
Audited `security_activity_view.rs`, `audit_activity_view.rs`, `audit_state.rs`,
and every `elapsed()`/`Instant`-gated loop in crates/app: none are periodic —
they only request a repaint when a debounce/timer is already armed
(`query_editor` completion, run-control timer, diagram fit, diagnostics
debounce). Monitoring was the only affected loop.

## Files
- `crates/app/src/app/app_lifecycle.rs` — repaint scheduling
- `crates/app/src/monitoring/monitoring_activity_view.rs` — interval const +
  `monitoring_poll_remaining`
- `crates/app/src/monitoring/tests/monitoring_activity_view_tests.rs` — tests
