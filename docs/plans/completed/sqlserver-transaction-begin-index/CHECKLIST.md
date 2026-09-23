# Checklist — SQL Server Transaction Begin Index Fix

- [x] Update `execute_transaction` in `crates/infrastructure/src/sqlserver/connector.rs` to set `statement_index: 0` for `Validation` phase.
- [x] Update `execute_transaction` in `crates/infrastructure/src/sqlserver/connector.rs` to set `statement_index: 0` for `Begin` phase.
- [x] Update `execute_parameterized_transaction` in `crates/infrastructure/src/sqlserver/connector.rs` to set `statement_index: 0` for `Begin` phase.
- [x] Add unit test `execute_transaction_reports_zero_statement_index_on_validation_failure`.
- [x] Add unit test `execute_transaction_reports_zero_statement_index_on_begin_failure`.
- [x] Add unit test `execute_parameterized_transaction_reports_zero_statement_index_on_begin_failure`.
- [x] Verify unit tests (`cargo test -p db-pro-infrastructure --lib statement_index` — 3 passed).
- [x] Pass all quality gates.

Implemented in commit `61e97658`; verified on HEAD `aaa7a69e` (2026-09-23).
