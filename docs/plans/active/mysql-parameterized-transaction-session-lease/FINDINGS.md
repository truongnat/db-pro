# FINDINGS — MySQL Parameterized Transaction Session Lease

## Severity: P1 (Data Safety / Transaction Semantics)

### Description
In `crates/infrastructure/src/mysql/connector.rs`, `MySqlConnector::execute_parameterized_transaction` opens a new connection transaction (`pool.begin()`) regardless of whether the target query session/tab has an active open transaction session (`open_session`).

### Evidence & Impact
1. `MySqlConnector::execute_transaction` already checks `checkout_if_held` and delegates to `execute_transaction_joined`.
2. `PostgresConnector::execute_parameterized_transaction` also checks `checkout_if_held` and delegates to `execute_parameterized_transaction_joined`.
3. `MySqlConnector::execute_parameterized_transaction` omits `checkout_if_held`. Consequently, staged table mutations (Data Grid changes) executed within an active MySQL query transaction tab bypass the open transaction session, execute on a separate connection, and commit independently of the tab's transaction state (or block/deadlock due to row locks).

### Root Cause
`MySqlConnector::execute_parameterized_transaction` was not updated when session lease joining was introduced for parameterized transactions across connectors.
