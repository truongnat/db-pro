# Findings — RC1 Wave 3: Workspace Recovery

## QA-P1-10 — Orphan dirty query / staged grid Close bypasses shared dirty guard
- **Status:** CONFIRMED & VERIFIED IN CODE
- **Location:** `frontend/src/commons/components/workspace-content.tsx` (`OrphanedTabView`), `frontend/src/commons/services/request-close-tab.ts`
- **Details:** Previously `OrphanedTabView` directly called `useWorkspaceStore.getState().closeTab(tabId)`. Updating it to call `requestCloseTab(tabId)` ensures that `hasUnsavedWork` checks both `tab.dirty` and `stagedChangesStore.getCount(tabId)`.

## QA-P1-11 — Orphan connection reassignment keeps incompatible schema/object context
- **Status:** CONFIRMED & VERIFIED IN CODE
- **Location:** `frontend/src/commons/stores/workspace.store.ts` (`reassignTabConnection`)
- **Details:** When reassigning a tab from PostgreSQL to SQLite (or vice versa), `schema` is normalized (`public` <-> `main`), resource keys and titles update, staged changes belonging to the tab are cleared, and query tab contexts are updated while preserving user SQL.
