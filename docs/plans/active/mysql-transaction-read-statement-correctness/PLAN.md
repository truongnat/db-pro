# Feature Plan: MySQL Transaction Read-Statement Execution, Validation, and Batch Atomicity

Canonical Lifecycle: `BACKLOG → PLANNING → IMPLEMENTING → REVIEW → RUNTIME_VERIFY → COMPLETED`

## Overview
In `MySqlConnector` (`crates/infrastructure/src/mysql/connector.rs`), batch execution (`execute_batch`) and multi-statement transactions (`execute_transaction`) had P1 transaction and mutation defects on MySQL:
1. `execute_batch` executed statements sequentially against a connection without an enclosing `START TRANSACTION` / `COMMIT` or `ROLLBACK`. On statement error or timeout, prior statements remained committed in MySQL, violating batch atomicity.
2. `execute_transaction` previously ignored `read_statements` and executed read statements as DML.

## Problem & Severity
- **Severity**: P1 (Database mutation correctness, transaction atomicity & rollback defect on MySQL).
- **Evidence**: `crates/infrastructure/src/mysql/connector.rs` lines 223–238 executed `conn.query_drop(statement)` directly in `execute_batch` without transaction boundaries or rollback handling.
- **Failure Scenario**: When running a batch execution with multiple statements on MySQL, if statement #2 fails or times out, statement #1 remains committed in MySQL (autocommit mode), causing partial state corruption and violating the `DbConnector::execute_batch` transaction contract.

## Scope & Implementation Plan
1. Refactor `MySqlConnector::execute_batch` to delegate execution to `self.execute_transaction(handle, statements, &vec![false; statements.len()]).await`.
2. Sum `row_count` across returned `TransactionStatementResult::Affected` results.
3. Map `TransactionFailure` back to `DbError` on failure (preserving error messages and rollback semantics).
4. Add unit tests in `crates/infrastructure/src/mysql/connector.rs` verifying `execute_batch` delegation, error mapping, and affected row count summation.

## Provider Coverage
- **MySQL**: Directly fixed and verified.
- **PostgreSQL**: Implemented and verified in `PostgresConnector::execute_batch`.
- **SQLite**: Implemented and verified in `SqliteConnector::execute_batch`.
- **SQL Server**: Implemented and verified in `SqlServerConnector::execute_batch`.
