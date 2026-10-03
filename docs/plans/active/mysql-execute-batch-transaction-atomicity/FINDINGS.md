# Findings: MySQL Batch Transaction Atomicity

## P1 — Non-Atomic Statement Execution in `MySqlConnector::execute_batch`
- **Location**: `crates/infrastructure/src/mysql/connector.rs` (lines 134-148)
- **Root Cause**: `execute_batch` iterated over `statements` calling `sqlx::query(stmt).execute(&pool)` directly on the `MySqlPool` connection pool in autocommit mode.
- **Impact**: If statement $N$ in a multi-statement batch failed, statements $1 \dots N-1$ were already committed to the database and not rolled back.
- **Fix**: Delegate `execute_batch` to `self.execute_transaction(handle, statements, &vec![false; statements.len()])`, ensuring all statements run inside an explicit `pool.begin()` transaction with rollback on failure.
