---
schema_version: 1
id: "MEM-SRC-bf389e18ec3e64c5"
title: "Source mirror: docs/plans/active/issue-489-monitoring-idle-poll/VERIFICATION.md"
type: pattern
scope: project
status: needs-review
mirror_kind: full-file
mirror_wave: v2-by-source-file
source_path: "docs/plans/active/issue-489-monitoring-idle-poll/VERIFICATION.md"
source_sha256: "bf389e18ec3e64c52ba6d9f79f955b334d51d5b20f3ede681feb8b7ab3dc6808"
mirror_body_sha256: "bf389e18ec3e64c52ba6d9f79f955b334d51d5b20f3ede681feb8b7ab3dc6808"
truncated: false
created: "2026-10-24"
updated: "2026-10-24"
verified_on: unknown
confidence: low
tags: [migrated, by-source-file]
---
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
