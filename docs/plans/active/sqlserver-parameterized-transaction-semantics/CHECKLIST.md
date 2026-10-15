# Checklist — SQL Server Parameterized Transaction Semantics

- [x] Check `open_transaction_count(handle)` in `SqlServerConnector::execute_parameterized_transaction`
- [x] Skip `BEGIN TRANSACTION` and `COMMIT TRANSACTION` when `already_open` is true
- [x] Return `TransactionFailureOutcome::LeftOpen` without `ROLLBACK TRANSACTION` on error when `already_open` is true
- [x] Add unit test for `execute_parameterized_transaction` on unconnected handle
- [x] Run `cargo check --workspace`
- [x] Run `cargo clippy --workspace --all-targets -- -D warnings`
- [x] Run `cargo test --workspace`
- [x] Run clean code audit `python3 tools/cc-scan.py . --fail-on error`
