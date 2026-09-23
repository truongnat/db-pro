# Findings — SQLite View Column Introspection

## Problem Statement
In `crates/infrastructure/src/sqlite/introspect.rs`, `run_introspection` calls `introspect_columns` using only `table_names` (base tables fetched from `sqlite_master WHERE type = 'table'`). SQL views (fetched from `sqlite_master WHERE type = 'view'`) are never passed to `introspect_columns`. As a result, `IntrospectResult.columns` contains no column metadata for SQLite views. In contrast, PostgreSQL introspection (`introspect_columns_raw`) includes columns for both tables and views.

## Severity
P1 — PostgreSQL vs SQLite capability and schema introspection mismatch.

## Root Cause
`fetch_table_names` filters for `type = 'table'`. `run_introspection` ran `introspect_columns` before `introspect_views` and supplied only `table_names`. In SQLite, `PRAGMA table_info(view_name)` returns column metadata for views identically to base tables, but view names were omitted from the table_names argument.

## Fix Strategy
1. Move `introspect_views` call before `introspect_columns` in `run_introspection`.
2. Extract view names and combine them with base table names when passing the object list into `introspect_columns`.
