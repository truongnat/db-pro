# MySQL Batch Transaction Atomicity (#315)

## State

`IMPLEMENTING`

## Goal

Ensure `MySqlConnector::execute_batch` executes statements inside a single transaction and rolls back on failure, conforming to `DbConnector::execute_batch` contract and preventing partial mutation bugs.

## Evidence Baseline

- Problem statement: `MySqlConnector::execute_batch` in `crates/infrastructure/src/mysql/connector.rs` executed statements sequentially directly on the connection pool without wrapping them in a transaction.
- Contract violation: `DbConnector::execute_batch` trait in `crates/core/src/ports/db_connector.rs` explicitly mandates:
  *"Execute multiple SQL statements atomically inside a single transaction. If any statement fails, all changes are rolled back."*
- Severity: P1 (incorrect database mutation, broken transaction semantics / atomicity, provider capability mismatch).

## Scope

- Update `MySqlConnector::execute_batch` in `crates/infrastructure/src/mysql/connector.rs` to delegate to `self.execute_transaction(handle, statements, &vec![false; statements.len()])`.
- Add unit tests in `crates/infrastructure/src/mysql/connector.rs` covering validation failure and unknown connection handle behavior for `execute_batch`.
- Verify with `cargo test -p db-pro-core -p db-pro-ui`.

## Non-goals

- Altering other provider connectors or query mapper logic.
- Rewriting MySQL connection pool options.

## Acceptance Criteria

1. `MySqlConnector::execute_batch` executes statements in a transaction using `execute_transaction`.
2. Validation failures (such as unknown handle) return a `DbError` directly.
3. Unit tests pass cleanly.
