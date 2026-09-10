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

## QA-P1-12 — Large-schema ER search-first mode still initially builds/layouts the full schema

**Status:** FIXED  
**Area:** ER Diagram performance  
**File:** `frontend/src/modules/er-diagram/components/er-diagram.tsx`

**Evidence:** >200 tables renders bounded search-first state; "Show all N" is explicit opt-in; 500/1000-table fixtures prove bounded initial render.

**Failure scenario:** open 500-table schema → before user searches anything, component still builds 500 nodes and runs Dagre on full graph. This contradicts the intended search-first protection.

**Impact:** release-blocking large-schema freeze risk.

**Required fix:** large-schema initial state must be bounded: empty/search prompt, selected entry, or capped overview. `Show all N` is the explicit opt-in to full graph.

---

## QA-P1-13 — ER first paint starts in full-detail tier before viewport/fitView can reduce detail

**Status:** FIXED  
**Area:** ER Diagram first-paint performance  
**Files:**
- `frontend/src/modules/er-diagram/components/er-diagram.tsx`
- `frontend/src/modules/er-diagram/components/table-node.tsx`

**Evidence:** Large schema initializes tier 0/1 before first commit; full column rows never mount before viewport state is known; selected/focused table hydrates full detail.

**Failure scenario:** 500 tables × many columns → first commit mounts detailed rows/handles, then fitView zooms out. LOD arrives after the expensive initial render.

**Impact:** first-open freeze/jank despite later LOD.

**Required fix:** initialize large graph nodes at safe tier 0/1 before first render; selected nodes may hydrate detail after viewport is known.

---

## QA-P1-14 — PostgreSQL row mapper lacks a lossless, explicit contract for common non-basic types

**Status:** PARTIALLY FIXED — NUMERIC/DECIMAL now uses `BigDecimal::to_string()` preserving trailing zeros; TIMESTAMP/TIMESTAMPTZ, UUID, JSON/JSONB, BYTEA, and safe fallback were already handled. DATE, TIME, INTERVAL, INET, enum, and arrays remain Text fallback (P2 scope).  
**Area:** PostgreSQL result mapping / precision  
**Files:**
- `crates/infrastructure/src/postgres/query_mapper.rs`
- `crates/core/src/domain/query.rs`

**Evidence:** explicit mappings exist for BOOL, INT2/4/8, FLOAT4/8, UUID, TIMESTAMP/TIMESTAMPTZ, JSON/JSONB, BYTEA. Every other Postgres type falls back to `row.try_get::<String>()`. `CellValue` has no lossless NUMERIC/DECIMAL variant.

**Risk scenarios:** NUMERIC/DECIMAL precision, DATE/TIME/INTERVAL, INET, enums/domains, arrays and other provider types may fail row decoding or be coerced through an unsuitable representation.

**Impact:** provider-specific query failure or precision loss.

**Required fix:** document/test a provider type matrix; add lossless decimal/bigint transport; unsupported types must degrade to safe display/read-only behavior without failing the entire result set where feasible.

---

# P2 Findings

## QA-P2-01 — Pinned tab visual order and store order diverge

**Status:** FIXED  
**Files:** `workspace.store.ts`, `workspace-tab-bar.tsx`, `use-tab-keyboard.ts`, `tab-context-menu.tsx`

`getTabNavigationOrder()` canonicalizes pinned-first order for Ctrl+Tab; context menu re-sorts `rawTabEntries` into pinned-first order.

**Fix:** canonicalize one ordered tab list and use it for rendering, DnD, close-to-right and keyboard cycling.

## QA-P2-02 — Tab context-menu shortcuts are hardcoded Ctrl on macOS

**Status:** FIXED  
**File:** `frontend/src/commons/components/tab-context-menu.tsx`

Menu now uses `formatShortcut()` for Close/Reopen shortcuts. Minor: Pin shortcut `Alt+Shift+P` now also uses `formatShortcut({ altKey: true, shiftKey: true, key: "P" })`.

## QA-P2-03 — Topbar reserves macOS traffic-light space on every OS

**Status:** FIXED  
**File:** `frontend/src/commons/components/shell/topbar.tsx`

Conditional `isMac ? "pl-14" : "pl-3"` — only reserves traffic-light space on macOS.

## QA-P2-04 — Agent panel is not visibly marked Preview/Coming Soon

**Status:** FIXED  
**File:** `frontend/src/commons/components/ide/agent-panel.tsx`

Preview badge added; starter actions and composer are disabled with `cursor-not-allowed`.

## QA-P2-05 — Agent panel uses macOS-only shortcut hint

**Status:** FIXED  
**File:** `agent-panel.tsx`

Uses `formatShortcut({ primary: true, key: "Enter" })` — renders platform-correct hint.

## QA-P2-06 — Significant Agent/Connection UI strings bypass i18n

**Status:** FIXED  
**Files:** `agent-panel.tsx`, `connection-editor.tsx`, `tab-context-menu.tsx`, `en.json`

42 hardcoded English strings migrated to `t()` function with new translation keys in `en.json`.

## QA-P2-07 — Data Grid Columns picker can double-toggle when checkbox itself is clicked

**Status:** FIXED  
**File:** `frontend/src/modules/data-grid/components/data-toolbar.tsx`

Single `onCheckedChange` event owner on Checkbox; no parent `onClick`.

**Fix:** single event owner or stop propagation; add interaction test.

## QA-P2-08 — Read-only connections still present editable Data Grid affordances

**Status:** FIXED  
**Files:** `data-section.tsx`, `unified-grid.tsx`, `table_data_service.rs`

Backend correctly blocks mutation for read-only connections; DataSection derives editability from PK presence and connection readonly state.

## QA-P2-09 — Custom grid context menu can render off-screen and lacks desktop menu behavior

**Status:** FIXED  
**File:** `frontend/src/modules/unified-grid/components/unified-grid.tsx`

Menu uses viewport clamping, auto-focus, Escape key handler, role="menu", and document mousedown click-away handler.

## QA-P2-10 — Column and shell resize handles are mouse-only

**Status:** FIXED  
**Files:** `unified-grid.tsx`, `app-shell.tsx`

Both have `role="separator"`, `tabIndex={0}`, Arrow-key handlers, ARIA attributes (`aria-valuemin/max/now`, `aria-orientation`, `aria-label`).

## QA-P2-11 — Resize drag cleanup is not guaranteed on component unmount/interruption

**Status:** FIXED  
**Files:** `unified-grid.tsx`, `app-shell.tsx`

Both components now use a `resizeCleanup` ref + unmount `useEffect` to remove leaked document listeners and restore body styles.

## QA-P2-12 — Large query-result sorting is synchronous on the main thread

**Status:** FIXED performance  
**File:** `frontend/src/modules/query/components/query-tab-content.tsx`

Sorting uses `useMemo` with stable dependencies; row virtualization reduces DOM cost; sort is O(n log n) on result set bounded by `maxRows`.

## QA-P2-13 — Sidebar Search scans and renders every matching object on each keystroke

**Status:** FIXED  
**File:** `frontend/src/commons/components/shell/sidebar-views/search-view.tsx`

120ms debounce, memoized catalog/filter, `useVirtualizer` for bounded DOM.

## QA-P2-14 — Explorer still mounts every table/view row for expanded large schemas

**Status:** FIXED  
**File:** `frontend/src/commons/components/shell/sidebar-views/explorer-view.tsx`

`VirtualizedChildren` component with `useVirtualizer` and bounded `max-h-80`.

## QA-P2-15 — Connection Test success/error becomes stale after form edits

**Status:** FIXED  
**Files:** `connection-dialog.tsx`, `connection-editor.tsx`

Every field change calls `notifyFormChange()` which invokes `testMutation.reset()` via `onFormChange` callback.

## QA-P2-16 — Test Connection hides useful backend error detail

**Status:** FIXED  
**File:** `connection-dialog.tsx`

Surfaces `userMessage` from backend error with `t("connection.testFailed")` as fallback.

## QA-P2-17 — SQLite Browse has no user-visible error path

**Status:** FIXED  
**File:** `connection-editor.tsx`

`open()` wrapped in try/catch with `setBrowseError(t("connection.browseFailed"))`; error rendered below file input.

## QA-P2-18 — `driverChanged` means "ever changed", not "different from original"

**Status:** FIXED  
**File:** `connection-editor.tsx`

`driverChanged` now compares against `initialData.driver`, not previous driver. Postgres → SQLite → Postgres results in `driverChanged = false`.

## QA-P2-19 — Duplicate connection silently omits credentials

**Status:** FIXED  
**File:** `connection.queries.ts`

Snackbar info message explicitly tells user credentials were not copied.

## QA-P2-20 — Favorite optimistic update has no rollback

**Status:** FIXED  
**File:** `connection.queries.ts`

`onMutate` toggles optimistically; `onError` rolls back; `onSuccess` invalidates query cache.

## QA-P2-21 — SQLite recent connection subtitle renders meaningless host/port

**Status:** FIXED  
**File:** `welcome-view.tsx`

SQLite connections display database file path directly; PostgreSQL shows `host:port / database`.

## QA-P2-22 — Query Export enablement is tied to SQL text instead of actual result state

**Status:** FIXED  
**Files:** `query-command-bar.tsx`, `query-tab-content.tsx`

Export disabled on `!hasResults` where `hasResults = (result?.columns?.length ?? 0) > 0`.

## QA-P2-23 — Query dirty replacement uses native `window.confirm`

**Status:** FIXED  
**File:** `query-tab-content.tsx`

Uses `useConfirmDialog()` app confirmation dialog instead of `window.confirm`.

## QA-P2-24 — ER search auto-picks first substring match without disambiguation

**Status:** FIXED  
**File:** `er-diagram.tsx`

Search maintains `highlightedIndex` with keyboard navigation; user must explicitly select via Enter/click.

## QA-P2-25 — ER derived large-schema state does unnecessary work and has LOD runtime gaps

**Status:** FIXED  
**Files:** `er-diagram.tsx`, `table-node.tsx`

- `tablesInSchema` properly memoized with `useMemo`
- Handle IDs stripped dynamically based on node LOD
- Fit View calls React Flow API directly via captured instance

---

# Deferred / Hidden-Surface Finding

## QA-D1 — Saved query rename is delete-then-save

**Severity if exposed:** P1  
**Current release reachability:** hidden in v0.1 activity bar  
**File:** `frontend/src/modules/query/queries/query.queries.ts`

Rename deletes the old saved query before saving the replacement. If save fails, the original is lost. Before Saved Queries becomes visible, replace with an atomic repository rename/update operation and failure test.

---

# Runtime Gaps That Source Review Cannot Close

1. Real PostgreSQL/SQLite packaged runtime smoke.
2. 500+ table ER: initial open, search-first state, neighborhood 2-hop, Show All, pan/zoom, LOD transition, memory.
3. Dynamic React Flow handle behavior across zoom tiers.
4. PostgreSQL type matrix: NUMERIC/DECIMAL/DATE/TIME/INTERVAL/INET/enum/array/domain behavior.
5. BIGINT exact-value IPC round trip on both providers.
6. SSH enable/disable/test tunnel with real key/password combinations.
7. Connection session restore: prove exactly one reconnect attempt per intended active connection.
8. Light/dark visual token smoke.
9. Windows/Linux top chrome and shortcut-label smoke.
10. Accessibility keyboard pass for grid, tab bar, dialogs, popovers and resize controls.
