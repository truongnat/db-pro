# Plan — RC1 Wave 3: Workspace Recovery & Dirty-State Safety (QA-P1-10 & QA-P1-11)

## Context & Objectives
Address remaining Wave 3 (W3) P1 defects documented in `docs/plans/active/rc1-full-product-qa/FINDINGS.md`:
- **QA-P1-10:** Orphan dirty query / staged grid Close bypasses shared dirty guard.
- **QA-P1-11:** Orphan connection reassignment preserves incompatible schema/object context across PostgreSQL and SQLite.

## Goals
1. Guarantee that closing an orphaned tab with unsaved work (dirty query text or staged data grid edits) always checks `hasUnsavedWork` via `requestCloseTab` and triggers the close guard confirmation dialog.
2. Ensure that when an orphaned tab is reassigned to a new connection, provider-specific schema context is normalized (`public` for PostgreSQL vs `main` for SQLite), `resourceKey` and tab titles are regenerated, staged edits for the tab are purged, and query execution context is cleanly reset without destroying user SQL text.

## Action Plan
- Write `docs/plans/active/rc1-p1-workspace-recovery/` tracking documents (`PLAN.md`, `CHECKLIST.md`, `FINDINGS.md`, `VERIFICATION.md`).
- Inspect and verify `frontend/src/commons/stores/workspace.store.ts` (`reassignTabConnection`).
- Inspect and verify `frontend/src/commons/components/workspace-content.tsx` (`OrphanedTabView`).
- Add tests in `frontend/src/commons/stores/__tests__/workspace-reassign.test.ts`.
- Run frontend quality gates.
