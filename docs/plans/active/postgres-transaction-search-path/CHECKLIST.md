# Checklist: PostgreSQL Parameterized Transaction Search Path Correctness

- [x] Identify root cause: `PostgresConnector::execute_parameterized_transaction` missing `set_local_search_path(&mut tx).await`.
- [x] Add `set_local_search_path(&mut tx).await` call immediately after `pool.begin().await` in `execute_parameterized_transaction`.
- [x] Handle `set_local_search_path` failure by rolling back transaction and returning `TransactionFailurePhase::Begin` with `statement_index = 0`.
- [x] Add integration test in `crates/infrastructure/tests/pg_integration.rs` testing parameterized transaction active search path resolution.
- [x] Verify Rust quality gates (`cargo fmt`, `cargo check`, `cargo clippy`, `cargo test`).
- [x] Run clean code scanner (`python3 tools/cc-scan.py`).
- [x] Publish PR against `main`.
