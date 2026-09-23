# Checklist — Atomic Saved Query Rename (QA-D1)

- [ ] Rust core: `SavedQueryRepository::rename` trait method defined
- [ ] Rust infra: `SQLiteMetaStore::rename` implemented with `UPDATE saved_queries SET name = ?1 WHERE id = ?2`
- [ ] Rust app: `QueryService::rename_saved_query` exposed
- [ ] Rust tauri-app: `rename_saved_query` command created & registered
- [ ] Frontend DI contract: `IQueryService::renameSaved` added
- [ ] Frontend service: `QueryService::renameSaved` implemented via IPC
- [ ] Frontend hook: `useRenameSavedQuery` updated to call `renameSaved` directly without `deleteSaved`
- [ ] Rust tests added & passing
- [ ] Frontend tests added & passing
- [ ] Pre-commit quality gates executed & passing
