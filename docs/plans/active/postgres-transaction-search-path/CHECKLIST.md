# Checklist: PostgreSQL Parameterized Transaction Search Path Isolation

- [ ] Add `set_local_search_path(&mut tx).await` in `PostgresConnector::execute_parameterized_transaction`.
- [ ] Roll back transaction and report `TransactionFailurePhase::Begin` on `set_local_search_path` failure.
- [ ] Add unit test verifying search path isolation during parameterized transaction execution.
- [ ] Run `cargo test -p db-pro-core` and `cargo test -p db-pro-infrastructure` (with `libdbus-sys` vendored feature enabled if needed).
- [ ] Perform pre-commit checks.
