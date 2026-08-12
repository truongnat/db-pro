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

**Status:** SOURCE_CONFIRMED  
**Area:** Connection Editor / SQLite  
**Files:**
- `frontend/src/modules/connection/components/connection-editor.tsx`
- `crates/infrastructure/src/sqlite/connector.rs`

**Evidence:** SQLite branch renders password with `required={!isEdit || driverChanged}`. Backend SQLite connector accepts `_password` and ignores it.

**Failure scenario:** create SQLite connection → select database file → Save/Test is blocked by browser form validation until a meaningless password is supplied.

**Impact:** provider-specific primary flow broken.

**Required fix:** do not require/display password for SQLite unless an explicit encrypted-SQLite feature is implemented end-to-end.

---

## QA-P1-06 — Disabling SSH in the UI does not clear the saved tunnel model

**Status:** SOURCE_CONFIRMED  
**Area:** Connection Editor / SSH  
**File:** `frontend/src/modules/connection/components/connection-editor.tsx`

**Evidence:** `showSsh` controls visibility independently. Unchecking only calls `setShowSsh`; existing `formData.sshTunnel` remains and is submitted.

**Failure scenario:** edit a PostgreSQL connection with SSH enabled → uncheck “Use SSH Tunnel” → Save → old tunnel config remains persisted although UI says SSH is disabled.

**Impact:** incorrect connection configuration mutation.

**Required fix:** checkbox is model-owned: disabling sets `sshTunnel: undefined`; enabling creates an explicit valid default model.

---

## QA-P1-07 — SSH form visually shows port 22 but can submit an object with no port

**Status:** SOURCE_CONFIRMED  
**Area:** Connection Editor / SSH serialization  
**Files:**
- `frontend/src/modules/connection/components/connection-editor.tsx`
- `frontend/src/modules/connection/types/connection.types.ts`
- `crates/core/src/domain/connection.rs`

**Evidence:** UI renders `formData.sshTunnel?.port ?? 22`, but enabling SSH does not initialize `sshTunnel`. Editing host/user/key creates a partial object; untouched port 22 may never be stored. Rust `SshTunnelConfig.port: u16` is required.

**Failure scenario:** enable SSH → fill host/user/key, leave visible default 22 untouched → submit → serialized tunnel may omit required `port` and fail command deserialization/validation.

**Impact:** major SSH connection flow broken by hidden model/UI mismatch.

**Required fix:** initialize complete tunnel state on enable and validate submitted model, not fallback-rendered values.

---

## QA-P1-08 — Late connection-create completion can poison a later New Connection session

**Status:** SOURCE_CONFIRMED  
**Area:** Connection Dialog async lifecycle  
**File:** `frontend/src/modules/connection/components/connection-dialog.tsx`

**Evidence:** dialog uses `effectiveConnectionId = editConnectionId ?? persistedConnectionId`; close resets persisted ID, but an in-flight create callback may later set it again. Opening a new create session does not unconditionally generation-reset/ignore stale callbacks.

**Failure scenario:** start create/save-and-connect → close dialog before create resolves → late success sets persisted ID → open New Connection → later submit may update the old created connection instead of creating a new one.

**Impact:** wrong connection record mutation.

**Required fix:** per-open session/generation token; stale callbacks cannot mutate current dialog state. Reset persisted identity on every new create session.

---

## QA-P1-09 — Connection-list query performs reconnect side effects on normal refetches

**Status:** SOURCE_CONFIRMED; runtime effect must be measured  
**Area:** Connection lifecycle / React Query  
**File:** `frontend/src/modules/connection/queries/connection.queries.ts`

**Evidence:** `useConnectionList` query function calls `restoreSession(connections)`. Create/update/connect/delete flows invalidate `connections`, so generic refetches can invoke session restoration and call `service.connect()` for persisted active IDs again.

**Failure scenario:** successful Connect persists active ID → query invalidation/refetch → restoreSession immediately reconnects the same connection and all other active IDs; status can flicker/race or duplicate runtime connections depending on backend behavior.

**Impact:** stale/incorrect connection lifecycle state; unnecessary provider connections.

**Required fix:** one-shot startup/session restoration coordinator, separate from pure connection-list fetch.

---

## QA-P1-10 — Orphan dirty query Close bypasses the shared dirty guard

**Status:** SOURCE_CONFIRMED  
**Area:** Workspace recovery  
**Files:**
- `frontend/src/commons/components/workspace-content.tsx`
- `frontend/src/commons/services/request-close-tab.ts`

**Evidence:** OrphanedTabView closes directly with `useWorkspaceStore.getState().closeTab(tabId)` rather than `requestCloseTab`.

**Failure scenario:** dirty query references a removed/missing connection → orphan recovery appears → click Close → unsaved SQL is closed without confirmation.

**Impact:** loss of unsaved user work.

**Required fix:** every UI close path routes through the shared close guard including staged mutation awareness.

---

## QA-P1-11 — Orphan connection reassignment keeps incompatible schema/object identity

**Status:** SOURCE_CONFIRMED  
**Area:** Workspace recovery / provider switching  
**Files:**
- `frontend/src/commons/components/workspace-content.tsx`
- `frontend/src/commons/stores/workspace.store.ts`

**Evidence:** orphan UI allows choosing any saved connection. `reassignTabConnection` changes connection ID/resource key for db-object and schema-workspace tabs but preserves old schema/object context. Query tabs clear context, but object tabs do not.

**Failure scenario:** orphan PostgreSQL `public.users` tab → reassign to SQLite → tab still requests `public.users` even though SQLite schema context is normally `main`; equivalent mismatches occur across unrelated databases.

**Impact:** stale provider/resource context producing incorrect/broken behavior.

**Required fix:** resource tabs require target resource re-resolution. Either restrict reassignment to compatible resources or open a target schema/object picker and rebuild tab identity from validated metadata.

---

## QA-P1-12 — Large-schema ER search-first mode still initially builds/layouts the full schema

**Status:** SOURCE_CONFIRMED  
**Area:** ER Diagram performance  
**File:** `frontend/src/modules/er-diagram/components/er-diagram.tsx`

**Evidence:** for `>200` tables, `neighborhoodSeed` initially null and `showAll` false. `neighborhoodSet` therefore returns null, and null means node construction uses every table in the selected schema.

**Failure scenario:** open 500-table schema → before user searches anything, component still builds 500 nodes and runs Dagre on full graph. This contradicts the intended search-first protection.

**Impact:** release-blocking large-schema freeze risk.

**Required fix:** large-schema initial state must be bounded: empty/search prompt, selected entry, or capped overview. `Show all N` is the explicit opt-in to full graph.

---

## QA-P1-13 — ER first paint starts in full-detail tier before viewport/fitView can reduce detail

**Status:** SOURCE_CONFIRMED  
**Area:** ER Diagram first-paint performance  
**Files:**
- `frontend/src/modules/er-diagram/components/er-diagram.tsx`
- `frontend/src/modules/er-diagram/components/table-node.tsx`

**Evidence:** `currentTier` initializes to tier 2 and initial node data also uses `zoomTier: 2`. Full column lists can mount before `onViewportChange` receives the fit-view zoom and downgrades detail.

**Failure scenario:** 500 tables × many columns → first commit mounts detailed rows/handles, then fitView zooms out. LOD arrives after the expensive initial render.

**Impact:** first-open freeze/jank despite later LOD.

**Required fix:** initialize large graph nodes at safe tier 0/1 before first render; selected nodes may hydrate detail after viewport is known.

---

## QA-P1-14 — PostgreSQL row mapper lacks a lossless, explicit contract for common non-basic types

**Status:** RUNTIME_CONFIRM but release-blocking until type matrix is proven  
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

**Status:** SOURCE_CONFIRMED  
**Files:** `workspace.store.ts`, `workspace-tab-bar.tsx`, `use-tab-keyboard.ts`, `tab-context-menu.tsx`

Pin toggles only a boolean. Tab bar renders pinned first, while drag/drop and Ctrl+Tab use raw store order. Visual order, keyboard order and drag indices can disagree.

**Fix:** canonicalize one ordered tab list and use it for rendering, DnD, close-to-right and keyboard cycling.

## QA-P2-02 — Tab context-menu shortcuts are hardcoded Ctrl on macOS

**Status:** SOURCE_CONFIRMED  
**File:** `frontend/src/commons/components/tab-context-menu.tsx`

Menu shows `Ctrl+W` / `Ctrl+Shift+T` rather than platform-formatted shortcuts.

## QA-P2-03 — Topbar reserves macOS traffic-light space on every OS

**Status:** SOURCE_CONFIRMED  
**File:** `frontend/src/commons/components/shell/topbar.tsx`

Left group always has `pl-14`; Windows/Linux get unnecessary blank inset.

## QA-P2-04 — Agent panel is not visibly marked Preview/Coming Soon

**Status:** SOURCE_CONFIRMED  
**File:** `frontend/src/commons/components/ide/agent-panel.tsx`

Header says only “Agent”; starter prompts and composer look functional while only send/new buttons are disabled. 0.1 smoke policy expects explicit Preview/Coming Soon.

## QA-P2-05 — Agent panel uses macOS-only `⌘↵` shortcut hint

**Status:** SOURCE_CONFIRMED  
**File:** `agent-panel.tsx`

Windows/Linux UI still renders `⌘↵`.

## QA-P2-06 — Significant Agent/Connection UI strings bypass i18n

**Status:** SOURCE_CONFIRMED  
**Files:** `agent-panel.tsx`, `connection-editor.tsx`, query result empty/message sections, tab context menu

Hardcoded English breaks the existing EN/JA localization contract and produces mixed-language UI.

## QA-P2-07 — Data Grid Columns picker can double-toggle when checkbox itself is clicked

**Status:** RUNTIME_CONFIRM (strong source evidence)  
**File:** `frontend/src/modules/data-grid/components/data-toolbar.tsx`

Parent row `onClick` and child Checkbox `onCheckedChange` both call `onToggleHiddenColumn`. A checkbox click can bubble and toggle twice.

**Fix:** single event owner or stop propagation; add interaction test.

## QA-P2-08 — Read-only connections still present editable Data Grid affordances

**Status:** SOURCE_CONFIRMED UX mismatch  
**Files:** `data-section.tsx`, `unified-grid.tsx`, `table_data_service.rs`

Backend correctly blocks mutation for read-only connections, but DataSection derives editability from PK presence and does not surface connection readonly state. User can stage edits only to fail at Apply.

## QA-P2-09 — Custom grid context menu can render off-screen and lacks desktop menu behavior

**Status:** SOURCE_CONFIRMED  
**File:** `frontend/src/modules/unified-grid/components/unified-grid.tsx`

Menu uses raw `clientX/clientY`, closes on mouse leave, and lacks viewport clamping, Escape/outside-click/focus/menu semantics.

## QA-P2-10 — Column and shell resize handles are mouse-only

**Status:** SOURCE_CONFIRMED  
**Files:** `unified-grid.tsx`, `app-shell.tsx`

Resize affordances are `div` + `onMouseDown`; no keyboard adjustment/ARIA separator behavior.

## QA-P2-11 — Resize drag cleanup is not guaranteed on component unmount/interruption

**Status:** SOURCE_CONFIRMED edge case  
**Files:** `unified-grid.tsx`, `app-shell.tsx`

Document listeners and `body.style.cursor/userSelect` are restored only by mouseup handlers. Unmount/interruption can leave leaked listeners/body state.

## QA-P2-12 — Large query-result sorting is synchronous on the main thread

**Status:** SOURCE_CONFIRMED performance risk  
**File:** `frontend/src/modules/query/components/query-tab-content.tsx`

Sorting copies/sorts the entire result array in React `useMemo`. Connection `maxRows` supports large result counts; row virtualization does not reduce sort cost.

## QA-P2-13 — Sidebar Search scans and renders every matching object on each keystroke

**Status:** SOURCE_CONFIRMED performance risk  
**File:** `frontend/src/commons/components/shell/sidebar-views/search-view.tsx`

No memoized index/debounce/virtual list. Thousands of tables/views can produce typing lag.

## QA-P2-14 — Explorer still mounts every table/view row for expanded large schemas

**Status:** SOURCE_CONFIRMED performance risk  
**File:** `frontend/src/commons/components/shell/sidebar-views/explorer-view.tsx`

P3 removed repeated filtering with maps, but expanded groups still map the entire object list into DOM nodes. Add virtualization/collapse safeguards for very large schemas.

## QA-P2-15 — Connection Test success/error becomes stale after form edits

**Status:** SOURCE_CONFIRMED UX state bug  
**Files:** `connection-dialog.tsx`, `connection-editor.tsx`

Mutation state is not reset when host/database/password/SSL/SSH values change. A green “success” can describe a previous config while user is editing a different one.

## QA-P2-16 — Test Connection hides useful backend error detail

**Status:** SOURCE_CONFIRMED  
**File:** `connection-dialog.tsx`

Test failure reports generic `connection.testFailed` rather than available structured `userMessage` used by other connection actions.

## QA-P2-17 — SQLite Browse has no user-visible error path

**Status:** SOURCE_CONFIRMED  
**File:** `connection-editor.tsx`

Native dialog call is awaited without a catch/feedback path. Plugin/capability/native failure can become an unhandled action with no explanation.

## QA-P2-18 — `driverChanged` means “ever changed”, not “different from original”

**Status:** SOURCE_CONFIRMED  
**File:** `connection-editor.tsx`

Switching Postgres → SQLite → Postgres leaves `driverChanged=true`, so password-required/reset behavior may be triggered even after returning to the initial driver.

## QA-P2-19 — Duplicate connection silently omits credentials

**Status:** SOURCE_CONFIRMED behavior; product decision required  
**File:** `connection.queries.ts`

Duplicate copies config but calls create with empty password. If this is intentional security behavior, UI must explicitly tell the user the duplicated connection requires credentials.

## QA-P2-20 — Favorite optimistic update has no rollback

**Status:** SOURCE_CONFIRMED  
**File:** `connection.queries.ts`

Favorite state is toggled optimistically without onError rollback/invalidation, so failed persistence can leave stale UI.

## QA-P2-21 — SQLite recent connection subtitle renders meaningless host/port

**Status:** SOURCE_CONFIRMED  
**File:** `welcome-view.tsx`

Recent connection subtitle always renders `host:port / database`; SQLite defaults can show `:0 / <path>` instead of a file-focused label.

## QA-P2-22 — Query Export enablement is tied to SQL text instead of actual result state

**Status:** SOURCE_CONFIRMED  
**Files:** `query-command-bar.tsx`, `query-tab-content.tsx`

Export menu disables on `!hasSql`. SQL can exist before any result (export wrongly enabled), or result can remain after editor clear (export wrongly disabled). Manual release checklist already expects result-driven availability.

## QA-P2-23 — Query dirty replacement uses native `window.confirm`

**Status:** SOURCE_CONFIRMED  
**File:** `query-tab-content.tsx`

History/import overwrite uses browser-native confirm instead of app AlertDialog/confirmation platform, causing inconsistent Tauri desktop behavior and poor testability.

## QA-P2-24 — ER search auto-picks first substring match without disambiguation

**Status:** SOURCE_CONFIRMED UX behavior  
**File:** `er-diagram.tsx`

Large-schema search uses first `table.name.includes(query)` match as neighborhood seed. Similar names can focus an arbitrary first table while user types.

**Fix:** search results/keyboard selection; Enter chooses explicit seed.

## QA-P2-25 — ER derived large-schema state does unnecessary work and has LOD runtime gaps

**Status:** mixed SOURCE_CONFIRMED / RUNTIME_CONFIRM  
**Files:** `er-diagram.tsx`, `table-node.tsx`

- `tablesInSchema = data.tables.filter(...)` is recreated every render and used in effect dependencies.
- low/medium LOD removes column handles while edges retain handle IDs; React Flow anchor behavior must be verified during dynamic tier transitions.
- Fit View currently uses a synthetic key dispatch instead of owning a React Flow instance/API.

Split this into focused fixes/tests if runtime confirms edge/fit failures.

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
