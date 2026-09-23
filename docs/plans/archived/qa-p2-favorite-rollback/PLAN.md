# Plan: Favorite Toggle Optimistic Update Rollback (QA-P2-20)

## Lifecycle State
State: OBSOLETE (frontend-era / archived React)  
Archived: 2026-09-23  
Note: Targets `frontend/src/`. Native connection favorite persistence is handled in `crates/ui/`.

## Goal
Ensure optimistic favorite toggle in `useToggleFavorite` is properly rolled back via `onError` callback if backend persistence fails.

## Tasks
1. Implement `onError` handler in `useToggleFavorite` in `frontend/src/modules/connection/queries/connection.queries.ts`.
2. Add regression unit test in `frontend/src/modules/connection/__tests__/connection-queries.test.tsx`.
3. Verify all frontend and Rust quality gates pass.
