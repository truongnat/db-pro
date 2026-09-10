# Findings — QA-P1-10 Orphan Tab Dirty Close Guard

## Problem Statement
In `frontend/src/commons/components/workspace-content.tsx`, `OrphanedTabView` directly called `useWorkspaceStore.getState().closeTab(tabId)`. This bypassed `requestCloseTab(tabId)` and its shared dirty check guard (`hasUnsavedWork(tabId)`), which inspects both `tab.dirty` and staged Data Grid revisions (`useStagedChangesStore`).

## Failure Scenario
1. User opens a query tab or table tab, edits SQL or stages Data Grid edits.
2. Connection becomes invalid/removed or context changes, causing the tab to display `OrphanedTabView`.
3. User clicks "Close".
4. Tab closes immediately without warning, discarding unsaved SQL or staged mutations.

## Severity
P1: Unsaved user work loss and state guard bypass.
