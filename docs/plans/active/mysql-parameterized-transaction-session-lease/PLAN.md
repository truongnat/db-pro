# Plan: MySQL Parameterized Transaction Session Lease Alignment

## Problem Statement
`MySqlConnector::execute_parameterized_transaction` previously called `pool.begin().await` directly without invoking `checkout_if_held(session, ...)`. This created a provider capability mismatch with `PostgresConnector::execute_parameterized_transaction`, which checks `checkout_if_held` to join an open session lease whenever a task context carries `active_query_session()`.

In addition, when executing parameterized transactions within an open session lease, statement or invariant failure rolls back to savepoint (`ROLLBACK TO SAVEPOINT dbpro_tx_<seq>`). In MySQL, `ROLLBACK TO SAVEPOINT` does NOT automatically release the savepoint identifier (unlike PostgreSQL), so an explicit `RELEASE SAVEPOINT dbpro_tx_<seq>` is required after rollback to prevent savepoint identifier accumulation on open outer transactions.

## Transaction Scope Design Clarification
As documented in `crates/core/src/application/query_session.rs`, Data Grid table mutations dispatched via `RuntimeCommand::ApplyTableChanges` run as standalone atomic transactions without joining a query tab's open transaction (`session_id: None`). When a task scope sets `with_query_session`, parameterized transactions join the open session lease.

## Severity
**P1**: Provider implementation mismatch / broken transaction semantics for session-scoped parameterized operations.

## Implementation Strategy
1. In `crates/infrastructure/src/mysql/connector.rs`:
   - Implement `execute_parameterized_transaction_joined`:
     - Query `SELECT CONNECTION_ID()` and track query cancellation via `track_guard`.
     - Issue `SAVEPOINT dbpro_tx_<seq>` via `crate::transaction_session::savepoint_name()`.
     - Bind parameters and execute statements sequentially.
     - Check `expect_affected_rows` and `max_affected_rows` invariants for each statement.
     - On failure, issue `ROLLBACK TO SAVEPOINT dbpro_tx_<seq>` and `RELEASE SAVEPOINT dbpro_tx_<seq>` so savepoints do not accumulate on open outer transactions.
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
