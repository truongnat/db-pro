# Terminate Session Identity — Findings

## Root cause

The monitoring snapshot has a backend PID and `backend_start`, but the old cancel and terminate confirmation paths reduced the target to only the PID. The infrastructure adapter then called `pg_cancel_backend` / `pg_terminate_backend` by PID alone.

## Data path

`pg_stat_activity` → `MonitorSession` → terminate confirmation → `UiCommand` / `RuntimeCommand` → `MonitoringApi` → `MonitoringService` → `MonitoringPort` → PostgreSQL adapter.

## Safety behavior

- `backend_start` is captured at confirmation time from the displayed session snapshot.
- Missing or blank identity fails closed in the application service; the adapter also rejects blank values defensively.
- PostgreSQL queries select the target row from `pg_catalog.pg_stat_activity` with both `pid` and `backend_start` predicates before calling either `pg_cancel_backend` or `pg_terminate_backend`.
- No matching row means no function call and the adapter reports `false`.

## Related monitoring issue #482

The old `list_locks` query paired granted and waiting rows by resource identity only, ignoring PostgreSQL lock-mode conflicts; it could attribute a compatible lock holder as a blocker. The implementation now expands `pg_catalog.pg_blocking_pids(a.pid)`, which is PostgreSQL’s authoritative blocker API and accounts for lock conflicts and fast-path/group locks.

## Scope decisions

- #481 is separate: `DbConnector::cancel` handles query-execution cancellation and does not share this monitoring session action port/API path.
- #456 is unrelated timeout/cancellation cleanup and remains out of scope.
