# DB Pro MVP Sprint Backlog

- Updated: 2026-03-05
- Source inputs:
  - `docs/mvp-gap-report-current.md`
  - `docs/mvp-dbeaver-gap-report.md`
- Goal: convert current MVP gaps into execution-ready sprint tickets with clear DoD and validation gates.

## 1) Sprint Model

- Sprint duration: 1 week/sprint.
- Release target: MVP candidate after Sprint 3.
- Priority policy: Blockers first, then workflow-quality UX.
- Global quality gate per ticket:
  - `npm run build`
  - `cargo check --manifest-path src-tauri/Cargo.toml`

## 2) Sprint 1 (Stability + PostgreSQL Reliability)

## 2.1 Objective

Eliminate unresolved "query stuck" scenarios on real PostgreSQL workloads and make failure modes deterministic.

## 2.2 Tickets

| Ticket | Title | Scope | DoD | Validation |
|---|---|---|---|---|
| S1-01 | PostgreSQL real-world test matrix | Define and run matrix: local PG, remote PG, high-latency PG, lock contention, large table paging | Matrix documented and executed; each scenario has pass/fail evidence | Add report section with scenario logs |
| S1-02 | Query lifecycle hardening | Ensure run/cancel/timeout state machine cannot remain indefinitely in `Running...` | No unresolved running state after timeout/cancel in all matrix scenarios | Manual test script + reproducible checklist |
| S1-03 | Stage diagnostics normalization | Standardize query diagnostics (`connect/session/fetch/mode`) across engines | Status and logs contain deterministic diagnostics format for each query result | Snapshot examples in docs |
| S1-04 | Wrapped pagination fallback policy | Prevent silent fallback when server-side filter/sort cannot be guaranteed | Filter/sort requests either run via wrapped SQL or fail with explicit actionable error | PG/MySQL/SQLite behavior table updated |
| S1-05 | Incident-friendly error surfaces | Improve user-facing error messages for network/auth/timeout/cancel | Errors are classified and actionable in status bar and error banner | Error taxonomy checklist |

## 2.3 Exit Criteria

- 0 unresolved blocker from B-001.
- At least 20 consecutive run/cancel cycles on PostgreSQL without UI dead state.
- No silent degradation for filter/sort pushdown behavior.

## 3) Sprint 2 (Architecture Refactor + Regression Safety)

## 3.1 Objective

Reduce coupling and make future feature delivery safe by introducing clean controller boundaries and integration coverage.

## 3.2 Tickets

| Ticket | Title | Scope | DoD | Validation |
|---|---|---|---|---|
| S2-01 | Connection controller extraction | Move connection CRUD/reset/cache orchestration from `App.tsx` to `useConnectionController` | `App.tsx` no longer owns connection business logic directly | Typecheck/build passes; behavior parity checklist |
| S2-02 | Query controller extraction | Move run/cancel/history/page/filter/sort orchestration into `useQueryController` | Query flow isolated with stable public API | Existing run/cancel/history UX unchanged |
| S2-03 | Navigator controller extraction | Move navigator loading/retry/refresh/action dispatch into `useNavigatorController` | Navigator logic isolated from root view | Refresh-after-DDL parity confirmed |
| S2-04 | Result grid controller extraction | Move grid modifiers/debounce/pagination integration logic into dedicated controller | Grid state changes are deterministic and testable | Filter/sort/page integration checklist |
| S2-05 | Integration tests for critical flows | Add integration tests for query/navigator/grid/connection restore/sample bootstrap | Critical regressions can be caught automatically | CI/local test command documented |
| S2-06 | Architecture boundaries doc | Document data flow, ownership, and anti-patterns | New architecture map committed under `docs/` | Reviewer sign-off checklist |

## 3.3 Exit Criteria

- `App.tsx` orchestration size reduced significantly (target: <= 450 LOC).
- Integration tests cover at least: run/cancel/timeout, navigator DDL refresh, filter/sort pushdown paging, sample bootstrap.
- No behavior regressions vs Sprint 1 baseline.

## 4) Sprint 3 (Core UX Parity for Daily Workflow)

## 4.1 Objective

Reach practical DBeaver-like comfort for daily query workflow through keyboard-first SQL completion and stronger result grid interactions.

## 4.2 Tickets

| Ticket | Title | Scope | DoD | Validation |
|---|---|---|---|---|
| S3-01 | Completion keyboard navigation overhaul | Up/Down/Enter/Tab/Esc behavior in all suggestion states | Completion can be fully operated without mouse | Keyboard walkthrough script passes |
| S3-02 | Completion ranking quality | Tune scoring for keyword/object/column + recency | Top suggestions are relevant in real schemas | Ranking acceptance cases documented |
| S3-03 | SQL visual readability upgrade | Improve syntax theme contrast and token differentiation | SQL readability improved in default light mode | Before/after screenshot checklist |
| S3-04 | Result grid selection upgrade | Multi-cell selection with robust copy semantics | Copy selection supports at least TSV and CSV modes | Clipboard output test matrix |
| S3-05 | Result grid column controls | Add column resize and basic show/hide | Column width/state interactions are stable and performant | Manual UX checklist on 1k+ rows |
| S3-06 | Connection panel pro polish | Add advanced validation, password input UX, reset/cache clarity | Add/Edit flow feels deterministic and polished | UX acceptance checklist |

## 4.3 Exit Criteria

- Keyboard-first query editing + completion flow is production-usable.
- Grid interactions no longer feel limited to "viewer-only" mode.
- MVP candidate quality aligns with `docs/mvp-gap-report-current.md` high-priority targets.

## 5) Cross-Sprint Risks and Mitigations

| Risk | Impact | Mitigation |
|---|---|---|
| Refactor causes hidden regressions | High | Build integration tests before deep restructuring; preserve behavior contracts |
| PostgreSQL edge cases are environment-specific | High | Keep reproducible matrix and capture structured diagnostics per scenario |
| UX changes drift from existing patterns | Medium | Keep shadcn primitives and existing visual language; iterate with acceptance checklist |
| Scope creep from non-MVP asks | Medium | Freeze scope to Sprint tickets and defer non-critical items |

## 6) MVP Candidate Sign-off Checklist

- [ ] Sprint 1 exit criteria complete.
- [ ] Sprint 2 exit criteria complete.
- [ ] Sprint 3 exit criteria complete.
- [ ] No blocker-level issues open.
- [ ] Build + checks pass in clean environment.
- [ ] Stakeholder UAT passes top workflows: connect, browse schema, run query, filter/sort, export/copy.

