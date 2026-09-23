# Plan — QA-P1-09 Connection-list query reconnect side-effect decoupling

State: OBSOLETE (frontend-era / archived React)  
Archived: 2026-09-23  
Note: Targets the archived React frontend (`_archive/frontend/`). Native connection lifecycle is handled in `crates/ui/`.

## Goal
Decouple session restoration side effects from `useConnectionList` fetching logic to ensure fetching connection lists remains pure and side-effect free on generic refetches.

## Architecture & Principles
- React Query `queryFn` should be a pure query operation without hidden reconnect side effects.
- Startup session restoration must be explicitly controlled via a dedicated one-shot coordinator or startup component / hook.
- Both PostgreSQL and SQLite connections must be handled consistently during restoration without unnecessary reconnect attempts.

## Tasks
- [ ] Create active plan documentation.
- [ ] Refactor `restoreSession` and `useConnectionList` in `frontend/src/modules/connection/queries/connection.queries.ts`.
- [ ] Add/update tests in `frontend/src/modules/connection/__tests__/`.
- [ ] Execute quality gates.
- [ ] Publish PR.
