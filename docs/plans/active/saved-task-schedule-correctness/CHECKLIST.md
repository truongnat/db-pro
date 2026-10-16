# Saved-task schedule correctness — Checklist

State: REVIEW
Baseline for source observations: `76b120724a7542de8b587a002811ddacad28156a`.

## Implementation tasks

- [x] Trace domain → saved-task state → per-frame app scheduler and persistence.
- [x] Surface strict-timestamp versus polling-jitter conflict; parent authorizes full-interval backlog boundary.
- [x] Add regression cases before applying production changes (not executed by worker).
- [x] Reject zero-interval enable before any mutation.
- [x] Consume SkipMissed backlog without dispatch; keep current ticks and RunOnce behavior.
- [x] Keep retry and destructive-policy behavior covered without SQL execution.
- [x] Supply public store smoke and baseline/candidate instructions.
- [x] PostgreSQL / SQLite impact explicitly provider-independent; no credentials needed.

## Parent-owned gates

- [x] Saved-task and app scheduler regressions pass in integrated workspace: 1,999 passed / 0 failed / 43 ignored.
- [x] Baseline independent smoke demonstrates four failures; candidate smoke passes 21/21 checks.
- [x] Integrated formatting/check/workspace tests executed; inherited workspace Clippy gate recorded separately.
- [x] Independent reviewer ACCEPT; no unresolved introduced P0/P1/P2 findings.
- [ ] Parent publish commit/push and close #504/#462 with evidence.

No build/test/lint/format/smoke commands were executed by this worker, per assignment.
