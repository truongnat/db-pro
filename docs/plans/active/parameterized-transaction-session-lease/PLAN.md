# Plan: Parameterized Transaction Session Lease Correctness

## Overview
In `MySqlConnector::execute_parameterized_transaction` and `PostgresConnector::execute_parameterized_transaction`, execution always begins a new transaction on an unleased pool connection (`pool.begin().await`), ignoring any active transaction session (`open_session` / `SessionSlot`) associated with the active query session.

When a query tab holds an open session transaction (e.g., after executing `BEGIN;`), subsequent parameterized operations (such as table cell edits via `TableDataService::apply_mutations`) execute on a separate pool connection outside the session transaction. This breaks transaction isolation, risks row lock contention / timeouts, and causes table mutations to commit independently of the active session transaction.

This feature updates both `MySqlConnector` and `PostgresConnector` to check `checkout_if_held` in `execute_parameterized_transaction`, executing parameterized statements within the active session connection using savepoint isolation when a session lease is held.

## Objectives
- Integrate `checkout_if_held` into `MySqlConnector::execute_parameterized_transaction`.
- Integrate `checkout_if_held` into `PostgresConnector::execute_parameterized_transaction`.
- Wrap parameterized transaction statements in `SAVEPOINT dbpro_parameterized_tx` when running on a joined session lease.
- Ensure rollback to savepoint on statement errors or invariant violations (`expect_affected_rows` / `max_affected_rows`).
- Add unit test coverage for session lease checkout in both MySQL and PostgreSQL connectors.
