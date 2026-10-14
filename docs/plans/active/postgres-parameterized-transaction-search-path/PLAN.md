# Plan: Fix missing search_path and backend tracking in PostgresConnector::execute_parameterized_transaction

## Core Directive & Evidence
In `PostgresConnector::execute_parameterized_transaction` (`crates/infrastructure/src/postgres/connector.rs`), parameterized transaction batches (used by `TableDataService::apply_mutations` for table row edits/deletes/inserts) begin a `sqlx::Transaction` but fail to invoke `super::search_path::set_local_search_path(&mut tx).await` and fail to track the active backend PID via `self.track_backend(handle.0, pid)`.

This violates the invariant that all database execution within a transaction respects the active query schema configured via `active_query_schema()`.

## Failure Scenario
1. A user connects to PostgreSQL with multiple schemas (e.g. `public` and `tenant_app`).
2. The user switches active query schema to `tenant_app`.
3. The user edits table rows in `TableDataService` (e.g. staging edits or inserts on table `users` in `tenant_app`).
4. `TableDataService::apply_mutations` calls `PostgresConnector::execute_parameterized_transaction`.
5. Because `set_local_search_path` is missing, the transaction connection defaults to PostgreSQL's default `search_path` (`$user, public`).
6. Unqualified or relative references in statements, triggers, sequence lookups, or custom types resolve against `public` instead of `tenant_app`, mutating the wrong table or failing with relation/type missing errors.

## Severity
**P1** - Database mutation correctness & transaction search_path defect in PostgreSQL.

## Scope
1. Update `PostgresConnector::execute_parameterized_transaction` in `crates/infrastructure/src/postgres/connector.rs`.
2. Add unit regression tests in `crates/infrastructure/src/postgres/tests/connector_tests.rs`.
3. Add live integration test in `crates/infrastructure/tests/pg_integration.rs`.
4. Update `docs/plans/STATUS.md`.
