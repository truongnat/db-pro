---
schema_version: 1
id: "MEM-SRC-8c110a67b93d3acb"
title: "Source mirror: docs/plans/archived/saved-query-rename/CHECKLIST.md"
type: pattern
scope: project
status: needs-review
mirror_kind: full-file
mirror_wave: v2-by-source-file
source_path: "docs/plans/archived/saved-query-rename/CHECKLIST.md"
source_sha256: "8c110a67b93d3acb9c57b25f315bcc0cf99a6ed2df4e4ac5ed5052e127766c1a"
mirror_body_sha256: "8c110a67b93d3acb9c57b25f315bcc0cf99a6ed2df4e4ac5ed5052e127766c1a"
truncated: false
created: "2026-10-24"
updated: "2026-10-24"
verified_on: unknown
confidence: low
tags: [migrated, by-source-file]
---
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
