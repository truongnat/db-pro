# Findings: MySQL Parameterized Transaction Active Connection Tracking

## Root Cause
In `crates/infrastructure/src/mysql/connector.rs`, `MySqlConnector::execute_parameterized_transaction` creates a transaction via `pool.begin().await` but does not query `SELECT CONNECTION_ID()` to populate `active_connections`.

## Impact
Query cancellation calls (`cancel(&ConnectionHandle)`) rely on `active_connections` containing the server connection thread ID to issue `KILL QUERY <connection_id>`. Without connection tracking during parameterized transactions, table mutations (staged edits, inserts, deletes) cannot be canceled by the user if they hang or take a long time to complete.
