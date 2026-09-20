# Findings — SQL Server Transaction Failure Statement Index Defect

## Description

In `crates/infrastructure/src/sqlserver/connector.rs`, the `SqlServerConnector` implementation of `execute_transaction` and `execute_parameterized_transaction` contained a defect in failure reporting during the `Validation` and `Begin` phases.

When metadata validation failed or when `BEGIN TRANSACTION` failed (for instance due to an invalid handle or connection state), `SqlServerConnector` constructed a `TransactionFailure` with `statement_index` set to `statements.len()`.

## Impact

1. **Out-of-bounds Statement Index**: For a transaction batch with $N$ statements ($N > 0$), if `BEGIN TRANSACTION` fails before any statements are executed, reporting `statement_index = N` indicates that $N$ statements were processed prior to failure.
2. **Caller Mapping Mismatch**: Caller routines such as `TableDataService::apply_mutations_detailed` attempt to map `failure.statement_index` back to original staged mutation indices. When `statement_index == N` (where $N = \text{indexed\_mutations.len()}$), the boundary check `if failure.statement_index < indexed_mutations.len()` fails, leaving the out-of-bounds index unmapped and misattributing the failure.
3. **Provider Inconsistency**: SQLite, PostgreSQL, and MySQL connectors all report `statement_index: 0` for `Validation` and `Begin` failure phases.

## Fix Strategy

Change `statement_index: statements.len()` to `statement_index: 0` for both `TransactionFailurePhase::Validation` and `TransactionFailurePhase::Begin` in `SqlServerConnector`.
