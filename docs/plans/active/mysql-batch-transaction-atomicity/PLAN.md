# Feature Plan: MySQL Batch Statement Transaction Atomicity

Canonical Lifecycle: `BACKLOG → PLANNING → IMPLEMENTING → REVIEW → RUNTIME_VERIFY → COMPLETED`

## Overview
In `MySqlConnector::execute_batch` (`crates/infrastructure/src/mysql/connector.rs`), batch SQL statements were executed sequentially directly against the connection pool (`&pool`) outside of a database transaction.
If an error occurs on statement N (N > 0), statements 0..N-1 remain committed in MySQL due to autocommit behavior, violating the atomicity and rollback invariant specified in the `DbConnector::execute_batch` trait contract ("Execute multiple SQL statements atomically inside a single transaction. If any statement fails, all changes are rolled back.").

## Problem & Severity
- **Severity**: P1 (Database mutation correctness, broken transaction/rollback semantics on MySQL provider).
- **Evidence**: `crates/infrastructure/src/mysql/connector.rs` lines 135-149:
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
- **Failure Scenario**: A caller executes a batch of DML mutations where statement 1 succeeds and statement 2 fails with a syntax or constraint error. Statement 1 remains committed on MySQL instead of rolling back, leaving the database in a partially mutated state.

## Implementation Plan
1. Delegate batch execution in `MySqlConnector::execute_batch` to `self.execute_transaction(handle, statements, &vec![false; statements.len()])`.
2. Map statement results back to a total row count sum (`TransactionStatementResult::Affected`).
3. Add unit tests for `execute_batch` error handling.
