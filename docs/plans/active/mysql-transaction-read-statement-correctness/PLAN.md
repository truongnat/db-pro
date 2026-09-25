# Feature Plan: MySQL Transaction Read-Statement Execution and Validation

Canonical Lifecycle: `BACKLOG → PLANNING → IMPLEMENTING → REVIEW → RUNTIME_VERIFY → COMPLETED`

## Overview
In `MySqlConnector::execute_transaction` (`crates/infrastructure/src/mysql/connector.rs`), the `read_statements: &[bool]` parameter is ignored (prefixed with `_read_statements`).
This causes two P1 transaction defects on MySQL:
1. Mismatched length between `statements` and `read_statements` is not validated during the `Validation` phase.
2. `SELECT` statements (or any statement marked with `is_read == true`) are executed as DML statements (`execute(&mut *tx)`), returning `TransactionStatementResult::Affected { row_count: 0 }` and discarding all result rows and column metadata.

## Problem & Severity
- **Severity**: P1 (Provider-specific correctness failure and broken transaction execution semantics on MySQL).
- **Evidence**: `crates/infrastructure/src/mysql/connector.rs` line 151:
  `async fn execute_transaction(&self, handle: &ConnectionHandle, statements: &[String], _read_statements: &[bool])`
- **Failure Scenario**: When running a transaction containing both `INSERT`/`UPDATE` and `SELECT` statements on MySQL (e.g. multi-statement queries, table mutation verification, or data editor transactions), any `SELECT` query returns 0 affected rows instead of its query result grid. Furthermore, invalid metadata parameters are silently passed to the statement execution loop.

## Scope & Implementation Plan
1. Validate `statements.len() == read_statements.len()` in `MySqlConnector::execute_transaction`.
2. Map `is_read == true` statements via `sqlx::query(stmt).fetch_all(&mut *tx).await` and `MySqlQueryMapper::map_rows(rows)`.
3. Add unit tests in `crates/infrastructure/src/mysql/connector.rs` for `Validation` and `Begin` transaction failure phases.

## Provider Coverage
- **MySQL**: Directly fixed and tested.
- **PostgreSQL**: Already implemented and verified in `PostgresConnector::execute_transaction`.
- **SQLite**: Already implemented and verified in `SqliteActor::handle_execute_transaction`.
- **SQL Server**: Already implemented and verified in `SqlServerConnector::execute_transaction`.
