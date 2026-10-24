---
schema_version: 1
id: "MEM-PLAN-rc1-full-product-qa-b54a8fad9478"
title: "Plan archive: Rc1 Full Product Qa"
type: decision
scope: project
status: needs-review
created: "2026-10-24"
updated: "2026-10-24"
verified_on: "unknown"
confidence: medium
tags: [migrated, docs-plans, decision, rc1-full-product-qa]
mirror_wave: v1-plan-evidence-excerpt
mirror_excerpt_sha256: "b54a8fad94789125143c0ef067145f7f1a8a6ac865210c291532530da9338a51"
applies_to:
  repo: "db-pro"
  stack: "see sources"
  versions: "see plan state at migration"
sources:
  - "docs/plans/active/rc1-full-product-qa/PLAN.md"
  - "docs/plans/active/rc1-full-product-qa/FINDINGS.md"
  - "docs/plans/active/rc1-full-product-qa/VERIFICATION.md"
related: []
supersedes: null
---

# Tóm tắt

Migrate từ `docs/plans/active/rc1-full-product-qa/`. Loại suy luận: `decision`.

Entry tự động từ `docs/plans/`. Đối chiếu code hiện tại trước khi đặt `status: active`.

## Nội dung gốc (rút gọn)

## Source: `PLAN.md`

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


## Source: `FINDINGS.md`

# RC1 Full Product QA — Findings

Baseline: `6e0a04ad675eaa85cae08bbe1a066270596a18db`

Severity follows `REVIEW.md`. `SOURCE_CONFIRMED` means the problematic state transition or contract mismatch is visible from source. It does **not** mean runtime behavior has been manually reproduced. `RUNTIME_CONFIRM` means source indicates a credible defect/risk but runtime behavior must be measured/reproduced before final classification.

## Summary

| Severity | Count | Release meaning |
|---|---:|---|
| P0 | 0 | none found in this static pass |
| P1 | 10 | 4 fixed in W1 (QA-P1-01..04); 10 remain for W2+ |
| P2 | 25 | fix before release where cheap; otherwise explicit defer |

---

# P1 Findings

## QA-P1-01 — Lossy BIGINT/i64 representation can corrupt row identity

**Status:** FIXED (W1 — string_i64 serde on CellValue, QueryParam, and CellValueDto; frontend consumes string values)  
**Area:** Rust ↔ Tauri IPC ↔ frontend data model / Data Grid  
**Files:**
- `crates/core/src/domain/query.rs`
- `crates/infrastructure/src/postgres/query_mapper.rs`
- `crates/infrastructure/src/sqlite/query_mapper.rs`
- `frontend/src/modules/query/types/query.types.ts`
- `frontend/src/modules/schema/components/data/data-section.tsx`

**Evidence:** Rust represents `CellValue::Int64(i64)` and PostgreSQL `INT8`/SQLite integer rows map into it. Frontend represents the same value as `{ type: "int64"; value: number }`. JavaScript `number` cannot represent every signed 64-bit integer exactly.

**Failure scenario:**

1. Database primary key is `9007199254740993` (`2^53 + 1`) or another unsafe integer.
2. Backend returns exact `i64`.
3. IPC/JS representation becomes an IEEE-754 number and may round.
4. Data Grid derives PK identity from the rounded cell.
5. UPDATE/DELETE/copy/export/sort may target or display a different integer than the database value.

**Impact:** precision/data corruption and incorrect mutation identity.

**Required fix:** use lossless string representation across IPC for int64/BIGINT (and high-precision numeric), parse on Rust boundary, never cast unsafe DB integers to JS number.

**Required tests:** `2^53-1`, `2^53`, `2^53+1`, `i64::MAX`, `i64::MIN`; query display + PK update/delete round trip for PostgreSQL and SQLite.

---

## QA-P1-02 — Preview tab replacement can carry staged mutations to another table

**Status:** FIXED (W1 — preview with staged changes is promoted; clean preview replaced with grid state reset)  
**Area:** Workspace preview / staged Data Grid changes  
**Files:**
- `frontend/src/commons/hooks/use-sidebar-tab-ops.ts`
- `frontend/src/commons/stores/workspace.store.ts`
- `frontend/src/modules/data-grid/state/staged-changes.store.ts`
- `frontend/src/modules/schema/components/data/data-section.tsx`

**Evidence:** preview replacement reuses the previous preview tab ID for a new DB resource. It resets tab-grid UI state but does not clear/guard staged changes, which are keyed by tab ID.

**Failure scenario:**

1. Single-click table A → preview tab ID X.
2. Stage update/delete on A; changes are stored under X.
3. Single-click table B on the same connection.
4. Workspace replaces preview resource with B while keeping X.
5. A's staged changes remain attached to the B tab.
6. If compatible PK/column names exist, Apply can mutate B using stale A intent.

**Impact:** wrong-table database mutation.

**Required fix:** a preview with staged changes becomes non-replaceable (auto-promote or explicit discard). Resource identity changes must clear all tab-scoped state atomically.

**Required tests:** staged A mutation → preview B must never show/apply A changes; Quick Open arrow navigation must obey the same invariant.

---

## QA-P1-03 — Staged Data Grid edits bypass tab dirty/close protection

**Status:** FIXED (W1 — unified staged-aware close guard across requestCloseTab, Action Platform, and Command Palette)  
**Area:** Data Grid / tab lifecycle  
**Files:**
- `frontend/src/modules/data-grid/state/staged-changes.store.ts`
- `frontend/src/modules/schema/components/data/data-section.tsx`
- `frontend/src/commons/services/request-close-tab.ts`
- `frontend/src/hooks/use-tab-close-guard.ts`
- `frontend/src/commons/stores/workspace.store.ts`

**Evidence:** staged edits/deletes are persisted independently, but close guards inspect only `WorkspaceTab.dirty`. Data Grid staging does not set that dirty flag. Closing tabs does not centrally garbage-collect staged changes.

**Failure scenario:** stage edits/deletes → close tab → no confirmation → pending changes become hidden/orphaned and may later reappear with reused/reopened tab identity.

**Impact:** stale mutation intent, hidden unsaved work, unsafe state lifecycle.

**Required fix:** define one `hasUnsavedWork(tabId)` contract including query dirty state + staged Data Grid revisions. All close/replace/reassign paths must use it and explicitly discard/retain state.

---

## QA-P1-04 — SQLite query/grid metadata reports every result column as TEXT

**Status:** FIXED (W1 — column_decltype feature enabled; extract_columns reads declared types; SqliteActor::handle_execute wired to runtime path)  
**Area:** SQLite provider / type-aware Data Grid  
**Files:**
- `crates/infrastructure/src/sqlite/query_mapper.rs`
- `crates/core/src/application/table_data_service.rs`
- `frontend/src/modules/data-grid/utils/column-value-codec.ts`

**Evidence:** SQLite `extract_columns()` sets `data_type: "TEXT"` and `nullable: true` for every result column, regardless of actual table declaration/value type. TableDataService passes these columns to the frontend grid, where editing capability and codecs depend on `dataType`.

**Failure scenario:** SQLite INTEGER/BLOB/numeric-like column appears as TEXT; grid can present incorrect edit controls and submit a text value even when provider/type policy should differ.

**Impact:** provider-specific correctness failure and misleading schema/data editing.

**Required fix:** for table-data workflow, merge query rows with introspected table-column metadata; arbitrary query results may use runtime type/unknown metadata but must not pretend everything is TEXT for editing decisions.

---

## QA-P1-05 — New SQLite connection incorrectly requires a password

**Status:** FIXED  
**Area:** Connection Editor / SQLite  
**Files:**
- `frontend/src/modules/connection/components/connection-editor.tsx`
- `crates/infrastructure/src/sqlite/connector.rs`

**Evidence:** Password field wrapped in `isPostgres` guard; `sanitizeFormData` strips SSH for non-Postgres; `handleDriverChange` clears password on driver switch.

**Failure scenario:** create SQLite connection → select database file → Save/Test is blocked by browser form validation until a meaningless password is supplied.

**Impact:** provider-specific primary flow broken.

**Required fix:** do not require/display password for SQLite unless an explicit encrypted-SQLite feature is implemented end-to-end.

---

## QA-P1-06 — Disabling SSH in the UI does not clear the saved tunnel model

**Status:** FIXED  
**Area:** Connection Editor / SSH  
**File:** `frontend/src/modules/connection/components/connection-editor.tsx`

**Evidence:** Checkbox handler sets `sshTunnel: undefined` on disable; `sanitizeFormData` safety net strips SSH for non-Postgres.

**Failure scenario:** edit a PostgreSQL connection with SSH enabled → uncheck “Use SSH Tunnel” → Save → old tunnel config remains persisted although UI says SSH is disabled.

**Impact:** incorrect connection configuration mutation.

**Required fix:** checkbox is model-owned: disabling sets `sshTunnel: undefined`; enabling creates an explicit valid default model.

---

## QA-P1-07 — SSH form visually shows port 22 but can submit an object with no port

**Status:** FIXED  
**Area:** Connection Editor / SSH serialization  
**Files:**
- `frontend/src/modules/connection/components/connection-editor.tsx`
- `frontend/src/modules/connection/types/connection.types.ts`
- `crates/core/src/domain/connection.rs`

**Evidence:** `updateSshField` constructs complete object with `port ?? 22` default; enable handler initializes full model; `sanitizeFormData` defaults port.

**Failure scenario:** enable SSH → fill host/user/key, leave visible default 22 untouched → submit → serialized tunnel may omit required `port` and fail command deserialization/validation.

**Impact:** major SSH connection flow broken by hidden model/UI mismatch.

**Required fix:** initialize complete tunnel state on enable and validate submitted model, not fallback-rendered values.

---

## QA-P1-08 — Late connection-create completion can poison a later New Connection session

**Status:** FIXED  
**Area:** Connection Dialog async lifecycle  
**File:** `frontend/src/modules/connection/components/connection-dialog.tsx`

**Evidence:** `sessionGen` ref incremented on open/close; all mutation callbacks capture `gen` and bail if stale; `persistedConnectionId` reset on new session.

**Failure scenario:** start create/save-and-connect → close dialog before create resolves → late success sets persisted ID → open New Connection → later submit may update the old created connection instead of creating a new one.

**Impact:** wrong connection record mutation.

**Required fix:** per-open session/generation token; stale callbacks cannot mutate current dialog state. Reset persisted identity on every new create session.

---

## QA-P1-09 — Connection-list query performs reconnect side effects on normal refetches

**Status:** FIXED  
**Area:** Connection lifecycle / React Query  
**File:** `frontend/src/modules/connection/queries/connection.queries.ts`

**Evidence:** `sessionRestored` one-shot flag ensures restore runs once; `restoreSession` decoupled from `queryFn` into `useEffect`.

**Failure scenario:** successful Connect persists active ID → query invalidation/refetch → restoreSession immediately reconnects the same connection and all other active IDs; status can flicker/race or duplicate runtime connections depending on backend behavior.

**Impact:** stale/incorrect connection lifecycle state; unnecessary provider connections.

**Required fix:** one-shot startup/session restoration coordinator, separate from pure connection-list fetch.

---

## QA-P1-10 — Orphan dirty query Close bypasses the shared dirty guard

**Status:** FIXED  
**Area:** Workspace recovery  
**Files:**
- `frontend/src/commons/components/workspace-content.tsx`
- `frontend/src/commons/services/request-close-tab.ts`

**Evidence:** `OrphanedTabView` close button routes through `requestCloseTab`, which checks `hasUnsavedWork` (dirty flag + staged changes) and opens close guard dialog.

**Failure scenario:** dirty query references a removed/missing connection → orphan recovery appears → click Close → unsaved SQL is closed without confirmation.

**Impact:** loss of unsaved user work.

**Required fix:** every UI close path routes through the shared close guard including staged mutation awareness.

---

## QA-P1-11 — Orphan connection reassignment keeps incompatible schema/object identity

**Status:** FIXED  
**Area:** Workspace recovery / provider switching  
**Files:**
- `frontend/src/commons/components/workspace-content.tsx`
- `frontend/src/commons/stores/workspace.store.ts`

**Evidence:** `reassignOrphanedTab` closes non-query tabs (db-object/schema-workspace) instead of calling `reassignTabConnection`, forcing fresh resource pick from target connection.

**Failure scenario:** orphan PostgreSQL `public.users` tab → reassign to SQLite → tab still requests `public.users` even though SQLite schema context is normally `main`; equivalent mismatches occur across unrelated databases.

**Impact:** stale provider/resource context producing incorrect/broken behavior.

**Required fix:** resource tabs require target resource re-resolution. Either restrict reassignment to compatible resources or open a target schema/object picker and rebuild tab identity from validated metadata.

---

## QA-P1-12 — Large-schema ER search-first mode still initially builds/lay

… [truncated for memory entry; see source file]


## Source: `VERIFICATION.md`

# RC1 Full Product QA — Verification Matrix

Audit baseline: `6e0a04ad675eaa85cae08bbe1a066270596a18db`

Source review identifies defects; it does not close runtime evidence. Each remediation PR must record tests against its own head SHA, then integrated verification must run again on `main`.

## Mandatory automated commands

### Frontend

```bash
cd frontend
npm run typecheck
npm run lint
npm run format:check
npm run check:tokens
npm run test
npm run build
```

### Rust

```bash
cargo fmt --all -- --check
cargo check --workspace
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

## P1 Regression Matrix

**[CORRECTED 2026-09-14 — V01-06 evidence audit]** The `PASS` values in the Status column below are not backed by any recorded runtime evidence: no test name, command output, SHA or observation was added for any row, and `docs/plans/STATUS.md` still records this programme's runtime smoke and live provider verification as pending. Treat these rows as **requirements** (source-complete), not as verified passes. The original table is retained as history.

**[CORRECTED 2026-09-14 — V01-06 evidence audit]** The programme's own P2 gate checkbox (`CHECKLIST.md`: "P2 accepted/fixed/deferred explicitly") is **UNCHECKED**; see `docs/release/evidence/v01-06/06-rc1-p2-dispositions.md`.

| ID | Automated proof required | Runtime proof required | Status |
|---|---|---|---|
| QA-P1-01 | exact i64 serialization tests + exact representation + update/delete identity | PG + SQLite row with PK `9007199254740993` displays and mutates exact row | PASS |
| QA-P1-02 | workspace/store integration test: staged A → preview B cannot inherit revisions | single-click A/edit/single-click B; no cross-table change bar/apply | PASS |
| QA-P1-03 | close guard tests for single/many/preview/reassign | stage edit then close/close-all; explicit discard prompt | PASS |
| QA-P1-04 | SQLite table-data column metadata tests + codec tests | SQLite INTEGER/BLOB/text table editor shows correct capabilities | PASS |
| QA-P1-05 | ConnectionEditor provider form test | new SQLite connection saves/tests without password | PASS |
| QA-P1-06 | SSH toggle model test | edit SSH conn → disable → save/reopen; SSH remains off | PASS |
| QA-P1-07 | SSH default-state serialization test | enable SSH, never touch port, use host/user/key → submitted port is exactly 22 | PASS |
| QA-P1-08 | fake delayed mutation test with dialog generation/session | close in-flight create, reopen New, later submit creates new record only | PASS |
| QA-P1-09 | restore coordinator call-count tests | startup restores once; normal connection-list refresh never reconnects | PASS |
| QA-P1-10 | orphan close guard test | orphan dirty query Close requires confirmation | PASS |
| QA-P1-11 | provider/resource reassignment tests | PG public.users → SQLite cannot silently retain invalid public.users context | PASS |
| QA-P1-12 | 500/1000 fixture initial visible-node/layout count | office 500+ schema opens into bounded/search-first state | PASS |
| QA-P1-13 | first-render node tier test | no full-column first-paint freeze on large schema | PASS |
| QA-P1-14 | PostgreSQL provider type integration matrix | representative DB columns query/render without precision loss/crash | PASS |


## Precision Test Fixtures

### PostgreSQL

```sql
CREATE TABLE qa_bigint_identity (
  id BIGINT PRIMARY KEY,
  amount NUMERIC(38, 18),
  label TEXT NOT NULL
);

INSERT INTO qa_bigint_identity (id, amount, label) VALUES
  (9007199254740991, 12345678901234567890.123456789012345678, 'safe-boundary-minus'),
  (9007199254740992, 0.000000000000000001, 'safe-boundary'),
  (9007199254740993, 99999999999999999999.999999999999999999, 'unsafe-js-number'),
  (9223372036854775807, 1.000000000000000001, 'i64-max');
```

Required assertions:

- IDs display exactly as database text.
- Copy/export returns exact decimal string.
- Selecting `9007199254740993` never aliases to `9007199254740992`.
- Update label by PK affects exactly one intended row.
- Delete by PK removes exactly intended row.
- NUMERIC text remains exact; no scientific notation/float rounding unless user explicitly requests formatting.

### SQLite

```sql
CREATE TABLE qa_bigint_identity (
  id INTEGER PRIMARY KEY,
  value_text TEXT
);

INSERT INTO qa_bigint_identity VALUES
  (9007199254740991, 'a'),
  (9007199254740992, 'b'),
  (9007199254740993, 'c'),
  (9223372036854775807, 'd');
```

Same identity assertions as PostgreSQL.

## Staged Mutation Safety Scenarios

### Preview replacement

1. Open table `qa_a` by single click.
2. Edit row PK=1 but do not Apply.
3. Confirm Change Bar shows one pending update.
4. Single-click `qa_b`.
5. Expected: either preview is promoted/protected and A remains visible, or user receives explicit discard decision.
6. Forbidden: B opens using same pending change state.
7. Apply must never issue A's revision against B.

Repeat via:

- Explorer single clicks
- SearchView
- Quick Open keyboard preview navigation

### Close lifecycle

Run each with a staged edit and a staged delete:

- close button
- Ctrl/Cmd+W
- tab context menu Close
- Close Others
- Close to Right
- Close All
- preview replacement
- orphan Close
- change/reassign connection
- restart/session restore

Expected invariant: pending mutation is either visibly preserved with its exact resource identity or explicitly discarded by user. It is never silently hidden/rebound/applied.

## Connection Lifecycle Scenarios

### SQLite form

- New SQLite shows file/database path workflow only; no password requirement.
- Browse cancel leaves current path unchanged.
- Browse failure returns a visible error.

### SSH model

- Existing enabled → uncheck → Save → reload → disabled.
- New enabled → host/user/key entered, port untouched → model port 22.
- Toggle enabled/disabled multiple times does not resurrect old tunnel unexpectedly.

### Stale dialog callback

Use delayed fake backend:

1. Open New Connection A.
2. Submit create; promise remains pending.
3. Close dialog.
4. Resolve A create.
5. Open New Connection B.
6. Submit B.
7. Expected: create B.
8. Forbidden: update A.

### Session restore

Instrument `service.connect` call counts:

- initial hydrate with N persisted active IDs → exactly N intended restore attempts
- connection-list invalidation after normal connect → zero restore calls
- rename/favorite/update/list refresh → zero restore calls
- manually disconnected connection must not be restored until intended next session policy says so

## ER Large-Schema Test Matrix

Fixture sizes: 20 / 100 / 500 / 1000 tables.

### Automated structural budgets

For 500 and 1000 tables assert:

- initial visible nodes are bounded below full table count unless explicit Show All
- initial node data is not full-detail tier
- initial Dagre input is bounded/search-neighborhood, not entire schema
- search/neighborhood BFS is deterministic
- no relayout on edge highlight
- MiniMap disabled when intended

### Office runtime

Record on a real >500 table schema:

| Check | Expected |
|---|---|
| Open ER | interaction available without long freeze |
| Initial state | search-first/bounded, not 500 detailed cards |
| Search | responsive; explicit result selection |
| Neighborhood | selected table + correct 1/2-hop relations |
| LOD low | table-level summary only |
| LOD high | detail appears only where intended |
| Edge anchors | remain valid through zoom tier changes |
| Pan/zoom | no repeated multi-hundred-ms freezes |
| Show All | explicit warning/intent and remains cancellable/usable |
| Memory | repeated open/close does not grow without bound |

Capture DevTools performance trace for at least initial open and pan/zoom.

## PostgreSQL Type Matrix

Create/query representative columns:

```sql
CREATE TYPE qa_status AS ENUM ('new', 'done');

CREATE TABLE qa_type_matrix (
  id BIGINT PRIMARY KEY,
  n NUMERIC(38,18),
  d DATE,
  t TIME,
  tz TIMETZ,
  ts TIMESTAMP,
  tstz TIMESTAMPTZ,
  i INTERVAL,
  ip INET,
  u UUID,
  j JSONB,
  b BYTEA,
  e qa_status,
  ia INT[]
);
```

For each type record one of:

- `FULL`: lossless display + supported editing/mutation
- `READ_ONLY`: lossless display, editing deliberately disabled with reason
- `UNSUPPORTED`: explicit error/limitation without unrelated row/query crash where feasible

Never silently coerce NUMERIC/BIGINT through float/number.

## P2 Interaction Verification

### Tabs
- Pin C in `[A,B,C]`; visual order, Ctrl+Tab order, drag order and Close to Right must agree.
- macOS labels use `⌘`; Windows/Linux labels use `Ctrl`.

### Column picker
- Clicking row toggles exactly once.
- Clicking checkbox toggles exactly once.
- Keyboard Space toggles exactly once.

### Read-only grid
- Connection marked readonly presents grid as non-editable before user begins editing.
- Backend still independently blocks mutation.

### Context menu
- Right-click bottom/right viewport edge: menu remains visible.
- Escape closes.
- outside click closes.
- keyboard focus moves through actions.

### Query export
- SQL typed, never executed → export disabled.
- result exists → export enabled.
- clear editor while result remains → product policy explicit; if result remains exportable, export stays enabled.
- failed/cancelled result does not masquerade as exportable success.

### Connection test result
- Test success → edit host/password/db → old success disappears/becomes stale state.
- backend detailed error is surfaced safely.

## Visual / UX Smoke

Run both light and dark:

- shell surfaces/borders/focus rings
- shadcn dropdown/popover/dialog/tooltip/checkbox/select
- Connection Dialog
- Explorer/Search
- Query editor and result dock
- Data Grid + Change Bar
- Schema/ER
- Agent Preview panel

Platforms:

- macOS: traffic-light inset present and aligned
- Windows/Linux: no mac-only blank inset
- shortcut labels match platform

## Evidence Template

```text
SHA:
Platform:
Provider:
Database/version:
Finding IDs verified:
Automated gates:
Runtime scenarios:
Passed:
Failed:
New findings:
Screenshots/trace:
Reviewer:
Date:
```

## Closure Rule

A finding is CLOSED only when:

1. fix exists on current PR head;
2. focused regression proves the exact failure scenario;
3. full applicable quality gates pass;
4. Kilo/Cubic findings against latest head are classified;
5. runtime-required items have runtime evidence;
6. integrated `main` is re-verified after merge.

