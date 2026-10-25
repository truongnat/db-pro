# Verification — Parameterized Transaction Session Lease for PostgreSQL and MySQL

## Required Quality Gates

- `cargo check --workspace`: Must exit 0.
- `cargo clippy --workspace --all-targets -- -D warnings`: Must exit 0 warnings.
- `cargo test -p db-pro-infrastructure --lib`: Must pass all unit tests without failures.
- `python3 tools/cc-scan.py . --fail-on error`: Must introduce 0 clean code errors.

## Test Matrix

- **PostgreSQL**:
  - `execute_parameterized_transaction_unconnected_handle_fails_validation`: Unconnected handle returns `TransactionFailurePhase::Validation`.
  - `execute_parameterized_transaction_empty_statements`: Empty statements list returns `Ok(vec![])`.
- **MySQL**:
  - `execute_parameterized_transaction_unconnected_handle_fails_validation`: Unconnected handle returns `TransactionFailurePhase::Validation`.
  - `execute_parameterized_transaction_empty_statements`: Empty statements list returns `Ok(vec![])`.
