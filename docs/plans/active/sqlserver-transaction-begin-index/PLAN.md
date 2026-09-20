# Plan — Fix SQL Server Transaction Begin and Validation Failure Statement Index

## Baseline

- Target branch: `fix/sqlserver-transaction-begin-index`
- Severity: P1 / P2 (Transaction failure reporting contract mismatch)
- Canonical Lifecycle: `PLANNING → IMPLEMENTING → REVIEW → RUNTIME_VERIFY → COMPLETED`

## Objective

Fix `SqlServerConnector::execute_transaction` and `SqlServerConnector::execute_parameterized_transaction` so that when a transaction fails during `TransactionFailurePhase::Validation` or `TransactionFailurePhase::Begin`, `TransactionFailure::statement_index` is reported as `0` (indicating 0 statements were executed) rather than `statements.len()` (out-of-bounds).

## Scope

1. Update `SqlServerConnector::execute_transaction` in `crates/infrastructure/src/sqlserver/connector.rs` to set `statement_index: 0` for `Validation` and `Begin` failure phases.
2. Update `SqlServerConnector::execute_parameterized_transaction` in `crates/infrastructure/src/sqlserver/connector.rs` to set `statement_index: 0` for the `Begin` failure phase.
3. Add unit tests in `crates/infrastructure/src/sqlserver/connector.rs` verifying that `statement_index` is `0` when `Validation` or `Begin` fails.
4. Run core unit tests (`cargo test -p db-pro-core`).
