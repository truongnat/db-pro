# Findings: MySQL Batch Transaction Atomicity

## P1 — Non-transactional execution in `MySqlConnector::execute_batch`
- **Location**: `crates/infrastructure/src/mysql/connector.rs`
- **Root Cause**: `execute_batch` iterated over `statements` and executed each statement directly via `sqlx::query(stmt).execute(&pool)` without an explicit transaction block.
- **Impact**: Batch statements ran in MySQL autocommit mode. If statement $N$ failed in a batch of $M$ statements ($N < M$), statements $1 \dots N-1$ remained committed in the database, breaking atomicity and leaving the database in a partially mutated state.
- **Fix**: Delegate `MySqlConnector::execute_batch` to `self.execute_transaction(handle, statements, &read_statements)`, which handles `BEGIN TRANSACTION`, per-statement execution, automatic rollback on error, and `COMMIT TRANSACTION`.
