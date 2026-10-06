# Feature Plan: MySQL Batch Transaction Atomicity

Canonical Lifecycle: `BACKLOG → PLANNING → IMPLEMENTING → REVIEW → RUNTIME_VERIFY → COMPLETED`

## Overview
In `MySqlConnector::execute_batch` (`crates/infrastructure/src/mysql/connector.rs`), individual statements are executed sequentially against the connection pool without an enclosing transaction (`sqlx::query(stmt).execute(&pool)`).
This violates the `DbConnector::execute_batch` trait contract, which specifies:
"Execute multiple SQL statements atomically inside a single transaction. If any statement fails, all changes are rolled back."

## Problem & Severity
- **Severity**: P1 (Broken transaction execution semantics and database mutation atomicity defect on MySQL).
- **Evidence**: `crates/infrastructure/src/mysql/connector.rs` lines 134–147:
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
- **Failure Scenario**: When running a batch of DDL or DML statements on MySQL (e.g., executing a multi-statement schema migration or batch mutation in `SchemaService::apply_migration`), if statement #1 succeeds and statement #2 fails, statement #1 remains committed in the database without rolling back, causing database state corruption or partial DDL/DML application.

## Scope & Implementation Plan
1. Delegate `execute_batch` in `MySqlConnector` to `execute_transaction`, passing `read_statements` set to `vec![false; statements.len()]`.
2. Sum and return total affected rows from the `TransactionStatementResult::Affected` variants upon success, or return `failure.error` on failure.
3. Add unit test in `crates/infrastructure/src/mysql/connector.rs` for `execute_batch` failure handling.
4. Add integration test in `crates/infrastructure/tests/mysql_integration.rs` verifying batch rollback behavior when a statement fails.

## Provider Coverage
- **MySQL**: Fixed and verified by delegating to `execute_transaction`.
- **Postgres**: Already executes inside explicit `pool.begin()` transaction in `PostgresConnector::execute_batch`.
- **SQLite**: Already executes inside explicit transaction in `SqliteActor::handle_execute_batch`.
- **SQL Server**: Already delegates to `execute_transaction` in `SqlServerConnector::execute_batch`.
