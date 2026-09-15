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
