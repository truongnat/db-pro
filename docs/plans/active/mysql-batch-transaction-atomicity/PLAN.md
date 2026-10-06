# Feature Plan: MySQL Batch Transaction Atomicity and Rollback

Canonical Lifecycle: `BACKLOG → PLANNING → IMPLEMENTING → REVIEW → RUNTIME_VERIFY → COMPLETED`

## Overview
In `MySqlConnector::execute_batch` (`crates/infrastructure/src/mysql/connector.rs`), batch statements were executed sequentially directly against `&pool` in auto-commit mode without wrapping them in an explicit transaction (`pool.begin()`).
This caused a P1 transaction atomicity defect: if statement $N$ in a batch fails, statements $1 \dots N-1$ were committed and not rolled back, violating the contract of `DbConnector::execute_batch`.

## Problem & Severity
- **Severity**: P1 (Broken transaction semantics & database mutation atomicity on MySQL).
- **Evidence**: `crates/infrastructure/src/mysql/connector.rs` lines 124-138 executing statements against `&pool` in a loop without `pool.begin()` or rollback logic.
- **Failure Scenario**: Calling `execute_batch` on `MySqlConnector` with multiple DML statements where a later statement fails (e.g. `["UPDATE accounts SET balance = balance - 100 WHERE id = 1", "INSERT INTO invalid_table VALUES (1)"]`). The first statement was committed immediately in auto-commit mode; when the second statement failed, the transaction was not rolled back because no transaction was started, leaving the database in a partially updated / inconsistent state.

## Scope & Implementation Plan
1. Delegate `MySqlConnector::execute_batch` to `self.execute_transaction(handle, statements, &vec![false; statements.len()])`.
2. Sum `TransactionStatementResult::Affected` row counts on success.
3. Map `TransactionFailure` to its underlying `DbError` on failure, guaranteeing atomicity and rollback of preceding statements.
4. Add unit tests in `crates/infrastructure/src/mysql/connector.rs` verifying `execute_batch` behavior.

## Provider Alignment
- **MySQL**: Fixed to execute batch statements inside a transaction with rollback on failure.
- **PostgreSQL**: `PostgresConnector::execute_batch` executes statements in an explicit transaction (`pool.begin()`).
- **SQLite**: `SqliteActor::execute_batch` executes statements in an explicit transaction (`BEGIN TRANSACTION`).
- **SQL Server**: `SqlServerConnector::execute_batch` delegates to `execute_transaction`.
