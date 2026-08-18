# Plan: Favorite Toggle Optimistic Update Rollback (QA-P2-20)

## Lifecycle State
IMPLEMENTING

## Goal
Ensure optimistic favorite toggle in `useToggleFavorite` is properly rolled back via `onError` callback if backend persistence fails.

## Tasks
1. Implement `onError` handler in `useToggleFavorite` in `frontend/src/modules/connection/queries/connection.queries.ts`.
2. Add regression unit test in `frontend/src/modules/connection/__tests__/connection-queries.test.tsx`.
3. Verify all frontend and Rust quality gates pass.
