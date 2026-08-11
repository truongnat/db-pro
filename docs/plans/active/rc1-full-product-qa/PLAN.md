# RC1 Full Product QA — Static Audit & Fix Program

## Baseline

- Release candidate under audit: `6e0a04ad675eaa85cae08bbe1a066270596a18db`
- Audit branch: `qa/rc1-static-audit`
- Release target: `v0.1.0`
- Lifecycle: `PLANNING → IMPLEMENTING → REVIEW → RUNTIME_VERIFY → COMPLETED`

This program is a pre-release adversarial QA pass across frontend, workspace state, connection lifecycle, query execution, data editing, schema/ER, provider boundaries, performance, accessibility, and desktop UX.

The audit is intentionally separated from frozen RC1. The audit branch contains plans/evidence only. Production fixes must be made on focused fix branches/PRs; do not push broad fixes directly to `main`.

## Audit Method

For every finding:

1. Identify a concrete source path.
2. State a reproducible failure scenario.
3. Assign P0/P1/P2 using `REVIEW.md`.
4. Distinguish source-proven behavior from runtime hypotheses.
5. Add a regression test that proves the invariant, not just a happy path.
6. Preserve PostgreSQL/SQLite separation.
7. Do not mark runtime behavior closed from source/tests alone.

## Executive State

Static audit of RC1 currently reports:

- **P0: 0 confirmed**
- **P1: 14 confirmed/blocking or release-blocking until disproved**
- **P2: 25 tracked UX/performance/consistency issues**
- **Runtime-only gaps remain**, especially real 500+ table ER behavior, provider type coverage, packaged desktop smoke, and visual theme verification.

`v0.1.0` must not be tagged while any confirmed P1 remains open.

## P1 Fix Waves

### QA-W1 — Precision & mutation identity

Highest priority because wrong-row mutation / precision loss is unacceptable.

Scope:

- `QA-P1-01` — BIGINT/i64 precision loss across Rust → Tauri JSON → JavaScript number
- `QA-P1-02` — preview tab replacement can carry staged mutations into another table
- `QA-P1-03` — staged Data Grid mutations do not participate in tab dirty/close lifecycle
- `QA-P1-04` — SQLite result metadata reports every query column as `TEXT`, undermining type-aware editing

Required outcome:

- exact BIGINT/INT8/SQLite i64 identity preserved for display, PK lookup, update, delete, copy/export, sort and round-trip mutation
- a staged mutation can never change resource identity without an explicit preserve/discard decision
- closing/replacing/reassigning a tab cannot orphan hidden mutation state
- provider column metadata used by editable grid must be trustworthy or editing must be capability-gated

### QA-W2 — Connection editor & lifecycle correctness

Scope:

- `QA-P1-05` — SQLite create form incorrectly requires password although SQLite connector ignores it
- `QA-P1-06` — disabling SSH in UI does not clear persisted `sshTunnel`
- `QA-P1-07` — enabling SSH can submit an incomplete tunnel while UI visually shows default port 22
- `QA-P1-08` — stale async create callback can poison next New Connection session
- `QA-P1-09` — connection-list query performs session-reconnect side effects on every refetch/invalidation

Required outcome:

- provider-specific form contracts exactly match backend contracts
- visible form values equal submitted model values
- stale async operations cannot mutate a future dialog session
- startup reconnect runs once as a coordinated lifecycle step, not as a generic query-fetch side effect

### QA-W3 — Workspace recovery & dirty-state safety

Scope:

- `QA-P1-10` — orphan dirty query closes without shared dirty guard
- `QA-P1-11` — orphan db-object/schema workspace reassignment preserves incompatible schema/object context

Required outcome:

- every close path uses the same dirty/staged guard
- connection reassignment clears or safely re-resolves provider-specific context
- no stale result, schema, table, editor or staged state leaks across reassignment

### QA-W4 — ER Diagram large-schema correctness

Scope:

- `QA-P1-12` — large-schema “search-first” mode still builds/layouts all tables on initial open
- `QA-P1-13` — initial ER zoom tier is full-detail, allowing a large first-paint DOM spike before fitView updates the tier

Required outcome:

- >200-table schema initially renders a bounded/search-first state
- initial node detail is safe before viewport initialization
- full graph rendering is explicit
- automated large-schema test must assert node/layout/DOM strategy, not only metadata-map build time

### QA-W5 — PostgreSQL type coverage

Scope:

- `QA-P1-14` — PostgreSQL row mapper has narrow explicit type coverage and falls back to `String` for all other types; high-precision NUMERIC/DECIMAL has no dedicated lossless `CellValue` representation

Required outcome:

- prove supported behavior for NUMERIC/DECIMAL, BIGINT, DATE, TIME, INTERVAL, INET, enums/unknown text-like types, arrays where applicable
- unsupported types must render safely/read-only with a user-visible reason rather than failing a whole row/query
- high-precision values never transit through IEEE-754 number representation

## P2 Fix Waves

### QA-W6 — Workspace/tab interaction

- canonical pinned-tab ordering across store, render order, drag/drop and Ctrl+Tab
- platform-correct tab/context-menu shortcut labels
- Quick Open recent schema-workspace consistency
- connection picker runtime-state affordance

### QA-W7 — Data Grid UX & scale

- Columns picker double-toggle event ownership
- read-only connection should present grid as read-only before Apply
- custom context menu viewport/focus/Escape behavior
- keyboard-accessible resize handles
- large result sorting main-thread budget
- grid zoom/virtualizer runtime validation
- all-column lookup hot paths

### QA-W8 — Shell/desktop UX

- macOS-only traffic-light inset must not appear on Windows/Linux
- Agent panel must visibly state Preview/Coming Soon and use platform shortcuts
- remove/hide dead Schema Overview placeholder for 0.1 if it has no real workflow
- hardcoded English strings migrate to i18n
- resize handlers need unmount/interruption cleanup

### QA-W9 — Connection UX polish

- test-connection result invalidates when form config changes
- test errors preserve backend user-facing detail
- SQLite Browse error path
- driverChanged state should reflect current-vs-initial driver rather than “ever changed”
- duplicate-connection credential semantics must be explicit
- optimistic favorite toggle rollback
- SQLite welcome/recent display must not show meaningless `:0`

### QA-W10 — Query/ER secondary UX

- Export availability must depend on actual result state, not merely SQL text
- avoid native `window.confirm` for dirty-history/import replacement
- result-panel and status copy localization
- query connection picker should show/handle disconnected runtime state
- ER search result disambiguation
- memoize large-schema derived table list
- dynamic ER handles at LOD transitions require runtime proof
- Fit View should use React Flow API instead of synthetic keyboard dispatch

## Required Fix-Branch Strategy

Do **not** put all findings in one monster PR.

Recommended order:

```text
qa/rc1-static-audit
  ├─ fix/rc1-p1-precision-staged-state      (QA-W1)
  ├─ fix/rc1-p1-connection-lifecycle        (QA-W2)
  ├─ fix/rc1-p1-workspace-recovery          (QA-W3)
  ├─ fix/rc1-p1-er-large-schema             (QA-W4)
  ├─ fix/rc1-p1-provider-types              (QA-W5)
  ├─ fix/rc1-p2-workspace-grid              (QA-W6 + QA-W7, split if large)
  └─ fix/rc1-p2-desktop-polish              (QA-W8 + QA-W9 + QA-W10, split if large)
```

Rules:

- One active P1 PR at a time if files overlap.
- Rebase/retarget later waves after earlier P1 merges.
- Every PR must request Kilo/Cubic and close all P0/P1 findings against current head.
- Do not mark `RUNTIME_VERIFY` complete without manual/provider evidence.
- No Agent/MCP/new product feature work in this program.

## Automated Gates Per P1 PR

Frontend when touched:

```bash
cd frontend
npm run typecheck
npm run lint
npm run format:check
npm run check:tokens
npm run test
npm run build
```

Rust when touched:

```bash
cargo fmt --all -- --check
cargo check --workspace
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

Provider-sensitive fixes additionally need focused integration tests.

## Release Exit Criteria

The program may advance to final release sign-off only when:

- P0 = 0
- P1 = 0 on latest integrated main SHA
- all P1 regression tests green
- exact-main frontend/Rust gates green
- PostgreSQL and SQLite provider checks are separately recorded
- packaged desktop smoke passes
- 500+ table ER office smoke passes or remaining limitations are explicitly release-approved
- light/dark token visual smoke passes
- all unresolved P2 are explicitly accepted/deferred with owner/reason
- final release docs are updated once, then exact-SHA CI reruns before tag
