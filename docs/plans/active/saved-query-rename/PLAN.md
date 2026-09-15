# Plan — Atomic Saved Query Rename (QA-D1)

## Summary
`useRenameSavedQuery` renames saved queries non-atomically by invoking `deleteSaved(id)` followed by `save(...)`. If saving fails, the original saved query is permanently lost. Additionally, deleting and re-saving changes the query's primary key (`id`) and timestamp (`created_at`). This plan updates the Rust core repository, application layer, Tauri IPC layer, and frontend service to execute an atomic `UPDATE saved_queries SET name = ?1 WHERE id = ?2` operation.

## Lifecycle State
`IMPLEMENTING`

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
