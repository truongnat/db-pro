# #481 checklist

- [x] `cancel` uses `query_scalar` + `fetch_one` and checks the returned bool
- [x] refused pids aggregated into `cancel_failures` → `DbError` naming pids
- [x] `Ok(())` preserved for empty pid list (UI cancel button)
- [x] tests: pure-fn aggregation test + source-shape test in `tests/connector_tests.rs`
- [x] `cargo test -p db-pro-infrastructure --lib` green: 167 passed / 0 failed
- [x] `cargo check -p db-pro-infrastructure` clean
