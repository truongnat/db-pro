# Feature Plan: MySQL Batch Statement Execution Transaction Atomicity

Canonical Lifecycle: `BACKLOG → PLANNING → IMPLEMENTING → REVIEW → RUNTIME_VERIFY → COMPLETED`

## Overview
`MySqlConnector::execute_batch` (`crates/infrastructure/src/mysql/connector.rs`) currently executes statements sequentially on a connection pool without wrapping them in an explicit transaction (`BEGIN` ... `COMMIT`). If statement N in a batch fails, statements 1..N-1 remain committed in autocommit mode, violating the `DbConnector::execute_batch` trait contract and causing partial database mutations.

## Problem & Severity
- **Severity**: P1 (Broken transaction / rollback semantics and database mutation correctness on MySQL).
- **Evidence**: `crates/infrastructure/src/mysql/connector.rs` lines 134-149:
  ```rust
  async fn execute_batch(&self, handle: &ConnectionHandle, statements: &[String]) -> Result<u64, DbError> {
      let pool = self.get_pool(handle).await...;
      let mut total = 0;
      for stmt in statements {
          let result = sqlx::query(stmt).execute(&pool).await...;
          total += result.rows_affected();
      }
      Ok(total)
  }
  ```
- **Failure Scenario**: A user runs a schema or batch script containing multiple DML/DDL statements on a MySQL database (e.g., updating multiple tables in a single operation). If statement 3 fails due to a constraint violation or syntax error, statements 1 and 2 are already committed permanently to the database instead of being rolled back atomically.

## Scope & Implementation Plan
1. Refactor `MySqlConnector::execute_batch` to delegate to `self.execute_transaction(handle, statements, &vec![false; statements.len()]).await`.
2. Map `TransactionFailure` back into `DbError`.
3. Sum up row counts from `TransactionStatementResult::Affected` variants.
4. Add unit test in `crates/infrastructure/src/mysql/connector.rs` verifying error handling for batch execution.

## Provider Coverage
- **MySQL**: Directly fixed and tested in this change.
- **PostgreSQL**: Already wraps batch execution in a transaction with rollback (`PostgresConnector::execute_batch`).
- **SQLite**: Already wraps batch execution in a transaction with rollback (`SqliteActor::execute_batch`).
- **SQL Server**: Already delegates batch execution to `execute_transaction` (`SqlServerConnector::execute_batch`).
