# Plan: Parameterized Transaction Session Lease & Savepoint Isolation in Postgres and MySQL Connectors

## Problem Statement & Evidence
In `PostgresConnector::execute_parameterized_transaction` (`crates/infrastructure/src/postgres/connector.rs`) and `MySqlConnector::execute_parameterized_transaction` (`crates/infrastructure/src/mysql/connector.rs`), the connector implementations ignore `open_session` and do not invoke `checkout_if_held`.

When a transaction is currently open on a session (e.g. user initiated `BEGIN` in a query tab), calling `execute_parameterized_transaction` (used by `TableDataService` for row insertions, updates, and deletions) creates a separate, new connection/transaction from the pool.

This leads to two critical defects:
1. **Isolation & Lock Contention Defect**: Mutations performed in `execute_parameterized_transaction` do not see uncommitted changes from the active session transaction and can block on row locks or foreign key constraints held by the session.
2. **Outer Transaction Destruction**: If `execute_parameterized_transaction` were to run on the leased connection using `BEGIN`, MySQL would implicitly commit the outer transaction, while PostgreSQL would emit errors. If a statement or invariant (`expect_affected_rows`, `max_affected_rows`) fails without savepoint isolation, rolling back would destroy the entire outer session transaction rather than just rolling back the staged batch mutations.

## Severity
**P1** - Database mutation correctness & transaction/rollback defect across PostgreSQL and MySQL connectors.

## Scope
1. Update `PostgresConnector::execute_parameterized_transaction` in `crates/infrastructure/src/postgres/connector.rs` to check out an active session lease using `checkout_if_held`. When joined to an active session lease, execute mutations inside `SAVEPOINT dbpro_parameterized_tx`. On statement or invariant failure, execute `ROLLBACK TO SAVEPOINT dbpro_parameterized_tx` and return `TransactionFailureOutcome::LeftOpen`.
2. Update `MySqlConnector::execute_parameterized_transaction` in `crates/infrastructure/src/mysql/connector.rs` to similarly check out active session leases using `checkout_if_held` and protect execution via `SAVEPOINT dbpro_parameterized_tx`.
3. Add unit and regression tests in `crates/infrastructure/src/postgres/tests/connector_tests.rs` and `crates/infrastructure/src/mysql/tests/connector_tests.rs`.
4. Run all Rust quality gates (`cargo fmt`, `cargo check`, `cargo clippy`, `cargo test`).
