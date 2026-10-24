---
schema_version: 1
id: "MEM-SRC-6a36a817c90869d2"
title: "Source mirror: docs/plans/archived/saved-query-rename/FINDINGS.md"
type: pattern
scope: project
status: needs-review
mirror_kind: full-file
mirror_wave: v2-by-source-file
source_path: "docs/plans/archived/saved-query-rename/FINDINGS.md"
source_sha256: "6a36a817c90869d259379e31b627df52bb2c3847a0567dd20e31d20e1f48c7db"
mirror_body_sha256: "6a36a817c90869d259379e31b627df52bb2c3847a0567dd20e31d20e1f48c7db"
truncated: false
created: "2026-10-24"
updated: "2026-10-24"
verified_on: unknown
confidence: low
tags: [migrated, by-source-file]
---
# Findings — Atomic Saved Query Rename (QA-D1)

## Defect Summary
- **Defect ID**: QA-D1
- **Severity**: P1 (Data Loss & Non-atomic Mutation)
- **Location**: `frontend/src/modules/query/queries/query.queries.ts` lines 283–306
- **Evidence**:
  ```ts
  await getQueryService().deleteSaved(id);
  return getQueryService().save(connectionId, newName, existing.sql, existing.folder);
  ```
- **Failure Scenario**:
  User triggers query rename. `deleteSaved` succeeds in deleting the query record from SQLite meta store. The subsequent `save` command fails due to payload validation, database lock, network interruption, or IPC serialization error. The original saved query is deleted permanently.

## Solution Architecture
1. Backend: Expose `rename` operation through `SavedQueryRepository`, `QueryService`, and Tauri `rename_saved_query` command. Execute atomic `UPDATE saved_queries SET name = ?1 WHERE id = ?2`.
2. Frontend: Call `renameSaved` directly in `useRenameSavedQuery`. Do not delete the record.
