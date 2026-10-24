---
schema_version: 1
id: "MEM-PLAN-issue-489-monitoring-idle-poll-f0860548439c"
title: "Plan archive: Issue 489 Monitoring Idle Poll"
type: bugfix
scope: project
status: needs-review
created: "2026-10-24"
updated: "2026-10-24"
verified_on: "unknown"
confidence: medium
tags: [migrated, docs-plans, bugfix, issue-489-monitoring-idle-poll]
mirror_wave: v1-plan-evidence-excerpt
mirror_excerpt_sha256: "f0860548439c4182c2f451df0e5d2eab46d4b9fa45be0a34f777503cb4adddf9"
applies_to:
  repo: "db-pro"
  stack: "see sources"
  versions: "see plan state at migration"
sources:
  - "docs/plans/active/issue-489-monitoring-idle-poll/PLAN.md"
  - "docs/plans/active/issue-489-monitoring-idle-poll/FINDINGS.md"
  - "docs/plans/active/issue-489-monitoring-idle-poll/VERIFICATION.md"
related: []
supersedes: null
---

# Tóm tắt

Migrate từ `docs/plans/active/issue-489-monitoring-idle-poll/`. Loại suy luận: `bugfix`.

Entry tự động từ `docs/plans/`. Đối chiếu code hiện tại trước khi đặt `status: active`.

## Nội dung gốc (rút gọn)

## Source: `PLAN.md`

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


## Source: `FINDINGS.md`

# Findings — #489

- Poll cadence was hard-coded `Duration::from_secs(5)` in
  `request_monitoring_poll_if_due`; now `MONITORING_POLL_INTERVAL`.
- Without a wake-up the whole draw path (surface → sessions → confirm dialogs)
  stops: poll, toasts of monitoring results, everything.
- `prepare_frame` already coalesces repaint requests for runtime work; the
  monitoring schedule composes — `request_repaint_after` takes the earliest.
- Clamp floor 50ms avoids a zero-delay busy frame when a poll is due this frame.
- Clock skew safety: `Instant::elapsed()` can panic when `last_poll > now`;
  `monitoring_poll_remaining` uses `saturating_duration_since` and returns ZERO
  for overdue, interval for in-the-future.


## Source: `VERIFICATION.md`

# Verification — #489

- `monitoring_poll_remaining` unit test: None→0, fresh→5s, 2s elapsed→3s,
  stale→0, future timestamp→saturates (no panic).
- `include_str!("../../app/app_lifecycle.rs")` test pins that prepare_frame
  calls `monitoring_poll_remaining` + `request_repaint_after` — the exact code
  path that wakes egui while idle.
- `request_monitoring_poll_if_due` now shares the same helper, so due-ness and
  the scheduled wake can never disagree.
- Runtime visual proof intentionally not required (per issue); logic + code
  path are covered.

