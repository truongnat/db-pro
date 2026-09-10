# Checklist — RC1 Wave 3: Workspace Recovery

- [x] Create plan documentation (`PLAN.md`, `CHECKLIST.md`, `FINDINGS.md`, `VERIFICATION.md`)
- [x] Verify `reassignTabConnection` clears staged grid edits
- [x] Verify `reassignTabConnection` normalizes `schema` between PostgreSQL (`public`) and SQLite (`main`)
- [x] Verify `reassignTabConnection` resets query context and status while preserving SQL text
- [x] Verify `OrphanedTabView` close button routes through `requestCloseTab`
- [x] Add unit & regression tests in `workspace-reassign.test.ts`
- [x] Run all frontend quality gates (`npm run typecheck`, `npm run lint`, `npm run format:check`, `npm run check:tokens`, `npm run test`, `npm run build`)
