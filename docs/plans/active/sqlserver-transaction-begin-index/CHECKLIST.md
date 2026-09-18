# Checklist — SQL Server Transaction Begin Index Fix

- [ ] Update `execute_transaction` in `crates/infrastructure/src/sqlserver/connector.rs` to set `statement_index: 0` for `Validation` phase.
- [ ] Update `execute_transaction` in `crates/infrastructure/src/sqlserver/connector.rs` to set `statement_index: 0` for `Begin` phase.
- [ ] Update `execute_parameterized_transaction` in `crates/infrastructure/src/sqlserver/connector.rs` to set `statement_index: 0` for `Begin` phase.
- [ ] Add unit test `execute_transaction_reports_zero_statement_index_on_validation_failure`.
- [ ] Add unit test `execute_transaction_reports_zero_statement_index_on_begin_failure`.
- [ ] Add unit test `execute_parameterized_transaction_reports_zero_statement_index_on_begin_failure`.
- [ ] Verify unit tests with `cargo test -p db-pro-core`.
- [ ] Pass all quality gates.
