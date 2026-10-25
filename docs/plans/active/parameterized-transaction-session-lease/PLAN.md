# Parameterized Transaction Session Lease for PostgreSQL and MySQL

## State

`IMPLEMENTING`

## Goal

Ensure `PostgresConnector::execute_parameterized_transaction` and `MySqlConnector::execute_parameterized_transaction` check out and join active query tab transaction session leases (`entry.open_session`) via `checkout_if_held` instead of running parameterized table mutations on separate pool connections, preserving session isolation and transactional integrity.

## Problem Statement

When a query tab in DB Pro executes an explicit transaction (`BEGIN`), the connection is held in `entry.open_session` for subsequent statements on that tab.
However, `PostgresConnector::execute_parameterized_transaction` and `MySqlConnector::execute_parameterized_transaction` currently ignore `entry.open_session` and always acquire a new connection from the pool (`pool.begin().await`).
As a result:
1. Staged table row mutations executed while a query tab has an open transaction run on a completely separate database connection/session.
2. The mutation cannot see uncommitted changes from the active transaction tab (or vice-versa).
3. If an explicit transaction is open on the tab, executing table row mutations or parameterized updates does not run within that transaction context, violating the session transaction boundary.
4. Furthermore, if `execute_parameterized_transaction` fails on a standalone connection, it calls `tx.rollback().await`, whereas if it were joined to an outer transaction session, a full rollback would destroy the caller's outer transaction.

## Evidence & Failure Scenario

- **Code Path (PostgreSQL)**: `PostgresConnector::execute_parameterized_transaction` in `crates/infrastructure/src/postgres/connector.rs`. Notice that `execute_transaction` checks `checkout_if_held(session, ...)`, while `execute_parameterized_transaction` directly calls `pool.begin().await`.
- **Code Path (MySQL)**: `MySqlConnector::execute_parameterized_transaction` in `crates/infrastructure/src/mysql/connector.rs`. Notice that `execute_transaction` checks `checkout_if_held(session, ...)`, while `execute_parameterized_transaction` directly calls `pool.begin().await`.
- **Failure Scenario**:
  1. User opens a query tab and issues `BEGIN`. The connection is saved in `entry.open_session`.
  2. User makes a staged edit in the table data view for that same query session.
  3. `execute_parameterized_transaction` acquires a new pool connection `B` while the tab session holds connection `A`.
  4. Connection `B` cannot see uncommitted changes made on `A`, causing false lock timeouts or foreign key constraint violations.
  5. Statements executed on connection `B` commit independently of connection `A`'s transaction, breaking atomicity and transaction boundary isolation.
- **Severity**: P1 (database mutation correctness & transaction/rollback defect).

## Scope

1. Update `PostgresConnector::execute_parameterized_transaction` in `crates/infrastructure/src/postgres/connector.rs`:
   - Check out `entry.open_session` via `checkout_if_held`.
   - If held, execute statements in `execute_parameterized_transaction_joined` using `SAVEPOINT dbpro_parameterized_tx`.
   - On error or invariant failure (`expect_affected_rows` / `max_affected_rows`), execute `ROLLBACK TO SAVEPOINT dbpro_parameterized_tx` and return `TransactionFailureOutcome::LeftOpen`.
   - On success, execute `RELEASE SAVEPOINT dbpro_parameterized_tx` and mark `lease.succeed()`.
2. Update `MySqlConnector::execute_parameterized_transaction` in `crates/infrastructure/src/mysql/connector.rs`:
   - Check out `entry.open_session` via `checkout_if_held`.
   - If held, execute statements in `execute_parameterized_transaction_joined` using `SAVEPOINT dbpro_parameterized_tx`.
   - On error or invariant failure, execute `ROLLBACK TO SAVEPOINT dbpro_parameterized_tx` and return `TransactionFailureOutcome::LeftOpen`.
   - On success, execute `RELEASE SAVEPOINT dbpro_parameterized_tx` and mark `lease.succeed()`.
3. Add regression unit tests in `crates/infrastructure/src/postgres/tests/connector_tests.rs` and `crates/infrastructure/src/mysql/tests/connector_tests.rs`.
