---
schema_version: 1
id: "MEM-PLAN-sqlite-view-column-introspection-a120ffefeabb"
title: "Plan archive: Sqlite View Column Introspection"
type: decision
scope: project
status: needs-review
created: "2026-10-24"
updated: "2026-10-24"
verified_on: "unknown"
confidence: medium
tags: [migrated, docs-plans, decision, sqlite-view-column-introspection]
mirror_wave: v1-plan-evidence-excerpt
mirror_excerpt_sha256: "a120ffefeabb69c0c9b390cd4917dfc496bd01d4e596ecdf2a8802791ba8e8ae"
applies_to:
  repo: "db-pro"
  stack: "see sources"
  versions: "see plan state at migration"
sources:
  - "docs/plans/active/sqlite-view-column-introspection/PLAN.md"
  - "docs/plans/active/sqlite-view-column-introspection/FINDINGS.md"
  - "docs/plans/active/sqlite-view-column-introspection/VERIFICATION.md"
related: []
supersedes: null
---

# Tóm tắt

Migrate từ `docs/plans/active/sqlite-view-column-introspection/`. Loại suy luận: `decision`.

Entry tự động từ `docs/plans/`. Đối chiếu code hiện tại trước khi đặt `status: active`.

## Nội dung gốc (rút gọn)

## Source: `PLAN.md`

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


## Source: `FINDINGS.md`

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


## Source: `VERIFICATION.md`

# Verification — SQLite View Column Introspection

## Test Execution
- `cargo check -p db-pro-infrastructure`
- `cargo clippy -p db-pro-infrastructure`
- `cargo test -p db-pro-infrastructure`

## Verification Evidence
- Unit test `introspection_populates_columns_for_sqlite_views` in `crates/infrastructure/src/sqlite/introspect.rs` passes (renamed from `introspect_view_columns_populated` — **[CORRECTED 2026-10-13]**, `_audit-2026-10-13` slice 01).
- Integration test `introspect_views` in `crates/infrastructure/tests/integration.rs` asserts that view columns (e.g., for `active_users`) are populated in `IntrospectResult.columns`.

