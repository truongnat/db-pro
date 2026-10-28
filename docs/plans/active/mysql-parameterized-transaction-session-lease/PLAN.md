# Plan: MySQL Parameterized Transaction Session Lease

## State
`IMPLEMENTING`

## Goal
Make `MySqlConnector::execute_parameterized_transaction` join the query tab's active session lease (`open_session`) when a transaction is open, using savepoints (`SAVEPOINT dbpro_tx_<seq>`) on the leased connection instead of acquiring a separate connection from the pool.

## Problem Statement
When a query tab executes an explicit transaction (`BEGIN`), the connection is held in `entry.open_session` for subsequent statements on that tab.
`MySqlConnector::execute_parameterized_transaction` currently ignores `entry.open_session` and acquires a separate connection via `pool.begin()`. As a result:
1. Staged table mutations (Data Grid edits, row updates/deletions) executed while the query tab has an open transaction run on a separate database connection.
2. The mutation cannot see uncommitted changes from the active transaction session (and vice versa).
3. If the user issues `ROLLBACK` in the query tab, the staged edit on the separate connection remains committed.
4. If the active transaction holds exclusive row locks, the staged edit on the separate connection blocks or deadlocks.

## Severity
- **Severity**: P1 (Data Corruption / Wrong Database Mutation / Broken Transaction Semantics).

## Failure Scenario
1. A user connects to a MySQL database and executes `BEGIN;` in a query tab.
2. In the Data Grid view for a table, the user edits a cell and saves changes. This invokes `MySqlConnector::execute_parameterized_transaction`.
3. `MySqlConnector::execute_parameterized_transaction` opens connection #2 from the pool and commits changes on connection #2 immediately.
4. The user executes `ROLLBACK;` in the query tab. The tab transaction on connection #1 rolls back, but the mutation on connection #2 was committed separately.

## Scope & Solution
1. Update `MySqlConnector::execute_parameterized_transaction` in `crates/infrastructure/src/mysql/connector.rs`:
   - Check out `entry.open_session` via `checkout_if_held`.
   - When a lease is held, delegate to `execute_parameterized_transaction_joined`.
   - In `execute_parameterized_transaction_joined`:
     - Query `SELECT CONNECTION_ID()` and track active connection for cancellation (`track_guard`).
     - Create a savepoint using `crate::transaction_session::savepoint_name()` (`dbpro_tx_<seq>`).
     - Loop over `statements`, executing each statement and checking `expect_affected_rows` / `max_affected_rows`.
     - On statement or invariant failure, execute `ROLLBACK TO SAVEPOINT` and `RELEASE SAVEPOINT`. On rollback/release success, return `TransactionFailure` with `phase: Statement`, `statement_index`, and `outcome: LeftOpen`. On rollback/release failure, return `outcome: Unknown` and call `lease.discard_if_abandoned()`.
     - On success, execute `RELEASE SAVEPOINT`, call `lease.succeed()`, and return `Ok(results)`.
2. Add unit and integration regression tests.
