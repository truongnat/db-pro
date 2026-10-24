---
schema_version: 1
id: "MEM-SRC-394f0819966118b0"
title: "Source mirror: docs/plans/active/saved-task-schedule-correctness/PLAN.md"
type: pattern
scope: project
status: needs-review
mirror_kind: full-file
mirror_wave: v2-by-source-file
source_path: "docs/plans/active/saved-task-schedule-correctness/PLAN.md"
source_sha256: "394f0819966118b0c7d8947a132100f1a971232cc17fb36613b594c6751b43ee"
mirror_body_sha256: "394f0819966118b0c7d8947a132100f1a971232cc17fb36613b594c6751b43ee"
truncated: false
created: "2026-10-24"
updated: "2026-10-24"
verified_on: unknown
confidence: low
tags: [migrated, by-source-file]
---
# Saved-task schedule correctness — Plan

State: REVIEW
Branch: `main` (owner-authorized shared checkout)
Issues: #504, #462

## Requirements / context

- Enabling a stored disabled zero-second interval must return an error before changing enabled state, timestamps, scheduling or retry state. Disabled invalid drafts remain storable; no schema migration or clamping is introduced.
- A current due slot runs at most once. A slot is backlog when `now - next_run_at >= interval`; SkipMissed consumes backlog without dispatch and sets the next slot to `now + interval`. RunOnce dispatches once and also advances from now.
- This full-interval boundary was explicitly authorized by the parent after surfacing the conflict with issue #462's literal `next < now` proposal. Exact timestamp equality is unsuitable for real polling; no new configurable grace period is added.
- Preserve destructive opt-in/confirmation boundaries and scheduled failure retry behavior. No UI changes or SQL execution.

## Source evidence / baseline

All baseline source claims here refer to **76b120724a7542de8b587a002811ddacad28156a**:

- `crates/core/src/domain/saved_task.rs:293-299,316-331`: upsert permits disabled zero intervals; enabling omits interval validation.
- `crates/core/src/domain/saved_task.rs:363-374`: both missed-run branches enqueue the overdue slot.
- `crates/app/src/app/app_lifecycle.rs:155,164-171`: scheduler ticks per frame and requests another repaint after 50ms when a schedule is enabled; this is a polling request, not a guaranteed hard real-time deadline.
- `crates/core/src/domain/tests/saved_task_tests.rs:118-143` and `crates/app/src/app/tests/app_tests.rs:7017-7068`: existing 30-second schedules run when sampled one second late. A strict-lateness rule would break this intended behavior.
- `crates/app/src/tools/saved_task_state.rs:119-141,163-191`: public domain due IDs become scheduled/retry triggers and retain execution safety checks. `crates/app/src/app/app_lifecycle.rs:43` persists the store unconditionally, so a skipped slot's timing update requires no caller change.

## Scope / architecture

Change only the saved-task domain implementation and adjacent behavioral tests, plus this plan directory. The store decides eligibility; existing app dispatch, confirmations and provider execution remain unchanged. No new schedule options, dependencies or serialization fields.

Non-goals: background scheduling, exact wall-clock daemon semantics, provider execution, UI layout, retry redesign, interval overflow validation, unrelated scheduler refactoring.

## Provider matrix

| Provider | Impact | Required proof |
|---|---|---|
| PostgreSQL | Provider-independent eligibility only | Public store smoke; no DB connection |
| SQLite | Provider-independent eligibility only | Same store smoke; no DB connection |

## Acceptance / definition of done

- [x] Regression tests authored for zero-interval enable rejection with and without next timestamp, whole-store no mutation, valid enabling and destructive refusal.
- [x] Deterministic tests authored for exact due time, 50ms jitter, one-second jitter, just-before/full-interval boundary, closed-app gap, RunOnce backlog and repeated polling.
- [x] Retry and destructive eligibility regressions authored; no execution/confirmation policy bypass.
- [x] Independent public-store smoke source and before/after recipe supplied without executing SQL.
- [x] Parent executed public smoke and integrated fmt/check/tests; actual evidence in VERIFICATION.md.
- [x] Independent reviewer accepted the scheduling change, no introduced findings.
- [ ] Parent commits/pushes and closes verified issues; record final exact SHA.
