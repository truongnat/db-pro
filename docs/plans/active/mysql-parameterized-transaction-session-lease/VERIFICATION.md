# Verification: MySQL Parameterized Transaction Session Lease

## Quality Gates
- `cargo fmt --all -- --check`
- `cargo check --workspace`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo test -p db-pro-infrastructure`

## Test Evidence
- Unit test `mysql_joined_parameterized_transaction_inspects_session_lease` in `crates/infrastructure/src/mysql/tests/connector_tests.rs`.
- Integration tests in `crates/infrastructure/tests/mysql_integration.rs`.
