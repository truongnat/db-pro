# Feature Plan: MySQL Batch Execution Transactional Atomicity & Rollback Correctness

Canonical Lifecycle: `BACKLOG → PLANNING → IMPLEMENTING → REVIEW → RUNTIME_VERIFY → COMPLETED`

## Overview
In `MySqlConnector::execute_batch` (`crates/infrastructure/src/mysql/connector.rs`), batch statements were executed sequentially directly against the connection pool (`sqlx::query(stmt).execute(&pool)`) without initiating a transaction (`BEGIN TRANSACTION`).

If statement $i$ succeeds and statement $i+1$ fails (due to a syntax error or constraint failure), statement $i$ remains permanently committed in the database, causing partial updates and data corruption. This violates the `DbConnector::execute_batch` trait contract: *"Execute multiple SQL statements atomically inside a single transaction. If any statement fails, all changes are rolled back."*

This plan refactors `MySqlConnector::execute_batch` to delegate execution to `self.execute_transaction`, guaranteeing transactional atomicity and rollback on failure across MySQL batch operations.

## Severity & Justification
- **Severity**: P1 (Database Mutation Correctness / Transaction Rollback Defect)
- **Evidence**: `crates/infrastructure/src/mysql/connector.rs` lines 124–138:
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
- **Failure Scenario**: A user or service executes a batch of SQL statements via `DbConnector::execute_batch` on MySQL (e.g. multi-statement schema modifications or table row updates):
  1. Statement 1: `UPDATE accounts SET balance = balance - 100 WHERE id = 1` (succeeds and commits immediately).
  2. Statement 2: `UPDATE non_existent_table SET balance = balance + 100 WHERE id = 2` (fails with table not found error).
  `execute_batch` returns `Err(...)`, but Statement 1's balance deduction remains committed in the database, causing data loss/corruption.

## Proposed Scope & Execution Steps
1. Refactor `MySqlConnector::execute_batch` in `crates/infrastructure/src/mysql/connector.rs` to delegate to `self.execute_transaction(handle, statements, &vec![false; statements.len()]).await`.
2. Map successful `TransactionStatementResult::Affected` row counts into a cumulative total `u64`.
3. On `TransactionFailure`, return `failure.error`, ensuring `execute_batch` returns an error after the transaction has been rolled back by `execute_transaction`.
4. Add unit test coverage in `crates/infrastructure/src/mysql/connector.rs` for `execute_batch` on an unknown connection handle.
5. Verify test suite and quality gates.
