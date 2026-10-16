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
