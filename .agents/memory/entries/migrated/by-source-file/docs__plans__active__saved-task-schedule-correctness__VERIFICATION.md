---
schema_version: 1
id: "MEM-SRC-da385f23f8449bb5"
title: "Source mirror: docs/plans/active/saved-task-schedule-correctness/VERIFICATION.md"
type: pattern
scope: project
status: needs-review
mirror_kind: full-file
mirror_wave: v2-by-source-file
source_path: "docs/plans/active/saved-task-schedule-correctness/VERIFICATION.md"
source_sha256: "da385f23f8449bb519ba8f6a9f8322dfff874f1ab5e0e0b513fc0cfe110996bb"
mirror_body_sha256: "da385f23f8449bb519ba8f6a9f8322dfff874f1ab5e0e0b513fc0cfe110996bb"
truncated: false
created: "2026-10-24"
updated: "2026-10-24"
verified_on: unknown
confidence: low
tags: [migrated, by-source-file]
---
# Saved-task schedule correctness — Verification

State: REVIEW
Baseline: `76b120724a7542de8b587a002811ddacad28156a`.
Candidate: uncommitted worker patch on shared `main`; parent must record final exact commit SHA.

## Parent execution — authoritative integration update

- Public-store baseline smoke on exact source `76b120724a7542de8b587a002811ddacad28156a`: exit 101, four failing checks (`zero_enable_rejected`, `zero_enable_atomic`, `skip_exact_boundary`, `skip_closed_gap`), other checks pass (`artifact://92`).
- Identical public-store candidate smoke: 21/21 checks PASS, exit 0 (`artifact://91`). No SQL/provider execution; fixed timestamps, full-interval backlog boundary.
- `cargo +nightly fmt --all && cargo check --workspace && cargo test --workspace`: exit 0, 1,999 passed / 0 failed / 43 ignored across 26 suites (`artifact://93`), including saved-task and app scheduler regressions. This precedes later unrelated provider fixes; no claim of their final gates.
- Independent source review `ReviewScheduleSshSqlite`: ACCEPT for #504/#462, no introduced findings. Baseline SHA above; uncommitted candidate reviewed.
- Scanner: inherited `sql_embeds_credential_literal` complexity/length warnings in the same file, not changed by scheduling fix. No suppression/baseline weakening.
- Lifecycle REVIEW; owner requested commit/push and closure of individually verified issues. No GUI or database runtime claim; this change is provider-independent eligibility logic.
- Temporary smoke code had a type-inference error on a parsed timestamp; corrected to compare `now + std::time::Duration::from_secs(30)` identically for baseline/candidate. Only corrected runnable output above counts as reproduction.

The worker's NOT RUN statements below describe worker actions only, superseded for integration by this section. Source probe is removed after evidence collection; no permanent throwaway executable required.


## Commands actually executed

None. No tests, builds, linters, formatters or smoke programs were run by this worker. Results/counts/exit status are **NOT RUN**, not passing evidence. The issue reports are accepted as the baseline bug observations.

## Focused commands for parent

```sh
cargo test -p db-pro-core saved_task
```

New selectors (all in `crates/core/src/domain/tests/saved_task_tests.rs`):

- `enabling_zero_interval_is_rejected_without_mutating_store`
- `enabling_valid_schedule_preserves_existing_timing_and_retry_state`
- `enabling_valid_schedule_initializes_missing_next_run`
- `enabling_destructive_schedule_without_allow_is_atomic`
- `skip_missed_runs_current_interval_with_polling_jitter_once`
- `skip_missed_discards_full_interval_backlog_and_closed_app_gap`
- `run_once_dispatches_backlog_once_and_advances_from_now`
- `skip_missed_preserves_retry_after_failed_scheduled_run`
- `scheduler_still_blocks_destructive_payload_without_schedule_opt_in`

Existing adjacent app regression selector: `scheduled_task_tick_dispatches_once_while_app_active`; existing confirmation selector: `run_policy_requires_manual_confirmation_and_blocks_scheduled_destructive_work`. Parent may include these in its final integrated app tests.

## Independent public-store runtime smoke

The parent used an independent temporary executable over public SavedTask/SavedTaskStore and JSON deserialization, with fixed dates/task IDs and no SQL dispatch. The same harness ran against an exact baseline source export and the candidate. Both zero-interval atomicity and interval/backlog boundaries were observed, not inferred; exact outcomes are in the parent execution section.

Temporary executable source/manifests were removed after proof. Permanent adjacent saved-task tests retain the boundary, error, retry and destructive-policy regressions and are runnable with `cargo test -p db-pro-core saved_task`. There is no checked-in throwaway smoke executable to maintain.

## Provider matrix / lifecycle

| Surface | Automated | Runtime | Notes |
|---|---|---|---|
| Public saved-task store | PASS (integrated tests) | PASS (21-check standalone probe) | Parent evidence above |
| PostgreSQL | N/A | N/A | No SQL/provider path is executed |
| SQLite | N/A | N/A | No SQL/provider path is executed |
| Native UI | N/A | N/A | No UI edits |

## Completion decision

Implementation and runnable regressions are handed off for parent verification. Do not mark COMPLETED or close #504/#462 until the parent records focused/integrated gates, independent smoke evidence, review and exact candidate SHA. Shared status/changelog/commit/push/issue closure remain parent-owned.
