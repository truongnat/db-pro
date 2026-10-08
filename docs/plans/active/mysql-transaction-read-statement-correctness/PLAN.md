# Feature Plan: MySQL Transaction Read-Statement Execution, Pre-execution Failure Indexing, and Batch Atomicity

Canonical Lifecycle: `PLANNING → IMPLEMENTING → REVIEW → RUNTIME_VERIFY → COMPLETED`

## Overview
In `MySqlConnector` (`crates/infrastructure/src/mysql/connector.rs`), batch execution (`execute_batch`), multi-statement transactions (`execute_transaction`), and parameterized transactions (`execute_parameterized_transaction`) require accurate failure phase reporting and transaction atomicity:
1. `execute_batch` delegates execution to `execute_transaction` inside an explicit transaction block (`START TRANSACTION` / `COMMIT` or `ROLLBACK`).
2. `execute_transaction` and `execute_parameterized_transaction` report `TransactionFailure` with `statement_index = 0` during pre-execution phases (`Validation` and `Begin`) when 0 statements were executed, preventing downstream callers (like `TableMutationExecution`) from out-of-bounds array remapping.

## Problem & Severity
- **Severity**: P1 (Database mutation correctness, error attribution, transaction failure reporting).
- **Evidence**: `MySqlConnector` lacked explicit unit testing for `execute_parameterized_transaction`'s `Validation` phase reporting `statement_index == 0`.
- **Failure Scenario**: If `execute_parameterized_transaction` returns a non-zero or invalid `statement_index` on `Validation` failure (before any statement executes), downstream callers like `TableMutationExecution` attempt to index into caller mutation arrays, causing out-of-bounds indexing or wrong error attribution for table data mutations.

## Scope & Implementation Plan
1. Refactor `MySqlConnector::execute_batch` to delegate execution to `self.execute_transaction(handle, statements, &vec![false; statements.len()]).await`.
2. Ensure pre-execution failures (`Validation` and `Begin`) in `execute_transaction` and `execute_parameterized_transaction` return `statement_index = 0`.
3. Add unit tests in `crates/infrastructure/src/mysql/connector.rs` verifying `execute_parameterized_transaction` reports `Validation` phase failure with `statement_index = 0` on an unknown handle.

## Provider Coverage
- **MySQL**: Directly fixed and verified.
- **PostgreSQL**: Implemented and verified in `PostgresConnector`.
- **SQLite**: Implemented and verified in `SqliteConnector`.
- **SQL Server**: Implemented and verified in `SqlServerConnector`.
