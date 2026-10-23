# Verification — Parameterized Transaction Session Isolation

## Strategy
- **Unit Source Contract Verification**:
  - `execute_parameterized_transaction_publishes_connection_id_before_statements` in `crates/infrastructure/src/mysql/tests/connector_tests.rs` verifies that `checkout_if_held` is invoked before transaction statement processing.
  - `owned_execute_parameterized_transaction_publishes_backend_and_sets_search_path` in `crates/infrastructure/src/postgres/tests/connector_tests.rs` verifies that `checkout_if_held` is invoked before transaction statement processing.
- **Automated Test Suite Results**:
  - `cargo +nightly fmt --all -- --check` -> **PASS**
  - `cargo check --workspace` -> **PASS**
  - `cargo clippy --workspace --all-targets -- -D warnings` -> **PASS** (0 warnings)
  - `cargo test -p db-pro-infrastructure` -> **PASS** (32 passed, 0 failed, 12 mysql ignored due to missing live fixture, 40 pg ignored due to missing live fixture)
