# Findings — Parameterized Transaction Session Lease for PostgreSQL and MySQL

## Code Analysis

1. `SessionSlot` & `SessionLease` (`crates/infrastructure/src/transaction_session.rs`):
   - `checkout_if_held(session, acquire)` checks if the active query session task-local (`active_query_session`) holds an active connection in the `SessionSlot`.
   - If held, it takes out the connection and wraps it in a `SessionLease`.
   - `execute_transaction` in both `PostgresConnector` and `MySqlConnector` already uses `checkout_if_held` to join an open transaction session.

2. `PostgresConnector::execute_parameterized_transaction`:
   - Currently ignores `entry.open_session` and calls `pool.begin().await` directly.
   - Fix: Call `checkout_if_held(session, ...)` first. If `Some(lease)` is returned:
     - Derive active backend PID via `SELECT pg_backend_pid()`.
     - Register `track_backend`.
     - Call `push_session_search_path`.
     - Execute `SAVEPOINT dbpro_parameterized_tx`.
     - For each statement, execute parameterized query and check `expect_affected_rows` and `max_affected_rows`.
     - On error, execute `ROLLBACK TO SAVEPOINT dbpro_parameterized_tx` and return `TransactionFailureOutcome::LeftOpen`.
     - On success, execute `RELEASE SAVEPOINT dbpro_parameterized_tx` and set `lease.succeed()`.

3. `MySqlConnector::execute_parameterized_transaction`:
   - Currently ignores `entry.open_session` and calls `pool.begin().await` directly.
   - Fix: Call `checkout_if_held(session, ...)` first. If `Some(lease)` is returned:
     - Derive active connection ID via `SELECT CONNECTION_ID()`.
     - Register `track_guard`.
     - Execute `SAVEPOINT dbpro_parameterized_tx`.
     - For each statement, execute parameterized query and check `expect_affected_rows` and `max_affected_rows`.
     - On error, execute `ROLLBACK TO SAVEPOINT dbpro_parameterized_tx` and return `TransactionFailureOutcome::LeftOpen`.
     - On success, execute `RELEASE SAVEPOINT dbpro_parameterized_tx` and set `lease.succeed()`.
