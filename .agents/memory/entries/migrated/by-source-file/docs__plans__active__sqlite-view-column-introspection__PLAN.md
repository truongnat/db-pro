---
schema_version: 1
id: "MEM-SRC-db0f7e13aaa182c0"
title: "Source mirror: docs/plans/active/sqlite-view-column-introspection/PLAN.md"
type: pattern
scope: project
status: needs-review
mirror_kind: full-file
mirror_wave: v2-by-source-file
source_path: "docs/plans/active/sqlite-view-column-introspection/PLAN.md"
source_sha256: "db0f7e13aaa182c0d9c7d98f4e59ced28710bf00e2de6d25d6969cd63965c9a7"
mirror_body_sha256: "db0f7e13aaa182c0d9c7d98f4e59ced28710bf00e2de6d25d6969cd63965c9a7"
truncated: false
created: "2026-10-24"
updated: "2026-10-24"
verified_on: unknown
confidence: low
tags: [migrated, by-source-file]
---
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
