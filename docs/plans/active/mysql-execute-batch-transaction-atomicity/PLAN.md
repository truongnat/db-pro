# Feature Plan: MySQL Batch Transaction Atomicity (`MySqlConnector::execute_batch`)

Canonical Lifecycle: `BACKLOG → PLANNING → IMPLEMENTING → REVIEW → RUNTIME_VERIFY → COMPLETED`

## Overview
`MySqlConnector::execute_batch` (`crates/infrastructure/src/mysql/connector.rs`) currently executes statements sequentially directly against `&pool` in autocommit mode without starting an explicit transaction. If any statement in the batch fails, earlier statements remain permanently committed in MySQL, violating the atomicity and rollback contract of `DbConnector::execute_batch`.

## Problem & Severity
- **Severity**: P1 (Broken transaction semantics, non-atomic database mutation, and silent partial persistence on batch execution failure).
- **Evidence**: `crates/infrastructure/src/mysql/connector.rs` lines 134-148.
- **Failure Scenario**: When running a batch of statements where statement 1 succeeds and statement 2 fails, statement 1 is autocommitted by MySQL before statement 2 fails. The caller receives `Err`, but statement 1's changes are already persisted, corrupting database state.

## Scope & Implementation Plan
1. Refactor `MySqlConnector::execute_batch` in `crates/infrastructure/src/mysql/connector.rs` to delegate to `self.execute_transaction(handle, statements, &vec![false; statements.len()])`.
2. Map `TransactionStatementResult::Affected { row_count, .. }` to sum row counts, or extract `failure.error` on failure.
3. Add unit tests for `MySqlConnector::execute_batch` in `crates/infrastructure/src/mysql/connector.rs`.
4. Run workspace quality gates.

## Provider Coverage
- **MySQL**: Directly fixed and tested.
- **PostgreSQL**: Already uses `pool.begin()` transaction in `PostgresConnector::execute_batch`.
- **SQLite**: Already uses `conn.unchecked_transaction()` in `SqliteActor::handle_execute_batch`.
- **SQL Server**: Already delegates to `execute_transaction` in `SqlServerConnector::execute_batch`.
