# Plan: MySQL Parameterized Transaction Session Lease Joining

## Problem Statement
In `crates/infrastructure/src/mysql/connector.rs`, `MySqlConnector::execute_parameterized_transaction` opens a new connection and pool transaction (`pool.begin().await`) for parameterized table row mutations without checking if the active query tab has an open session transaction (via `checkout_if_held`). As a result, table row mutations (such as Data Grid edits executed via `TableMutationExecution`) run on a separate connection outside the transaction opened in the query tab.

## Severity
**P1**: Broken transaction semantics / database mutation correctness / provider capability mismatch.

## Failure Scenario
1. A user opens a query tab and executes `BEGIN;`.
2. The user edits row values in the Table Data Editor / Data Grid, which calls `execute_parameterized_transaction`.
3. The mutation runs on a separate connection from the pool and immediately commits when `execute_parameterized_transaction` finishes.
4. If the user executes `ROLLBACK;` in the query tab, the table data edits are NOT rolled back because they were committed on a separate connection outside the open transaction.

## Implementation Strategy
1. In `crates/infrastructure/src/mysql/connector.rs`:
   - Implement `execute_parameterized_transaction_joined`:
     - Query `SELECT CONNECTION_ID()` and track query cancellation via `track_guard`.
     - Issue `SAVEPOINT dbpro_tx_<seq>` via `crate::transaction_session::savepoint_name()`.
     - Bind parameters and execute statements sequentially.
     - Check `expect_affected_rows` and `max_affected_rows` invariants for each statement.
     - On failure, issue `ROLLBACK TO SAVEPOINT dbpro_tx_<seq>` and return `TransactionFailure` with `outcome: TransactionFailureOutcome::LeftOpen` (handling rollback failure if `ROLLBACK TO SAVEPOINT` fails).
     - On success, issue `RELEASE SAVEPOINT dbpro_tx_<seq>`.
   - Update `execute_parameterized_transaction`:
     - Call `checkout_if_held(session, ...)` before calling `pool.begin()`.
     - If a lease exists, delegate to `execute_parameterized_transaction_joined`.
     - Handle preflight checkout errors returning `phase: TransactionFailurePhase::Begin`, `statement_index: 0`.

2. Add unit tests in `crates/infrastructure/src/mysql/tests/connector_tests.rs`:
   - Verify `execute_parameterized_transaction` checks out active session leases before opening new transactions.

3. Verify quality gates:
   - `cargo fmt --all -- --check`
   - `cargo check --workspace`
   - `cargo clippy --workspace --all-targets -- -D warnings`
   - `cargo test --workspace`
