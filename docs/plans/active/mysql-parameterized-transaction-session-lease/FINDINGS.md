# Findings: MySQL Parameterized Transaction Session Lease Joining

## Root Cause Analysis
`MySqlConnector::execute_parameterized_transaction` directly calls `pool.begin().await` to acquire a connection and start a new transaction without checking `open_session` via `checkout_if_held`.

In contrast, `PostgresConnector::execute_parameterized_transaction` checks `checkout_if_held` and delegates to `execute_parameterized_transaction_joined` when a session is held by the current query tab.

Similarly, `MySqlConnector::execute_transaction` already calls `checkout_if_held` and delegates to `execute_transaction_joined`.

Therefore, `execute_parameterized_transaction` in `MySqlConnector` suffers from a transaction isolation bug where table mutations (such as grid row edits) bypass open transactions in the query tab.
