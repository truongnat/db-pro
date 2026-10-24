---
schema_version: 1
id: "MEM-PLAN-saved-task-schedule-correctness-344189347cf9"
title: "Plan archive: Saved Task Schedule Correctness"
type: decision
scope: project
status: needs-review
created: "2026-10-24"
updated: "2026-10-24"
verified_on: "unknown"
confidence: medium
tags: [migrated, docs-plans, decision, saved-task-schedule-correctness]
mirror_wave: v1-plan-evidence-excerpt
mirror_excerpt_sha256: "344189347cf9d221d3ef1ddfd34bd6716033d062a1d322b3315029554262e732"
applies_to:
  repo: "db-pro"
  stack: "see sources"
  versions: "see plan state at migration"
sources:
  - "docs/plans/active/saved-task-schedule-correctness/PLAN.md"
  - "docs/plans/active/saved-task-schedule-correctness/FINDINGS.md"
  - "docs/plans/active/saved-task-schedule-correctness/VERIFICATION.md"
  - "docs/plans/active/saved-task-schedule-correctness/AGENT_EVIDENCE.md"
related: []
supersedes: null
---

# Tóm tắt

Migrate từ `docs/plans/active/saved-task-schedule-correctness/`. Loại suy luận: `decision`.

Entry tự động từ `docs/plans/`. Đối chiếu code hiện tại trước khi đặt `status: active`.

## Nội dung gốc (rút gọn)

## Source: `PLAN.md`

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


## Source: `FINDINGS.md`

# Saved-task schedule correctness — Findings

All baseline source claims below are at `76b120724a7542de8b587a002811ddacad28156a`; working-tree changes are not executed evidence.

## P0

None identified in this slice.

## P1

None identified in this slice.

## P2

- **#504, inherited:** `crates/core/src/domain/saved_task.rs:293-299,316-331` permits storing a disabled zero interval, then enables it without validation. The working-tree patch adds the same interval error before mutation; tests compare the entire pre/post store with present and absent `next_run_at`.
- **#462, inherited:** `crates/core/src/domain/saved_task.rs:363-374` enqueues overdue slots for both policies. The working-tree patch suppresses dispatch only for SkipMissed backlog and advances to `now + interval`; skipped slots do not overwrite the last actually scheduled marker.

## Rejected / clarified proposals

Issue #462 suggests skipping every strictly late timestamp. At the exact baseline SHA above, `crates/app/src/app/app_lifecycle.rs:155,171` polls with independently sampled current time and a requested 50ms repaint delay. Existing tests at `crates/core/src/domain/tests/saved_task_tests.rs:118-143` and `crates/app/src/app/tests/app_tests.rs:7017-7068` explicitly accept one-second lateness for a 30-second interval. [INFERENCE] Requiring timestamp equality would make normal scheduling unreliable. Parent explicitly authorized a full interval as the backlog boundary: lateness `< interval` is current; lateness `>= interval` is skipped under SkipMissed. This intentionally does not detect app downtime shorter than one interval, since persisted timing alone cannot distinguish it from late polling.

No additional configurable grace, persisted poll marker, clock source or daemon was introduced.

## Runtime gaps

- Automated and independent smoke evidence: NOT RUN by worker; parent owns execution and final counts.
- PostgreSQL / SQLite: no provider calls are changed or needed to exercise scheduling; live provider execution is outside this slice.
- UI: unchanged; no visual evidence required for this domain-only correction.


## Source: `VERIFICATION.md`

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


## Source: `AGENT_EVIDENCE.md`

# Agent evidence — #504 / #462 saved-task schedule correctness

## 1. Claim

| Field | Value |
|---|---|
| Agent identity | ScheduleCorrectness / saved-task domain |
| Issue(s) | #504, #462 |
| Task state | Review |
| Baseline SHA | 76b120724a7542de8b587a002811ddacad28156a |
| Branch / PR | shared main; no new PR |
| Scope interpretation | Atomic refusal of zero-interval enable; distinguish current due slots from SkipMissed backlog without breaking polling jitter |
| Out of scope | UI, provider execution, retry redesign, shared changelog/status, commits/pushes/issue closure |

## 2. Progress checkpoint

- Current HEAD: parent-supplied baseline `76b120724a7542de8b587a002811ddacad28156a`; no worker commit or HEAD probe. Candidate consists of uncommitted slice changes, not a separate reviewed SHA.
- Acceptance implemented: zero enable refusal with whole-store equality; valid enable; SkipMissed current/closed-gap/full-interval boundaries; RunOnce backlog; repeat polling; retry and destructive-policy regression cases; independent public-store runtime smoke source.
- Remaining acceptance evidence: parent execution of focused and integrated gates and independent runtime smoke; final review/commit/closure.
- Findings / risks: inherited P2 #504 and #462 at baseline source `crates/core/src/domain/saved_task.rs:293-299,316-331,363-374`. Parent authorized full-interval backlog boundary after polling conflict was surfaced; no unresolved policy question remains in this slice.
- Tests already run: none; pass/fail/ignored counts and exit status NOT RUN per worker contract.
- Dependency / blocker changes: semantic `find` unavailable because judge requests were rejected; literal grep/read provided required source evidence. No implementation blocker.

## 3. Implementation handoff / review request

| Field | Value |
|---|---|
| Exact SHA | Baseline 76b120724a7542de8b587a002811ddacad28156a; candidate uncommitted, parent records final SHA |
| Commit list | none by worker |
| File / surface inventory | saved_task.rs: enable guard, backlog eligibility/policy docs; adjacent saved_task_tests.rs: nine behavioral regressions; canonical plan/acceptance/evidence files. Temporary public-store executable removed after observed before/after proof. |
| Acceptance mapping | #504 → zero-interval whole-store equality plus valid enable tests; #462 → jitter/boundary/gap/RunOnce tests and public smoke; adjacent safety → retry and destructive tests |
| Commands and counts | Worker executed none; parent outcomes and public-store smoke results are authoritative in VERIFICATION.md. |
| CI run IDs / status | not run |
| Known limitations | Lateness shorter than an interval is intentionally current, including short downtime; no persisted poll history or configurable grace |
| Migrations / config implications | none; disabled zero-interval drafts remain storable but cannot be enabled |
| Out-of-scope changes | none made |

## 4. Review outcome

| Field | Value |
|---|---|
| Reviewed SHA | n/a — independent parent review pending |
| Verdict | n/a — not independently reviewed |
| P0 / P1 / P2 counts | no new independently reviewed counts; two inherited P2 reports addressed in source, verification pending |
| Findings | See FINDINGS.md; all baseline source observations anchored to exact baseline SHA |
| CI disposition | not executed by worker |
| Next task(s) unblocked | parent final gates, runtime smoke, changelog/status update, commit/push/verified issue closure |

## 5. Research / audit handoff

- Source date: 2026-10-16.
- References: https://github.com/truongnat/db-pro/issues/504 ; https://github.com/truongnat/db-pro/issues/462 ; source paths in PLAN.md at baseline `76b120724a7542de8b587a002811ddacad28156a`.
- Factual findings: scheduler runs per frame and requests 50ms repaint while schedules are enabled; existing tests allow 1s late sampling for a 30s interval; the old two policies both dispatch backlog; enabling omits the existing upsert interval check.
- Inference: literal `next < now` rejection would make normal timestamped polling unreliable, not just suppress backlog.
- Decision: parent explicitly authorizes backlog at `now - next >= interval`; SkipMissed starts future timing from now without enqueue; current slots and RunOnce enqueue once.
- Unresolved questions: none in authorized slice.
- Downstream tasks activated: parent verification and closure of #504/#462.

## 6. Tổng kết (Vietnamese summary)

Đã thêm chặn bật lịch 0 giây trước mọi thay đổi trạng thái; SkipMissed không phát lệnh cho backlog trễ ít nhất một chu kỳ, còn tick trong chu kỳ hiện tại và RunOnce vẫn chạy đúng một lần. Ranh giới này đã được Main chấp thuận để tránh bỏ toàn bộ tick do jitter 50ms. Đã bổ sung 9 kiểm thử hành vi và chương trình smoke public API với thời gian cố định, không thực thi SQL. Chưa chạy build/test/lint/format/smoke theo phân công; Main cần chạy gate tích hợp, ghi SHA/kết quả thật, cập nhật tài liệu chung và đóng issue sau xác minh.

