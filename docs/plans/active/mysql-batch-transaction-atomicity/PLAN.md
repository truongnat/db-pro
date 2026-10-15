# Plan — MySQL Batch Transaction Atomicity Fix (#mysql-batch-transaction-atomicity)

Lifecycle: `BACKLOG → PLANNING → IMPLEMENTING → REVIEW → RUNTIME_VERIFY → COMPLETED`

## Overview
In `MySqlConnector::execute_batch` (`crates/infrastructure/src/mysql/connector.rs`), batch SQL statements are executed sequentially against the connection pool without initiating a database transaction.
This violates the `DbConnector::execute_batch` trait contract, which requires atomic execution within a single transaction and automatic rollback on statement failure.

## Problem & Severity
- **Severity**: P1 (Data safety, broken transaction/rollback semantics, and database mutation atomicity on MySQL).
- **Evidence**: `crates/infrastructure/src/mysql/connector.rs` lines 134-148:
  ```rust
  async fn execute_batch(&self, handle: &ConnectionHandle, statements: &[String]) -> Result<u64, DbError> {
      let pool = self
          .get_pool(handle)
          .await
          .ok_or_else(|| DbError::ConnectionFailed("no MySQL pool for handle".into()))?;

      let mut total = 0;
      for stmt in statements {
          let result = sqlx::query(stmt)
              .execute(&pool)
              .await
              .map_err(|e| DbError::QueryFailed(format!("MySQL batch statement failed: {}", e)))?;
          total += result.rows_affected();
      }
      Ok(total)
  }
  ```
- **Failure Scenario**: A batch containing multiple DML statements (e.g. `UPDATE acc1 ...`, `INVALID_SQL`, `UPDATE acc2 ...`) executes statement 1 against the pool in autocommit mode. When statement 2 fails, statement 1's mutations remain committed in MySQL, causing data corruption and partial batch updates.

## Scope & Implementation Plan
1. Refactor `MySqlConnector::execute_batch` to delegate batch execution to `self.execute_transaction(handle, statements, &read_statements)` with `read_statements = vec![false; statements.len()]`.
2. Extract total affected rows from `TransactionStatementResult::Affected` or return `failure.error` on transaction failure.
3. Add unit test in `crates/infrastructure/src/mysql/connector.rs` verifying that `execute_batch` delegates to `execute_transaction` and handles failure phases correctly.
