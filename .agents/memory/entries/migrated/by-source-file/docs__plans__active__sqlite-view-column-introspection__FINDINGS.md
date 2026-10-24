---
schema_version: 1
id: "MEM-SRC-bf485dd9b9c0e5c5"
title: "Source mirror: docs/plans/active/sqlite-view-column-introspection/FINDINGS.md"
type: pattern
scope: project
status: needs-review
mirror_kind: full-file
mirror_wave: v2-by-source-file
source_path: "docs/plans/active/sqlite-view-column-introspection/FINDINGS.md"
source_sha256: "bf485dd9b9c0e5c5aead2cafffb812c2590ad2102bb121e7f6f079ff968dc2be"
mirror_body_sha256: "bf485dd9b9c0e5c5aead2cafffb812c2590ad2102bb121e7f6f079ff968dc2be"
truncated: false
created: "2026-10-24"
updated: "2026-10-24"
verified_on: unknown
confidence: low
tags: [migrated, by-source-file]
---
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
