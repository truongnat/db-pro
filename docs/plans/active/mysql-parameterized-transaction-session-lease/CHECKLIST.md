# Checklist: MySQL Parameterized Transaction Session Lease

- [ ] Implement `execute_parameterized_transaction_joined` in `crates/infrastructure/src/mysql/connector.rs`
- [ ] Call `checkout_if_held` in `MySqlConnector::execute_parameterized_transaction`
- [ ] Add unit tests in `crates/infrastructure/src/mysql/tests/connector_tests.rs`
- [ ] Add integration tests in `crates/infrastructure/tests/mysql_integration.rs`
- [ ] Verify `cargo check --workspace` and `cargo clippy --workspace --all-targets -- -D warnings`
- [ ] Run `cargo test -p db-pro-infrastructure`
