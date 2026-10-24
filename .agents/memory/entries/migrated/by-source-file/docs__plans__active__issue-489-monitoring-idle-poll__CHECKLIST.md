---
schema_version: 1
id: "MEM-SRC-a16e970f70c078ea"
title: "Source mirror: docs/plans/active/issue-489-monitoring-idle-poll/CHECKLIST.md"
type: pattern
scope: project
status: needs-review
mirror_kind: full-file
mirror_wave: v2-by-source-file
source_path: "docs/plans/active/issue-489-monitoring-idle-poll/CHECKLIST.md"
source_sha256: "a16e970f70c078eafd1ea194e8371113d3ff2b323e8c254d1ffdd6f3d205b32e"
mirror_body_sha256: "a16e970f70c078eafd1ea194e8371113d3ff2b323e8c254d1ffdd6f3d205b32e"
truncated: false
created: "2026-10-24"
updated: "2026-10-24"
verified_on: unknown
confidence: low
tags: [migrated, by-source-file]
---
# Checklist — #489

- [x] `prepare_frame` calls `request_repaint_after(remaining.max(50ms))` when
      Monitor + poll enabled + connected
- [x] Single cadence source `MONITORING_POLL_INTERVAL` shared by poll + schedule
- [x] `monitoring_poll_remaining` unit-tested (never/just/mid/overdue/future)
- [x] Code-path test asserts prepare_frame schedules the repaint
- [x] Audited other refresh loops (security/audit/editor) — none periodic
- [x] `cargo check`/`cargo test` on db-pro-app
