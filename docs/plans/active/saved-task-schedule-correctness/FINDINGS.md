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
