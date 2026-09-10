# Verification — RC1 Wave 3: Workspace Recovery

## Automated Test Coverage
- `frontend/src/commons/stores/__tests__/workspace-reassign.test.ts`
  - Reassigning `query` tab resets execution state and updates default `context` while keeping SQL.
  - Reassigning `db-object` and `schema-workspace` tabs normalizes schema between PostgreSQL and SQLite.
  - Reassigning tabs clears staged changes from `stagedChangesStore`.
  - Closing an orphaned tab with unsaved work via `requestCloseTab` triggers `closeGuardStore`.

## Quality Gate Verification
- Frontend: `npm run typecheck`, `npm run lint`, `npm run format:check`, `npm run check:tokens`, `npm run test`, `npm run build`
