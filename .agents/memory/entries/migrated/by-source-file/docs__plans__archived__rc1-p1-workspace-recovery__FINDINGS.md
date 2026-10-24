---
schema_version: 1
id: "MEM-SRC-7ef7c200265e9607"
title: "Source mirror: docs/plans/archived/rc1-p1-workspace-recovery/FINDINGS.md"
type: pattern
scope: project
status: needs-review
mirror_kind: full-file
mirror_wave: v2-by-source-file
source_path: "docs/plans/archived/rc1-p1-workspace-recovery/FINDINGS.md"
source_sha256: "7ef7c200265e9607e2f074aff7aa7b1fc2b60e2b9b321321089a95ad58ce7742"
mirror_body_sha256: "7ef7c200265e9607e2f074aff7aa7b1fc2b60e2b9b321321089a95ad58ce7742"
truncated: false
created: "2026-10-24"
updated: "2026-10-24"
verified_on: unknown
confidence: low
tags: [migrated, by-source-file]
---
# Findings — RC1 Wave 3: Workspace Recovery

## QA-P1-10 — Orphan dirty query / staged grid Close bypasses shared dirty guard
- **Status:** CONFIRMED & VERIFIED IN CODE
- **Location:** `frontend/src/commons/components/workspace-content.tsx` (`OrphanedTabView`), `frontend/src/commons/services/request-close-tab.ts`
- **Details:** Previously `OrphanedTabView` directly called `useWorkspaceStore.getState().closeTab(tabId)`. Updating it to call `requestCloseTab(tabId)` ensures that `hasUnsavedWork` checks both `tab.dirty` and `stagedChangesStore.getCount(tabId)`.

## QA-P1-11 — Orphan connection reassignment keeps incompatible schema/object context
- **Status:** CONFIRMED & VERIFIED IN CODE
- **Location:** `frontend/src/commons/stores/workspace.store.ts` (`reassignTabConnection`)
- **Details:** When reassigning a tab from PostgreSQL to SQLite (or vice versa), `schema` is normalized (`public` <-> `main`), resource keys and titles update, staged changes belonging to the tab are cleared, and query tab contexts are updated while preserving user SQL.
