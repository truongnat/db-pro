# Checklist: Postgres Parameterized Transaction Search Path Fix

- [ ] Create plan documentation under `docs/plans/active/postgres-parameterized-transaction-search-path/`
- [ ] Implement `set_local_search_path` and `track_backend` in `PostgresConnector::execute_parameterized_transaction`
- [ ] Add unit test in `crates/infrastructure/src/postgres/tests/connector_tests.rs`
- [ ] Add integration test in `crates/infrastructure/tests/pg_integration.rs`
- [ ] Verify `cargo check --workspace`
- [ ] Verify `cargo clippy --workspace --all-targets -- -D warnings`
- [ ] Verify `cargo test --workspace`
- [ ] Verify `tools/cc-scan.py` clean code scan
- [ ] Update `docs/plans/STATUS.md`
