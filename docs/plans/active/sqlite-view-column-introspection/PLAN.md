# Plan — SQLite View Column Introspection Fix

## Objective
Fix SQLite schema introspection in `crates/infrastructure/src/sqlite/introspect.rs` so that column metadata is fetched for SQL views as well as base tables, aligning SQLite schema introspection behavior with PostgreSQL.

## Target Branch
`fix/sqlite-view-column-introspection`

## Severity
P1 (PostgreSQL vs SQLite capability and schema introspection mismatch)

## Lifecycle State
`PLANNING → IMPLEMENTING → REVIEW → RUNTIME_VERIFY`

## Scope
1. Update `run_introspection` in `crates/infrastructure/src/sqlite/introspect.rs`:
   - Introspect view names before column introspection.
   - Combine base table names and view names when invoking `introspect_columns`.
2. Add unit and integration test coverage verifying that `IntrospectResult.columns` includes columns for SQLite views.
3. Run applicable Rust quality gates (`cargo test -p db-pro-infrastructure`).
