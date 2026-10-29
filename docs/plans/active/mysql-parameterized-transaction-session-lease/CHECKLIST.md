# CHECKLIST — MySQL Parameterized Transaction Session Lease

- [x] Create `PLAN.md`, `CHECKLIST.md`, `FINDINGS.md`, `VERIFICATION.md` under `docs/plans/active/mysql-parameterized-transaction-session-lease/`
- [x] Implement `execute_parameterized_transaction_joined` in `MySqlConnector` (`crates/infrastructure/src/mysql/connector.rs`)
- [x] Wire `checkout_if_held` in `MySqlConnector::execute_parameterized_transaction`
- [x] Implement savepoint creation (`SAVEPOINT dbpro_tx_<seq>`), rollback, and release logic
- [x] Add unit test for `execute_parameterized_transaction` joined lease handling in `mysql/tests/connector_tests.rs`
- [x] Run `cargo check --workspace`, `cargo clippy -p db-pro-infrastructure --all-targets -- -D warnings`, `cargo test -p db-pro-infrastructure`
- [x] Perform pre-commit checks and submit PR
