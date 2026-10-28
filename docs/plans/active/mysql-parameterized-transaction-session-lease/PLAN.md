# Feature Plan: MySQL Parameterized Transaction Session Lease

## Slug
`mysql-parameterized-transaction-session-lease`

## Summary
In `MySqlConnector::execute_parameterized_transaction`, active session leases (`SessionLease`) held by an open query tab transaction were ignored, causing `execute_parameterized_transaction` to open a separate pool connection via `pool.begin()`. Parameterized table row mutations thus executed outside the tab's active transaction and were not reverted on tab `ROLLBACK`.

This plan implements `execute_parameterized_transaction_joined` in `MySqlConnector`. It uses `sqlx::raw_sql` for `SAVEPOINT`, `ROLLBACK TO SAVEPOINT`, and `RELEASE SAVEPOINT` to execute savepoints over MySQL's text protocol (`COM_QUERY`), avoiding prepared-statement protocol Error 1295 (`This command is not supported in the prepared statement protocol yet`).

## Target Files
- `crates/infrastructure/src/mysql/connector.rs`
- `crates/infrastructure/src/mysql/tests/connector_tests.rs`
- `docs/plans/active/mysql-parameterized-transaction-session-lease/*`

## Verification
- Unit tests in `crates/infrastructure/src/mysql/tests/connector_tests.rs`
- Quality gates: `cargo fmt`, `cargo check --workspace`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test -p db-pro-infrastructure`
