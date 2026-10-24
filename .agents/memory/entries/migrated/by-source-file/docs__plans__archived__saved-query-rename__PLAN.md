---
schema_version: 1
id: "MEM-SRC-36fe1580d6130b4c"
title: "Source mirror: docs/plans/archived/saved-query-rename/PLAN.md"
type: pattern
scope: project
status: needs-review
mirror_kind: full-file
mirror_wave: v2-by-source-file
source_path: "docs/plans/archived/saved-query-rename/PLAN.md"
source_sha256: "36fe1580d6130b4c5949ab6382c4fa4c191e833487964b8a02d28e6957b670a9"
mirror_body_sha256: "36fe1580d6130b4c5949ab6382c4fa4c191e833487964b8a02d28e6957b670a9"
truncated: false
created: "2026-10-24"
updated: "2026-10-24"
verified_on: unknown
confidence: low
tags: [migrated, by-source-file]
---
# Plan — Atomic Saved Query Rename (QA-D1)

## Summary
`useRenameSavedQuery` renames saved queries non-atomically by invoking `deleteSaved(id)` followed by `save(...)`. If saving fails, the original saved query is permanently lost. Additionally, deleting and re-saving changes the query's primary key (`id`) and timestamp (`created_at`). This plan updates the Rust core repository, application layer, Tauri IPC layer, and frontend service to execute an atomic `UPDATE saved_queries SET name = ?1 WHERE id = ?2` operation.

## Lifecycle State
`OBSOLETE (frontend-era / archived React)`  
Archived: 2026-09-23  
Note: Frontend hooks and Tauri IPC commands are archived. Native query library/saved-query management is implemented in `crates/ui/`.

## Key Invariants
1. Renaming a saved query MUST be an atomic single database operation.
2. Renaming a saved query MUST NOT alter its primary key `id` or `created_at` timestamp.
3. If renaming fails, the original query MUST remain intact and untouched.

## Steps
1. Add `rename` to `SavedQueryRepository` interface and implement in `SQLiteMetaStore`.
2. Add `rename_saved_query` method to `QueryService`.
3. Add `rename_saved_query` command in `crates/tauri-app/src/commands/query.rs` and register in `crates/tauri-app/src/lib.rs`.
4. Update `IQueryService` and `QueryService` in frontend.
5. Refactor `useRenameSavedQuery` in React Query layer.
6. Add Rust and React regression tests.
