# Checklist — MySQL Parameterized Transaction Session Lease

- [x] Create plan artifacts in `docs/plans/active/mysql-parameterized-transaction-session-lease/`
- [x] Implement `execute_parameterized_transaction_joined` in `crates/infrastructure/src/mysql/connector.rs`
- [x] Check `checkout_if_held` in `execute_parameterized_transaction`
- [x] Use `sqlx::query` for MySQL savepoint statements (`SAVEPOINT`, `ROLLBACK TO SAVEPOINT`, `RELEASE SAVEPOINT`)
- [x] Register connection ID with `track_guard` for cancellation
- [x] Handle preflight failures with `TransactionFailurePhase::Begin` and `statement_index = 0`
- [x] Add unit tests in `crates/infrastructure/src/mysql/tests/connector_tests.rs`
- [x] Run Rust quality gates and clean code scan
- [ ] Create branch `fix/mysql-parameterized-transaction-session-lease` and submit PR
