# Plan: MySQL Parameterized Transaction Active Connection Tracking

## State
`IMPLEMENTING`

## Problem & Severity
- **Severity**: P1 (Database mutation correctness, query cancellation & provider defect on MySQL).
- **Problem**: In `MySqlConnector::execute_parameterized_transaction` (`crates/infrastructure/src/mysql/connector.rs`), parameterized transaction batches begin a `sqlx::Transaction` but fail to query `SELECT CONNECTION_ID()` and hold a `track_guard` (`ActiveMySql`) guard during statement execution. As a result, when a user cancels a running parameterized table mutation (such as a large batch row update or delete), `MySqlConnector::cancel` finds an empty `active_connections` list and fails to issue `KILL QUERY <connection_id>`, leaving the MySQL server executing the mutation un-cancelable from DB Pro.
- **Evidence**: `MySqlConnector::execute_parameterized_transaction` in `crates/infrastructure/src/mysql/connector.rs` lacks the `SELECT CONNECTION_ID()` tracking call present in `MySqlConnector::query`, `MySqlConnector::execute`, `MySqlConnector::execute_transaction`, and `PostgresConnector::execute_parameterized_transaction`.

## Failure Scenario
1. A user connects to a MySQL database and opens a table in the Table Data Editor.
2. The user stages mutations affecting many rows (e.g., updating or deleting thousands of rows in a table).
3. `TableDataService::apply_mutations` invokes `MySqlConnector::execute_parameterized_transaction`.
4. While the transaction is running on MySQL, the user clicks "Stop" / Cancel in DB Pro.
5. `MySqlConnector::cancel` reads `entry.active_connections`, which is empty because `execute_parameterized_transaction` never registered its connection ID.
6. The cancellation request silently succeeds without executing `KILL QUERY <connection_id>`, and the MySQL server continues processing the transaction unhindered until completion.

## Scope & Solution
1. Update `MySqlConnector::execute_parameterized_transaction` in `crates/infrastructure/src/mysql/connector.rs`:
   - Query `SELECT CONNECTION_ID()` on `&mut tx` after `pool.begin().await`.
   - Acquire and hold `_active = match sqlx::query_scalar::<_, u64>("SELECT CONNECTION_ID()").fetch_one(&mut *tx).await { Ok(id) => Some(self.track_guard(handle.0, id)), Err(_) => None }`.
2. Add regression unit test in `crates/infrastructure/src/mysql/tests/connector_tests.rs`.
3. Update `docs/plans/STATUS.md`.
