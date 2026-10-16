# Terminate Session Identity — Plan

State: REVIEW — implementation and focused validation are complete; results are recorded in `VERIFICATION.md`.
Issues: truongnat/db-pro#501 (PID reuse during terminate confirmation) and #482 (incorrect PostgreSQL lock blocker attribution).
Branch: `fix/terminate-session-identity`

## Goal

Monitoring must cancel or terminate only the PostgreSQL backend selected in the snapshot and report only real lock blockers. A PID can be reused after the snapshot, and matching lock resources alone does not prove lock-mode conflict.

## Evidence

- `MonitorSession` already contains `backend_id` and `backend_start` from `pg_stat_activity`.
- The existing cancel and terminate confirmation paths retained the full session row in app state but reduced it to the PID before calling `pg_cancel_backend($1)` / `pg_terminate_backend($1)`.
- This allowed a stale confirmation to target a different backend if the PID had been reused.

## Scope

- Carry the captured `backend_start` alongside the PID through both cancel and terminate confirmation actions, UI/runtime commands, workers, API, application service, and monitoring port.
- Reject missing/empty identity before either backend action.
- In the PostgreSQL adapter, invoke `pg_cancel_backend` / `pg_terminate_backend` only for a `pg_stat_activity` row matching both PID and `backend_start`; a missing/mismatched row returns `false` without acting.
- Replace manual lock-resource matching with PostgreSQL `pg_blocking_pids()` and add focused application/adapter regression tests.

## Exclusions

- #481 is not included: its issue path is `DbConnector::cancel` for query execution, not the monitoring port/session termination flow.
- #456 remains excluded; timeout/cancellation cleanup has separate state-management scope.
- No changes to query-execution cancellation (`DbConnector::cancel`) or unrelated monitoring features.

## Acceptance criteria

- [x] Confirmation carries the snapshot identity end-to-end.
- [x] Missing/empty identity is rejected before `pg_cancel_backend` and `pg_terminate_backend`.
- [x] Adapter SQL requires PID and snapshot start timestamp in the same statement as cancel/termination.
- [x] Regression tests cover matching identity, stale/no-row identity, missing identity, and blocker PID query contract.
- [x] Focused tests, cross-crate check, formatting and diff checks recorded.
