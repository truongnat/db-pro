# Checklist: MySQL Transaction Read-Statement Execution, Pre-execution Failure Indexing, and Batch Atomicity

- [x] Refactor `MySqlConnector::execute_batch` to delegate to `execute_transaction`.
- [x] Ensure `execute_transaction` and `execute_parameterized_transaction` return `statement_index = 0` on `Validation` and `Begin` failures.
- [x] Add unit test `execute_parameterized_transaction_reports_validation_failure_on_unknown_handle` in `crates/infrastructure/src/mysql/connector.rs`.
- [x] Verify cargo test passes for `db-pro-infrastructure`.
- [x] Verify quality gates (`cargo fmt`, `cargo check`, `cargo clippy`, `cargo test`).
