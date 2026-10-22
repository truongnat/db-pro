# Checklist: Parameterized Transaction Session Lease Correctness

- [x] Implement `checkout_if_held` in `MySqlConnector::execute_parameterized_transaction`
- [x] Implement `checkout_if_held` in `PostgresConnector::execute_parameterized_transaction`
- [x] Add savepoint rollback logic for joined session leases in MySQL and PostgreSQL
- [x] Verify compilation with `cargo check -p db-pro-infrastructure`
- [x] Add unit tests in `mysql/tests/connector_tests.rs` and `postgres/tests/connector_tests.rs`
- [x] Run quality gates and verify tests pass
