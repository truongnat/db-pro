# Checklist — Parameterized Transaction Session Lease for PostgreSQL and MySQL

- [ ] Implement `checkout_if_held` check in `PostgresConnector::execute_parameterized_transaction`
- [ ] Implement `execute_parameterized_transaction_joined` in `PostgresConnector` using `SAVEPOINT dbpro_parameterized_tx` and `ROLLBACK TO SAVEPOINT` on failure
- [ ] Implement `checkout_if_held` check in `MySqlConnector::execute_parameterized_transaction`
- [ ] Implement `execute_parameterized_transaction_joined` in `MySqlConnector` using `SAVEPOINT dbpro_parameterized_tx` and `ROLLBACK TO SAVEPOINT` on failure
- [ ] Add unit tests in `crates/infrastructure/src/postgres/tests/connector_tests.rs`
- [ ] Add unit tests in `crates/infrastructure/src/mysql/tests/connector_tests.rs`
- [ ] Run `cargo check --workspace`
- [ ] Run `cargo clippy --workspace --all-targets -- -D warnings`
- [ ] Run `cargo test -p db-pro-infrastructure --lib`
- [ ] Run clean code audit `python3 tools/cc-scan.py . --fail-on error`
