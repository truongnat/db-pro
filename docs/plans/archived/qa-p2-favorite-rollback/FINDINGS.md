# Findings: Favorite Toggle Optimistic Update Rollback (QA-P2-20)

## Problem Description
In `frontend/src/modules/connection/queries/connection.queries.ts`, `useToggleFavorite` performs an optimistic UI update (`toggleFavoriteLocal(id)`) in `onMutate`. However, if the server mutation fails, there is no `onError` callback, leaving the UI state out of sync with backend persistence.

## Severity
P2 — UI state desynchronization on mutation failure.
