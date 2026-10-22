# Postgres autocommit search_path — CHECKLIST

- [x] Reproduce `SET LOCAL` expiry on live PostgreSQL (autocommit)
- [x] Reproduce through the connector: new regression test failed before the fix with
      `relation "scoped_values" does not exist`
- [x] Scope push/pop by connection transaction state
- [x] Preserve #456 with discard-on-abandon + `close_on_drop`
- [x] Unit tests for abandoned vs restored leases
- [x] Integration tests: autocommit run, held transaction, timeout leak
- [x] `cargo +nightly fmt --all -- --check`
- [x] `cargo check --workspace`
- [x] `cargo clippy --workspace --all-targets -- -D warnings`
- [x] `cargo test --workspace` (2243 passed / 0 failed / 65 ignored)
- [x] Clean-code scan scoped to the changed files — no new findings (one inherited `execute_batch`
      `HUGE_FUNCTION` error also present at HEAD)
- [ ] UAT: tenant picker + unqualified query in the native app against a real tenant database
