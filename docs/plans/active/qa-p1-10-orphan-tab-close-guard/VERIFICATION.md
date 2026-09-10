# Verification — QA-P1-10 Orphan Tab Dirty Close Guard

## Code Evidence
- `frontend/src/commons/components/workspace-content.tsx`: `handleCloseTab` updated to invoke `requestCloseTab(tabId)`.
- `frontend/src/commons/__tests__/request-close-tab.test.ts`: Regression unit tests added for dirty tab close and staged mutations close guard.

## Automated Quality Gates
- `npm run typecheck`
- `npm run lint`
- `npm run format:check`
- `npm run check:tokens`
- `npm run test`
- `npm run build`
