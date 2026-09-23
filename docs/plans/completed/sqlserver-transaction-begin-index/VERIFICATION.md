# Verification — SQL Server Transaction Begin Index Fix

State: COMPLETED
Implementation commit: `61e97658` ("fix(sqlserver): report statement_index 0 on transaction begin and validation failures")
Verified on: 2026-09-23 (HEAD `aaa7a69e`)

## Automated Tests Executed

- `cargo test -p db-pro-infrastructure --lib statement_index` — **3 passed / 0 failed**:
  - `sqlserver::connector::tests::execute_transaction_reports_zero_statement_index_on_validation_failure`
  - `sqlserver::connector::tests::execute_transaction_reports_zero_statement_index_on_begin_failure`
  - `sqlserver::connector::tests::execute_parameterized_transaction_reports_zero_statement_index_on_begin_failure`

## Source evidence

`crates/infrastructure/src/sqlserver/connector.rs`:
- `execute_transaction` — `TransactionFailurePhase::Validation` and `TransactionFailurePhase::Begin` now set `statement_index: 0` (were `statements.len()`).
- `execute_parameterized_transaction` — `TransactionFailurePhase::Begin` sets `statement_index: 0`.

This matches the SQLite/PostgreSQL/MySQL connectors, so the caller boundary check
`if failure.statement_index < indexed_mutations.len()` in `TableDataService::apply_mutations_detailed`
maps the failure correctly instead of leaving an out-of-bounds index unmapped.

## Provider matrix

| Provider | Supported operation | Automated evidence | Live/runtime evidence | Capability gate |
|---|---|---|---|---|
| SQL Server | yes | PASS (unit) | inherited by parent `sqlserver-provider` (no live SQL Server fixture here) | n/a |
| PostgreSQL / SQLite / MySQL | yes (already correct) | PASS | n/a for this fix | n/a |

The Begin/Validation `statement_index` invariant is provider-logic that unit tests fully exercise
(no statements run before the failure, so no live server is required to prove index `0`). Live SQL
Server runtime for the broader provider remains tracked under `sqlserver-provider`.

## Findings

- P0: 0
- P1: 0 (the reported statement-index defect is fixed and regression-tested)
- P2: 0

## Verification Status

COMPLETED — code merged to `main` (`61e97658`) and regression tests pass.
