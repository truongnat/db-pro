# Checklist — QA-P1-10 Orphan Tab Dirty Close Guard

- [x] Create feature plan documentation under `docs/plans/active/qa-p1-10-orphan-tab-close-guard/`
- [x] Verify `OrphanedTabView` in `frontend/src/commons/components/workspace-content.tsx` invokes `requestCloseTab(tabId)`
- [x] Add unit tests in `frontend/src/commons/__tests__/request-close-tab.test.ts`
- [x] Run all frontend quality gates (`typecheck`, `lint`, `format:check`, `check:tokens`, `test`, `build`)
- [ ] Create branch `fix/qa-p1-10-orphan-tab-close-guard` and publish Pull Request
