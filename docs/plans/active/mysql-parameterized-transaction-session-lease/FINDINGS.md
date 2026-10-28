# Findings: MySQL Parameterized Transaction Session Lease Joining & Savepoint Cleanup

## Root Cause Analysis
1. `MySqlConnector::execute_parameterized_transaction` directly calls `pool.begin().await` to acquire a connection and start a new transaction without checking `open_session` via `checkout_if_held`.
2. In `execute_parameterized_transaction_joined`, rolling back to a savepoint via `ROLLBACK TO SAVEPOINT dbpro_tx_<seq>` restores the state to the savepoint but does not remove the savepoint in MySQL (unlike PostgreSQL). If an outer transaction remains open, savepoint identifiers accumulate unless `RELEASE SAVEPOINT` is issued after `ROLLBACK TO SAVEPOINT`.

## Architectural Verification
- `ApplyTableChanges` command in UI / worker runs mutations via `TableDataService::apply_mutations_detailed`, which executes `connector.execute_parameterized_transaction`.
- When `with_query_session` is active in the worker task, `checkout_if_held` resolves `SessionSlot` using `active_query_session()`.
- Joining active session transactions via `checkout_if_held` ensures Data Grid mutations execute inside the user's active transaction when `BEGIN;` was run in the active query tab.
