# PLAN — MySQL Parameterized Transaction Session Lease

## Summary
`MySqlConnector::execute_parameterized_transaction` currently ignores open session leases held by a query tab and executes parameterized batch transactions (such as Data Grid table mutations) on a newly acquired connection via `pool.begin()`. In contrast, `PostgresConnector` checks out open session leases (`checkout_if_held`) and uses savepoints (`SAVEPOINT dbpro_tx_<seq>`). This creates a P1 transaction isolation / session leak defect where table mutations in a MySQL query tab with an active transaction execute outside the tab's transaction session on a separate pool connection.

This plan addresses this capability mismatch and defect by implementing session lease joining for `MySqlConnector::execute_parameterized_transaction`.

## Proposed Solution
1. Add `execute_parameterized_transaction_joined` in `MySqlConnector` (`crates/infrastructure/src/mysql/connector.rs`).
2. Call `checkout_if_held` before `pool.begin()`. If a session lease is held, delegate execution to `execute_parameterized_transaction_joined`.
3. In `execute_parameterized_transaction_joined`:
   - Acquire connection_id (`SELECT CONNECTION_ID()`) and register active tracking via `track_guard`.
   - Issue `SAVEPOINT dbpro_tx_<seq>`.
   - Execute parameterized statements inside the savepoint context.
   - On statement error or invariant failure (zero affected rows when expected, max affected rows exceeded), execute `ROLLBACK TO SAVEPOINT dbpro_tx_<seq>` and `RELEASE SAVEPOINT dbpro_tx_<seq>`.
   - Return appropriate `TransactionFailure` with `TransactionFailureOutcome::LeftOpen` (or `Unknown` if rollback fails).
   - On success, execute `RELEASE SAVEPOINT dbpro_tx_<seq>` and mark lease succeeded.
4. Add regression tests covering session lease joining and savepoint rollback for MySQL parameterized transactions.
