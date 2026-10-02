# Findings: MySQL Batch Execution Transactional Atomicity

## Issue Identified
In `crates/infrastructure/src/mysql/connector.rs`, `MySqlConnector::execute_batch` was executing statements sequentially against `MySqlPool` without wrapping them in `pool.begin()`.

## Root Cause
When executing multi-statement batches without an explicit transaction, MySQL auto-commits each statement individually upon execution. If any subsequent statement fails, earlier statements remain committed, violating the `DbConnector::execute_batch` contract:
> Execute multiple SQL statements atomically inside a single transaction. If any statement fails, all changes are rolled back.

## Resolution
By delegating `execute_batch` to `self.execute_transaction(handle, statements, &vec![false; statements.len()]).await`, `MySqlConnector` enforces:
1. `pool.begin()` transaction creation prior to statement execution.
2. Automatic `tx.rollback()` on any statement failure.
3. Proper calculation and return of cumulative affected rows upon successful commit.
