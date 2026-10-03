# Findings

## Problem Analysis

In `crates/infrastructure/src/mysql/connector.rs`, `MySqlConnector::execute_batch` was previously implemented as:

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

This caused statements in a batch to execute auto-committed on the connection pool. If statement #2 of 3 failed, statement #1 remained committed, leaving the database in a partially mutated state.

## Solution

Delegating `execute_batch` to `self.execute_transaction(handle, statements, &vec![false; statements.len()])` ensures that:
1. `pool.begin()` opens an explicit transaction.
2. All statements execute sequentially within the transaction.
3. If any statement fails, `tx.rollback()` rolls back all prior statements in the batch.
4. If all statements succeed, `tx.commit()` commits the batch atomically.
5. The sum of affected rows across write statements is returned.
