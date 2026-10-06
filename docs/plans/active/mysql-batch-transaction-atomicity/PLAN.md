# Feature Plan: MySQL Batch Transaction Atomicity

Canonical Lifecycle: `BACKLOG → PLANNING → IMPLEMENTING → REVIEW → RUNTIME_VERIFY → COMPLETED`

## Overview
`MySqlConnector::execute_batch` (`crates/infrastructure/src/mysql/connector.rs`) currently executes statements sequentially directly against `MySqlPool` without wrapping them in an explicit transaction (`BEGIN` / `COMMIT` / `ROLLBACK`).

This violates the `DbConnector::execute_batch` trait contract:
> "Execute multiple SQL statements atomically inside a single transaction. If any statement fails, all changes are rolled back."

## Problem & Severity
- **Severity**: P1 (Database mutation correctness and broken transaction/rollback semantics on MySQL).
- **Evidence**: `crates/infrastructure/src/mysql/connector.rs` lines 134-148 executes statements sequentially against `pool` in autocommit mode without transaction boundaries.
- **Failure Scenario**: When executing a batch of DML or DDL statements on MySQL and a statement mid-batch fails (e.g. constraint violation, syntax error, or duplicate key), preceding statements remain permanently committed in the database while the caller receives an error, resulting in corrupted or partially applied state.

## Scope & Implementation Plan
1. Update `MySqlConnector::execute_batch` to delegate statement execution to `self.execute_transaction(handle, statements, &read_statements)` where `read_statements` is initialized to `vec![false; statements.len()]`.
2. Retain the `"vendored"` feature flag for `keyring` in `Cargo.toml` to ensure clean workspace compilation in Linux container environments without pre-installed system D-Bus headers.
3. Add regression unit tests in `crates/infrastructure/src/mysql/connector.rs` for `execute_batch`.

## Provider Coverage
- **MySQL**: Fixed and verified.
- **PostgreSQL**: Implemented and verified in `PostgresConnector::execute_batch`.
- **SQLite**: Implemented and verified in `SqliteActor::handle_execute_batch`.
- **SQL Server**: Implemented and verified in `SqlServerConnector::execute_batch`.
