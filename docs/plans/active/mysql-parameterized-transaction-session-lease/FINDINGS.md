# Findings — MySQL Parameterized Transaction Session Lease

## Defect Summary
In `MySqlConnector::execute_parameterized_transaction`, active session leases (`SessionLease`) held by an open query tab transaction were ignored, causing `execute_parameterized_transaction` to open a separate pool connection via `pool.begin()`.

## Root Cause & Failure Scenario
1. User opens a transaction in a MySQL Query tab via `BEGIN;`. The connection is checked out into `open_session`.
2. User performs a row mutation in the data editor (which uses `execute_parameterized_transaction`).
3. `MySqlConnector::execute_parameterized_transaction` calls `pool.begin()`, acquiring a second connection from the pool.
4. Mutation executes and commits on the second connection.
5. User executes `ROLLBACK;` on the Query tab.
6. The tab rolls back, but the row mutation on the second connection was already committed.

## Protocol Detail for MySQL Savepoints
MySQL 8.x supports `SAVEPOINT <name>`, `ROLLBACK TO SAVEPOINT <name>`, and `RELEASE SAVEPOINT <name>` in the prepared statement protocol (`COM_STMT_PREPARE`), whereas `BEGIN` or `START TRANSACTION` return `ERROR 1295`.
Using `sqlx::query(&format!("SAVEPOINT {savepoint}"))` executes savepoint transaction control cleanly on MySQL leased connections.

## Severity
P1 (database mutation correctness, transaction isolation failure, provider parity).
