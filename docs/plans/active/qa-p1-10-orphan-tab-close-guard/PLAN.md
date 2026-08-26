# Plan — QA-P1-10 Orphan Tab Dirty Close Guard

## Goal
Ensure closing an orphaned tab (a tab referencing a missing connection) routes through the shared dirty close guard (`requestCloseTab`), preserving unsaved query edits or staged Data Grid mutations instead of bypassing confirmation.

## Lifecycle
State: IMPLEMENTING

## Scope
1. Update `OrphanedTabView` in `frontend/src/commons/components/workspace-content.tsx` to call `requestCloseTab(tabId)`.
2. Add regression tests in `frontend/src/commons/__tests__/request-close-tab.test.ts`.
3. Verify all frontend quality gates pass.
