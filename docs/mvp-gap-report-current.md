# DB Pro MVP Gap Report (Current State)

- Updated: 2026-03-05
- Baseline source: `docs/mvp-dbeaver-gap-report.md`
- Scope: gap-to-MVP after delivered P0/P1 slices and latest query pushdown fixes.

## 1) Executive Summary

DB Pro has reached a strong functional baseline for a desktop SQL client MVP:
- Multi-engine connections (SQLite/PostgreSQL/MySQL), persisted profiles, keychain password storage.
- Query execution with timeout/cancel, schema navigator, SQL autocomplete, history, formatter, templates.
- Result workflow now has server-side quick filter/sort pushdown and CSV/copy actions.

Current status is **"near-MVP for solo/dev usage"**, but not yet **"ship-ready MVP comparable to DBeaver basic workflow"** for real production datasets because of stability, architecture, and UX depth gaps.

## 2) What Is Closed vs Original Gap List

Closed/highly improved from original report:
- `OBS-001/OBS-005`: query history + rerun.
- `NAV-007`: navigator search/filter.
- `NAV-009`: navigator context actions (generate SQL/open data/copy name).
- `SQL-008`: SQL formatter.
- `SQL-009`: SQL templates.
- `GRID-007/GRID-015`: copy page + export CSV.
- `GRID-012`: server-side sort.
- `GRID-013`: server-side quick filter.
- `UX-001/UX-004/UX-005` (partial): global keyboard shortcuts + loading/error consistency.

## 3) Open Gaps to MVP (Prioritized)

## 3.1 Blockers (must close before MVP sign-off)

| ID | Gap | Current Risk | MVP DoD |
|---|---|---|---|
| B-001 | Real PostgreSQL workload stability not fully validated end-to-end | User-reported hangs under real DB conditions; now has diagnostics, but not proven across network/latency/large table cases | Validate against real PG instances (small/medium/large tables, lock contention, slow network). No infinite "Running..." states; timeout/cancel always deterministic |
| B-002 | Clean architecture gap (over-centralized app orchestration) | `src/App.tsx` is 1100+ LOC; high coupling raises regression risk and slows iteration | Split into feature controllers/hooks (`useConnectionController`, `useQueryController`, `useNavigatorController`, `useResultGridController`) with strict boundaries |
| B-003 | Automated regression coverage is missing for critical flows | Changes to query/navigator/grid can regress without detection | Add integration tests for: run/cancel/timeout, navigator refresh-after-DDL, server filter/sort paging, connection restore/sample bootstrap |

## 3.2 High Priority (core workflow quality)

| ID | Gap | Current Risk | MVP DoD |
|---|---|---|---|
| H-001 | SQL completion keyboard UX is still below pro-tool expectation | Suggestion list behavior can feel inconsistent vs DBeaver-like tools | Reliable Up/Down/Enter/Tab/Esc behavior, stable focus, ranking tuned (keyword vs object vs column) |
| H-002 | SQL syntax styling/readability depth | User expectation for richer SQL visual parsing is higher | Distinct token styling per dialect, better contrast/theming, consistent light mode readability |
| H-003 | Result grid interaction depth | Missing column resize/reorder/show-hide, richer selection copy modes | Column resize + multi-cell selection + copy as CSV/TSV/JSON + stable scroll UX |
| H-004 | Connection panel UX hardening | Add/edit/reset/cache UX still not at “professional tooling” polish level | High-fidelity add/edit modal flow, password visibility/validation states, robust cache reset semantics |
| H-005 | Navigator scalability for big schemas | Full metadata load can be heavy on large DBs | Lazy schema/object loading, progressive fetch, no endless loading states |

## 3.3 Medium Priority (important but not MVP blockers)

| ID | Gap | Current Risk | MVP DoD |
|---|---|---|---|
| M-001 | Query manager view (history analytics/log panel) | Hard to trace execution incidents across sessions | Filterable query log panel by connection/time/status |
| M-002 | Transaction UX (auto-commit/manual commit/rollback) | Production DML safety workflow still basic | Auto-commit toggle + explicit commit/rollback controls |
| M-003 | Parameter binding UX (`:param`) | Script portability/workflow incomplete | Parameter prompt + typed binding before execute |
| M-004 | Multi-tab SQL workspace | Single-script workflow limits power users | Multiple editor tabs with persisted buffers and per-tab result context |
| M-005 | Import/export wizard depth | CSV export exists, but transfer workflow is limited | Guided export/import wizard with profile presets |

## 4) Architecture Gap Notes

- `App.tsx` currently contains orchestration for connection, query, navigator, result grid, history, cache, and modal flows in one place.
- This increases blast radius for each feature change and conflicts with clean architecture goals.
- Immediate refactor target: isolate use-case-level hooks and move side-effect logic out of root component.

## 5) Suggested MVP Readiness Score

- Functional capability: **78/100**
- Stability under real DB conditions: **62/100**
- UX parity for daily workflow: **70/100**
- Architecture/maintainability: **58/100**
- Overall MVP readiness: **67/100**

## 6) Recommended Execution Slices (Next)

1. **Stability slice (B-001)**
- Run controlled PG test matrix and close all hanging/cancel edge cases.
- Add telemetry-driven diagnostics dashboard/logging for query stages.

2. **Architecture slice (B-002 + B-003)**
- Break `App.tsx` into feature controllers.
- Add integration tests for run/cancel/navigator/grid pushdown.

3. **UX core slice (H-001 + H-003)**
- Completion keyboard overhaul.
- Result grid interaction depth (selection/resize/copy modes).

## 7) Immediate Acceptance Checklist

- [ ] No unresolved "Running..." state in PostgreSQL real-world tests.
- [ ] Cancel query works consistently under slow/locked queries.
- [ ] App-level orchestration split from root component into modular controllers.
- [ ] Integration test suite exists for critical query+navigator+grid flows.
- [ ] SQL completion keyboard behavior matches expected desktop conventions.

