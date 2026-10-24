# Checklist: Parameterized Transaction Session Lease & Savepoint Isolation

- [ ] Implement `checkout_if_held` in `PostgresConnector::execute_parameterized_transaction`
- [ ] Implement `execute_parameterized_transaction_joined` with `SAVEPOINT dbpro_parameterized_tx` in `PostgresConnector`
- [ ] Implement `checkout_if_held` in `MySqlConnector::execute_parameterized_transaction`
- [ ] Implement `execute_parameterized_transaction_joined` with `SAVEPOINT dbpro_parameterized_tx` in `MySqlConnector`
- [ ] Add unit tests for `PostgresConnector` session lease checkout and savepoint behavior
- [ ] Add unit tests for `MySqlConnector` session lease checkout and savepoint behavior
- [ ] Run `cargo fmt --all -- --check`
- [ ] Run `cargo check --workspace`
- [ ] Run `cargo clippy --workspace --all-targets -- -D warnings`
- [ ] Run `cargo test --workspace`
