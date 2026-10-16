# Verification — SQL Server Parameterized Transaction Semantics

## Quality Gates

- `cargo check --workspace` (PASS)
- `cargo clippy --workspace --all-targets -- -D warnings` (PASS)
- `cargo test --workspace` (PASS, 665 passed)
- `python3 tools/cc-scan.py . --fail-on error` (PASS)

## Verification Evidence

- Automated unit tests in `crates/infrastructure/src/sqlserver/tests/connector_tests.rs`:
  - `execute_parameterized_transaction_reports_zero_statement_index_on_begin_failure`
  - `execute_parameterized_transaction_empty_statements_and_unconnected_handle`
  verifying that `execute_parameterized_transaction` queries `@@TRANCOUNT` during the initial `Begin` phase and correctly returns `TransactionFailurePhase::Begin` with `statement_index = 0` and `TransactionFailureOutcome::NotStarted` on failure.
