# Parameterized Transaction Session Isolation

Lifecycle: IMPLEMENTING
Branch: `fix/parameterized-transaction-session-isolation`

## Goal
Fix `execute_parameterized_transaction` in both `MySqlConnector` and `PostgresConnector` so that when a connection has an active transaction session lease (`open_session`), parameterized operations (such as staged table mutations) run within savepoints on the leased connection instead of acquiring a new connection from the pool and executing an unisolated transaction outside the active session.

## Problem Statement
- **Severity**: P1 (Broken transaction semantics / Database mutation isolation failure)
- **Evidence**:
  - `MySqlConnector::execute_parameterized_transaction` (`crates/infrastructure/src/mysql/connector.rs:555`) and `PostgresConnector::execute_parameterized_transaction` (`crates/infrastructure/src/postgres/connector.rs:806`) call `pool.begin().await` directly without invoking `crate::transaction_session::checkout_if_held`.
  - In contrast, `execute_transaction` and `execute_batch` in both connectors properly call `checkout_if_held` (e.g. `mysql/connector.rs:436`, `postgres/connector.rs:645`) to run inside active session leases.
- **Failure Scenario**:
  A user opens an explicit transaction in the Query Editor (`BEGIN;`). The connection's `open_session` holds a connection lease. The user then performs staged row edits/deletions in the Table Data Editor, which routes through `TableDataService` -> `execute_parameterized_transaction`. Because `execute_parameterized_transaction` ignores `open_session`, it acquires a second connection from `pool`, opens an independent transaction, mutates rows, and commits them immediately. The user's explicit session transaction still sees uncommitted/stale state or locks, and rolling back `BEGIN` fails to undo the staged table mutations because they were committed in a separate pool connection.
