# Feature Plan: Fix Non-Atomic MySQL `execute_batch` Transaction Execution

Canonical Lifecycle: `BACKLOG → PLANNING → IMPLEMENTING → REVIEW → RUNTIME_VERIFY → COMPLETED`

## Overview
`MySqlConnector::execute_batch` (`crates/infrastructure/src/mysql/connector.rs`) executes batch statements sequentially using `sqlx::query(stmt).execute(&pool)` directly on the connection pool without initiating a transaction.
If statement 2 of 3 fails, statement 1 remains committed to the MySQL database. This violates the `DbConnector::execute_batch` contract which requires atomic execution where all changes roll back on failure.

## Problem & Severity
- **Severity**: P1 (Data corruption / Broken transaction semantics on MySQL batch execution).
- **Evidence**:
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
- **Failure Scenario**: A batch script containing multiple mutations (e.g. `UPDATE accounts SET balance = balance - 100 WHERE id = 1; UPDATE accounts SET balance = balance + 100 WHERE invalid_col = 2;`) is executed against a MySQL database. Statement 1 succeeds immediately on the pool connection. Statement 2 fails due to a syntax error. Because no transaction was started, statement 1 is not rolled back, leaving the database in an inconsistent/corrupted state.

## Proposed Fix
Delegate `execute_batch` to `execute_transaction`:
```rust
async fn execute_batch(&self, handle: &ConnectionHandle, statements: &[String]) -> Result<u64, DbError> {
    let read_statements = vec![false; statements.len()];
    let results = self
        .execute_transaction(handle, statements, &read_statements)
        .await
        .map_err(|failure| failure.error)?;
    let total = results
        .into_iter()
        .map(|res| match res {
            db_pro_core::ports::TransactionStatementResult::Affected { row_count, .. } => row_count,
            db_pro_core::ports::TransactionStatementResult::Query(_) => 0,
        })
        .sum();
    Ok(total)
}
```

## Provider Impact
- **MySQL**: Fixed to execute batch statements inside a transaction with automatic rollback on failure.
- **PostgreSQL**: `PostgresConnector::execute_batch` already executes inside a transaction (`pool.begin()`) with explicit rollback.
- **SQLite**: `SqliteActor::handle_execute_batch` already executes inside a transaction (`conn.unchecked_transaction()`) with explicit rollback.
- **SQL Server**: `SqlServerConnector::execute_batch` already delegates to `execute_transaction`.
