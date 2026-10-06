# Findings: MySQL Batch Transaction Atomicity

## Code Inspection Findings
- `MySqlConnector::execute_batch` directly loops over `statements` and calls `sqlx::query(stmt).execute(&pool)` without acquiring a transaction connection or calling `pool.begin()`.
- If an early statement in the slice mutates data or schema, and a subsequent statement fails, the early statement's effects persist in the database because MySQL autocommits each statement when autocommit is enabled (the default for connections from the pool).
- In contrast, `SqlServerConnector` implements `execute_batch` by calling `self.execute_transaction(handle, statements, &vec![false; statements.len()])`.
- `PostgresConnector` implements `execute_batch` by explicitly managing a `pool.begin()` transaction and calling `tx.rollback().await` on failure or timeout.
- `SqliteActor` implements `execute_batch` using a rusqlite transaction (`conn.transaction()`) and rolls back on failure.

## Refactoring Solution
- `MySqlConnector::execute_batch` should delegate to `self.execute_transaction(handle, statements, &vec![false; statements.len()]).await`.
- On `Ok(results)`, sum `row_count` from `TransactionStatementResult::Affected` variants.
- On `Err(failure)`, return `Err(failure.error)`.
- This ensures atomicity, proper transaction error handling, and exact conformance with `DbConnector::execute_batch`.
