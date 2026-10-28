# Findings: MySQL Parameterized Transaction Session Lease Alignment

## Root Cause Analysis
1. `MySqlConnector::execute_parameterized_transaction` directly called `pool.begin().await` without invoking `checkout_if_held`. This differed from `PostgresConnector::execute_parameterized_transaction`, which checks `checkout_if_held` when executing parameterized statements within a session context.
2. In `execute_parameterized_transaction_joined`, MySQL retains savepoint identifiers after `ROLLBACK TO SAVEPOINT dbpro_tx_<seq>`. Explicitly calling `RELEASE SAVEPOINT dbpro_tx_<seq>` after `ROLLBACK TO SAVEPOINT` cleans up server savepoint memory when an outer session transaction stays open (`LeftOpen`).

## Transaction Scope Architecture
- `crates/core/src/application/query_session.rs` defines query session boundaries.
- Data Grid mutations (`RuntimeCommand::ApplyTableChanges`) run without a `session_id`, executing as standalone atomic batch transactions.
- Parameterized operations running under `with_query_session` (e.g. session-scoped tasks or tab operations) check out open leases and execute inside savepoints.
