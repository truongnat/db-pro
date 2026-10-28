# Checklist: MySQL Parameterized Transaction Session Lease Joining

- [ ] Create plan documentation in `docs/plans/active/mysql-parameterized-transaction-session-lease/`
- [ ] Implement `execute_parameterized_transaction_joined` in `crates/infrastructure/src/mysql/connector.rs`
- [ ] Call `checkout_if_held` in `execute_parameterized_transaction` in `crates/infrastructure/src/mysql/connector.rs`
- [ ] Add unit tests in `crates/infrastructure/src/mysql/tests/connector_tests.rs`
- [ ] Verify `cargo fmt --all -- --check`
- [ ] Verify `cargo check --workspace`
- [ ] Verify `cargo clippy --workspace --all-targets -- -D warnings`
- [ ] Verify `cargo test --workspace`
