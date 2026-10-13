# Plan: PostgreSQL Parameterized Transaction Search Path Isolation

## Lifecycle State
RUNTIME_VERIFY

## Goal
Ensure `PostgresConnector::execute_parameterized_transaction` sets local session search path (`SET LOCAL search_path`) immediately after `pool.begin()`, matching `execute_batch` and `execute_transaction`, so that staged grid edits target the user's active schema.

## Impact
Fixes P1 mutation correctness defect where staged mutations in the Table Data Editor run against PostgreSQL default search path (`public`) rather than `active_query_schema()`.

## Tasks
- [x] Create active plan files under `docs/plans/active/postgres-parameterized-transaction-search-path/`
- [x] Implement `set_local_search_path` call in `PostgresConnector::execute_parameterized_transaction`
- [x] Add unit test verifying search path handling on `execute_parameterized_transaction`
- [x] Pass quality gates (`cargo check`, `cargo test`)
- [x] Complete pre-commit checklist
