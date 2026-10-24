---
schema_version: 1
id: "MEM-SRC-3e7f3e6630160d14"
title: "Source mirror: docs/plans/active/issue-489-monitoring-idle-poll/FINDINGS.md"
type: pattern
scope: project
status: needs-review
mirror_kind: full-file
mirror_wave: v2-by-source-file
source_path: "docs/plans/active/issue-489-monitoring-idle-poll/FINDINGS.md"
source_sha256: "3e7f3e6630160d14366b2acb0a5b8ce99fab60d15efa62f270a505a0d5a7f28d"
mirror_body_sha256: "3e7f3e6630160d14366b2acb0a5b8ce99fab60d15efa62f270a505a0d5a7f28d"
truncated: false
created: "2026-10-24"
updated: "2026-10-24"
verified_on: unknown
confidence: low
tags: [migrated, by-source-file]
---
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
