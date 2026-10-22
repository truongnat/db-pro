# Postgres autocommit search_path — VERIFICATION

- Verified against the uncommitted tree on HEAD `ebba5ca74590abf50c0c3f72daf3e7248ff90755`; the
  exact code-snapshot SHA is recorded in `AGENT_EVIDENCE.md` §1 (plan docs are not part of it).
- Files:
  - `crates/infrastructure/src/postgres/search_path.rs`
  - `crates/infrastructure/src/postgres/connector.rs`
  - `crates/infrastructure/src/transaction_session.rs`
  - `crates/infrastructure/src/tests/transaction_session_tests.rs`
  - `crates/infrastructure/tests/pg_integration.rs`

## Live PostgreSQL regression evidence

Fixture: local PostgreSQL 18, `DATABASE_URL=postgres://postgres:postgres@127.0.0.1:5432/postgres`.

| Test | Before fix | After fix |
|------|-----------|-----------|
| `pg_autocommit_run_resolves_unqualified_tables_in_active_query_schema` | FAIL — `relation "scoped_values" does not exist` | PASS |
| `pg_transaction_run_resolves_unqualified_tables_in_active_query_schema` | n/a (new) | PASS |
| `pg_autocommit_timeout_discards_the_schema_scoped_connection` | n/a (new) | PASS |

Commands (all exit 0):
- `DATABASE_URL=... cargo test -p db-pro-infrastructure --test pg_integration run_resolves -- --ignored`
  → 2 passed / 0 failed, 38 filtered out.
- `DATABASE_URL=... cargo test -p db-pro-infrastructure --test pg_integration pg_autocommit_timeout_discards_the_schema_scoped_connection -- --ignored`
  → 1 passed / 0 failed (4.09 s).
- `cargo test -p db-pro-infrastructure --lib transaction_session` → 7 passed / 0 failed, including
  `abandoned_session_scoped_run_discards_the_connection` and
  `completed_session_scoped_run_keeps_the_connection`.

## Workspace gates

| Gate | Result |
|------|--------|
| `cargo +nightly fmt --all -- --check` | PASS (exit 0) |
| `cargo check --workspace` | PASS (exit 0) |
| `cargo clippy --workspace --all-targets -- -D warnings` | PASS (exit 0) |
| `cargo test --workspace` | PASS — 2243 passed / 0 failed / 65 ignored across 27 test binaries (exit 0) |
| `python3 tools/cc-scan.py crates/infrastructure/src/postgres/connector.rs crates/infrastructure/src/postgres/search_path.rs crates/infrastructure/src/transaction_session.rs --fail-on error` | FAIL — 1 inherited error: `execute_batch` is 76 lines (unchanged by this work; the same finding is present at HEAD). No new findings from this change. |

## Limitations

- Native-app UAT (tenant picker → RunQuery → runtime → connector against a real tenant database)
  is not yet recorded; the connector boundary and the app dispatch schema are verified separately.
- SQLite/MySQL/SQL Server were not re-run for this change; their code paths are untouched.
- `execute_batch` carries a pre-existing clean-code `HUGE_FUNCTION` error at HEAD; it was not
  touched to keep this fix scoped.

