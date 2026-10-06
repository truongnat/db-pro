# Technical Findings — MySQL Batch Transaction Atomicity

## Problem Description
`MySqlConnector::execute_batch` in `crates/infrastructure/src/mysql/connector.rs` previously looped over `statements` and executed each statement directly on `&pool` (`sqlx::query(stmt).execute(&pool)`).

## Severity Classification
**P1 — Transaction & Database Mutation Correctness Defect**.
`DbConnector::execute_batch` trait contract explicitly specifies:
> "Execute multiple SQL statements atomically inside a single transaction. If any statement fails, all changes are rolled back."

Executing batch statements outside a transaction in auto-commit mode breaks atomicity when any statement in the batch fails, leaving previous statements committed and database state corrupted.

## Solution Architecture
Delegate `MySqlConnector::execute_batch` to `self.execute_transaction(handle, statements, &vec![false; statements.len()])`.
`execute_transaction` starts an explicit transaction using `pool.begin()`, executes all statements, rolls back on any error (`tx.rollback()`), and commits on success (`tx.commit()`).
On success, `execute_batch` sums `TransactionStatementResult::Affected { row_count, .. }` across all statements and returns `Ok(total)`.
On failure, `execute_batch` returns `Err(failure.error)`.
