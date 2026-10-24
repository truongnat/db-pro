# Verification: Parameterized Transaction Session Lease & Savepoint Isolation

## Verification Plan
1. Unit tests in `crates/infrastructure/src/postgres/tests/connector_tests.rs` and `crates/infrastructure/src/mysql/tests/connector_tests.rs` verifying source structure and runtime execution when `checkout_if_held` is used with `SAVEPOINT dbpro_parameterized_tx`.
2. Workspace quality gates pass (`cargo fmt`, `cargo check`, `cargo clippy`, `cargo test`).
