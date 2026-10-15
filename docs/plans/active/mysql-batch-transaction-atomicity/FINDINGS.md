# Findings: Fix Non-Atomic MySQL `execute_batch` Transaction Execution

## Code Analysis
In `crates/infrastructure/src/mysql/connector.rs`, `execute_batch` was executing statements sequentially against `MySqlPool` without wrapping them in `pool.begin()`.

Other connector implementations in DB Pro handle `execute_batch` as follows:
- **PostgreSQL**: Begins a transaction via `pool.begin()`, executes statements, and calls `tx.rollback()` if any statement fails or times out.
- **SQLite**: Begins an `unchecked_transaction()`, executes statements, and rolls back if an error occurs.
- **SQL Server**: Delegates directly to `execute_transaction(handle, statements, &vec![false; statements.len()])`.

By delegating `MySqlConnector::execute_batch` to `self.execute_transaction(handle, statements, &vec![false; statements.len()])`, MySQL batch executions immediately gain atomicity, statement execution tracking, and automatic rollback on failure.
