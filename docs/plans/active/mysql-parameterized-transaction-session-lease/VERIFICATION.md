# Verification — MySQL Parameterized Transaction Session Lease

## Test Plan
1. Unit tests in `crates/infrastructure/src/mysql/tests/connector_tests.rs`:
   - Verify `execute_parameterized_transaction` dispatches to `checkout_if_held`.
   - Verify `execute_parameterized_transaction_joined` handles savepoint rollback and error phases.

2. Quality Gates:
   - `cargo fmt --all -- --check`
   - `cargo check --workspace`
   - `cargo clippy --workspace --all-targets -- -D warnings`
   - `cargo test -p db-pro-infrastructure`
   - `bash scripts/check-ui-architecture.sh`
   - `bash .skills/clean-code/scripts/clean-code-scan.sh rust --diff --ratchet --ci`
