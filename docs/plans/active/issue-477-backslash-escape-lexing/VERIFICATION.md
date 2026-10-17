# #477 verification

## Checks run
- `cargo check -p db-pro-core` — clean (after fixing signature fallout in
  `multi_query_execution.rs`, `query_service/tests.rs`).
- `cargo test -p db-pro-core` — 502 passed, 0 failed.
- `cargo check --workspace` — clean (db-pro-core, infrastructure, ui, app,
  runtime, native-app).

## New regression coverage
- `safety/tests.rs`
  - `split_statements_postgres_backslash_does_not_escape_quote` —
    `SELECT 'x\'; SELECT 2` → 2 statements (default, Postgres, SQLite,
    SqlServer drivers)
  - `split_statements_mysql_backslash_escapes_quote` — same text → 1
    statement on MySQL; `'it\'s ok'` handled
  - `split_statements_escape_string_literal` — `E'x\''`/`e'a\'b'` split
    correctly; `someE'x\'` stays a single identifier+string run
  - `script_safety_sees_statements_after_a_closed_backslash_string` —
    `SELECT 'x\'; DROP TABLE important; --` → Destructive (PG), Read (MySQL)
- `sql_policy_tests.rs`
  - `reject_multi_statement_catches_pg_backslash_before_quote` —
    the exact issue-477 exploit string is rejected on PG
  - `reject_multi_statement_keeps_mysql_backslash_escape` — accepted on MySQL
- `query_service/tests.rs` — all `classify_statement` calls updated to
  `classify_statement(sql, DriverType::Postgres)`; `build_service` now uses
  `mock_connections_full_access()` since `execute` consults the config for the
  driver before the gate.
