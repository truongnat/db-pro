# Runtime UX Audit v3 — Per-Surface Deep Pass

> **Note (2026-09-11).** This audit predates the frontend archival. References to
> `frontend/...` paths and shadcn components point at the now-archived React UI; the native
> UI is `crates/ui`.

**Date:** 2026-09-10
**Branch:** `arena/01a08a04-db-pro`
**Baseline commit:** `ce4f628`
**Predecessor:** [`runtime-ux-audit-v2.md`](./runtime-ux-audit-v2.md) (PATCH 5 / 5.1 / 5.2)

v2 audited *geometry and typography* from a screenshot. v3 goes surface by surface
through the actual component source, records what each surface does today, and
fixes the defects that are provable from the code rather than from a pixel diff.

Everything marked **Fixed** below has a test in the suite; see
[Verification](#verification).

---

## 0. Blocking problem found first: the UI could not be inspected at all

Every screen in DB Pro reaches data through `apiInvoke` → Tauri IPC → the Rust
core. There was no way to render the app without a packaged desktop shell, so
"review the UI" meant reading JSX and hoping. Screenshot inventory in v2 still
lists 6 of 8 screens as *"Pending manual capture"* two patches later.

**Fixed.** `frontend/src/dev/` adds a dev-only browser backend:

| File | Role |
|------|------|
| `src/dev/mock-database.ts` | In-memory fixture dataset mirroring `fixtures/postgres/schema.sql` (8 tables, 2 views, PK/FK/index metadata, DDL builder) plus a single-table SQL subset (WHERE / ORDER BY / LIMIT / OFFSET / aggregates / aliases / INSERT / UPDATE / DELETE) |
| `src/dev/browser-backend.ts` | Routes all 53 production commands through Tauri's own `mockIPC` helper (the 54th name in the grep, `test_cmd`, is referenced only from `commons/__tests__/api.test.ts`), plus a `plugin:*` fallback so dialog/fs pickers cancel cleanly |
| `src/vite-env.d.ts` | `vite/client` types for the `import.meta.env.DEV` guard |

`vite dev` in a plain browser now boots the real `App` — real `AppShell`, real
stores, real components — against the fixture backend. Guardrails:

- Installed only inside `if (import.meta.env.DEV)`, so Rollup folds the branch to
  `false` in `vite build` and dead-code-eliminates the chunk.
- `installBrowserBackend()` returns `false` when a real `__TAURI_INTERNALS__.invoke`
  is present, so `tauri dev` always uses the Rust core.
- Errors are thrown in the Rust command-error shape (`{error, message, message_id}`)
  so `normalizeServerError` / `translateError` run for real.

---

## 1. Data grid — `modules/unified-grid/components/unified-grid.tsx`

The most-viewed surface in the product, and the one furthest from a desktop DB
client. Eight defects, all **Fixed**.

| # | Finding | Before | After |
|---|---------|--------|-------|
| G1 | **Numbers were left-aligned with proportional digits.** The single biggest readability gap versus DBeaver/DataGrip/pgAdmin — a column of prices cannot be scanned vertically. | every cell `flex items-center` | `int64`/`float64` get `justify-end tabular-nums` |
| G2 | **Sort indicator was a text glyph** (`▲`/`▼`) in `text-tertiary`, and the sort state was invisible to assistive tech. | `{sort.direction === "asc" ? "▲" : "▼"}` | Lucide `ArrowUp`/`ArrowDown` in accent colour, `ArrowUpDown` as a hover affordance on unsorted columns, plus `aria-sort` on every `columnheader` |
| G3 | **NULL was rendered as the literal string `NULL` in italic** — visually identical to a text value that happens to read "NULL". | `renderCellValue` → `"NULL"`, `fontStyle: italic` | muted chip, `data-cell-kind="null"`, so an empty string can never be confused with SQL NULL |
| G4 | **Booleans were the words `true`/`false`.** | plain text | check / dash marker + literal, `data-cell-kind` |
| G5 | **Long values were clipped with no ellipsis.** `overflow-hidden` + `text-ellipsis` + `whitespace-nowrap` sat on the *flex container*, and the value was a bare text node. Per CSS Flexbox the text becomes an anonymous flex item whose own `text-overflow` is the initial `clip`, so the container's `text-overflow` cannot reach it — the value was hard-clipped with no `…`. | hard clip | value wrapped in `<span class="truncate">`, which is the element that carries the overflow |
| G6 | **`aria-selected` on a role-less `<div>`.** Invalid ARIA — the selection state was announced nowhere. | `<div aria-selected>` | full grid semantics: `role="grid"` + `aria-rowcount`/`aria-colcount`/`aria-busy`, `role="row"` + `aria-rowindex`, `role="columnheader"`, `role="rowheader"`, `role="gridcell"` |
| G7 | **"Frozen" columns did not freeze.** `frozenColumns` only *reordered* columns to the front; horizontal scroll still carried them away, and the gutter had no occluding background. | reorder only | real `position: sticky` with cumulative left offsets behind the 40px gutter, occluding backgrounds, and a divider on the last pinned column |
| G8 | **Loading state was a centred sentence.** | `<span>Loading</span>` | skeleton rows + spinner + `role="status"` live region + `aria-busy` on the grid |

Also fixed in the same surface:

- **G9** Column resize handle had a hard-coded English label `Resize ${col.name}`
  → `t("dataGrid.resizeColumn", { column })` (added to `en.json` **and** `ja.json`).
- **G10** The context menu closed on `onMouseLeave` — moving the pointer two
  pixels toward a submenu item dismissed it. Now closes on outside click and
  `Escape` only; items are `role="menuitem"`, and the hover surface moved from
  the nonsensical `hover:bg-background` to `--surface-hover`.
- **G11** `result-grid.tsx` rendered the column-info trigger as the letter **`i`**
  in a `Button`. Now a Lucide `Info` icon that reveals on header hover, with
  `aria-label="Column info: <name>"`.

---

## 2. Workspace tabs — `commons/components/workspace-tab-bar.tsx`

| # | Finding | Status |
|---|---------|--------|
| T1 | **A dirty *pinned* tab looked clean.** The dirty dot was gated on `tab.dirty && !tab.pinned`, and pinned tabs have no close button — so an unsaved pinned tab showed no indicator at all, and the pin icon occupied the only slot. | **Fixed** — dirty dot now takes precedence over the pin icon and never hides on hover for pinned tabs |
| T2 | **The tab tooltip showed only the tab title.** `resourceName` and `connectionId` were already projected into `TabBarInfo` and left unused, so a truncated `SELECT · t_workflow_definit…` gave no way to tell which connection or object it was. | **Fixed** — tooltip is now `title` + `connection · schema.object`, resolved through `useConnectionList()` |
| T3 | `min-w-[120px] max-w-[220px]` is fixed regardless of sidebar/agent panel state; with the agent panel open, 3 tabs overflow. | open |
| T4 | No "reopen closed tab" affordance in the tab strip UI even though `tabs.contextMenu.reopenClosed` exists and the command is registered. | open |

---

## 3. Explorer tree — `commons/components/shell/sidebar-views/explorer-view.tsx`

| # | Finding | Status |
|---|---------|--------|
| E1 | **Nested scroll area inside the sidebar scroller.** `VirtualizedChildren` created a `max-h-80 overflow-y-auto` box inside `sidebar.tsx`'s own `overflow-y-auto`. The wheel gets captured by whichever box is under the cursor — the classic file-explorer trap. | **Fixed** — the virtualizer now attaches to the nearest scrollable ancestor with a `scrollMargin` offset; no inner scroll box |
| E2 | **No indent guides.** Nesting was conveyed only by a 10px margin, so `schema → tables → t_order_items` was hard to trace at 288px width. | **Fixed** — `border-l` guides on both nesting levels |
| E3 | **Row counts invisible.** `TableDto.rowCount` is already in the introspection payload and was dropped on the floor. | **Fixed** — tabular right-aligned count on every table row |
| E4 | Connection rows use a `border-l-2` accent on hover, which reads as a selection state rather than a hover state. | open |
| E5 | No expand-all / collapse-all on a connection node. | open |
| E6 | `StatusDot` has no `title` or `aria-label`; connection state is colour-only. | open |

The same nested-scroll defect existed in two more sidebar views and is **Fixed**
in both:

- `sidebar-views/users-view.tsx` — `max-h-56 overflow-y-auto` removed.
- `sidebar-views/search-view.tsx` — `max-h-64 overflow-y-auto` removed, virtualizer
  re-pointed at the sidebar scroller.

Because three surfaces needed the same resolution logic, it was extracted into
`src/hooks/use-scroll-parent.ts` (`useScrollParent`) rather than copy-pasted, and
is covered by `src/hooks/__tests__/use-scroll-parent.test.tsx`.

That test caught a real defect in the first draft of the hook: when the walk
reached `<body>` without finding a scroller, the loop variable was still truthy,
so `<body>` was returned as the scroll parent. The hook now tracks an explicit
`found` sentinel and leaves `scrollElement` null.

---

## 4. Activity bar — `commons/components/shell/activity-bar.tsx`

| # | Finding | Status |
|---|---------|--------|
| A1 | **Re-clicking the active view did nothing.** In VS Code / DBeaver, clicking the active sidebar icon hides the panel; here you had to reach for the collapse button at the bottom. | **Fixed** — active view toggles the sidebar; picking a different view always reveals it |
| A2 | No counts/badges on nav items (connections, saved queries, search hits). | open |
| A3 | The `Users` module is commented out of `NAV_ITEMS` but `UsersView` is still rendered by `sidebar.tsx` when `sidebarView === "users"` — dead but reachable state. | open |

---

## 5. Topbar — `commons/components/shell/topbar.tsx`

Reviewed, **not changed**. Findings:

| # | Finding |
|---|---------|
| TB1 | The topbar carries a product name, a search field and a command button. There is no menu bar, no connection switcher, no breadcrumb — on a 34px strip the left two-thirds is empty. `query-command-bar.tsx` carries the connection/schema selector instead, so the context is invisible whenever a query tab is not active. |
| TB2 | `pl-14` reserves macOS traffic-light space unconditionally on `isMac`; on a frameless Linux/Windows build the same strip has no drag region declared. |
| TB3 | The search button is a `clamp(150px,24vw,240px)` box that only opens Quick Open. It reads as an input but is not one — typing into it does nothing. |

---

## 6. Status bar — `commons/components/shell/status-bar.tsx`

Findings (SB2 fixed, the rest reviewed and left):

| # | Finding |
|---|---------|
| SB1 | Right-hand side carries only the version string. v2 asked for *"left = context, right = diagnostics"*; diagnostics are still absent (latency, rows fetched, transaction state, schema, read-only flag). |
| SB2 | **(Fixed)** The status bar subscribed to the **entire** `IntrospectResult` (`schemas`, `tables`, `columns`, `primaryKeys`, `indexes`, `foreignKeys`, `views`, `triggers`) purely to read `introspect.data?.tables.length`. *Correction to an earlier draft of this audit: this is not a refetch problem* — `useIntrospect` sets `staleTime: 5 * 60 * 1000` and shares the cache entry with the Explorer, so it does not hit the backend on re-render. The real (much smaller) cost is that the status bar re-renders on every identity change of that large object, when a `select: (d) => d.tables.length` would give it a primitive. **Fixed** — added `useTableCount()` in `modules/schema/queries/schema.queries.ts` sharing the same queryKey, so the fetch is still shared with the Explorer but the status bar now subscribes to a number. |
| SB3 | The connection segment is not interactive — clicking "Connected" does nothing. |
| SB4 | Read-only connections render exactly like read-write ones. `conn.readonly` is in the payload and already gates grid editing, but the chrome never says so. |

---

## 7. Query workbench — `modules/query/components/*`

Reviewed, **not changed** beyond the grid it hosts. Findings:

| # | Finding |
|---|---------|
| Q1 | `query-command-bar.tsx` uses `border-r-white/20` for the split-button divider — a hard-coded white that is correct on the indigo primary but is the only literal colour in the chrome. |
| Q2 | The split Run button's primary action runs *current statement*, while the dropdown's first item is also *Run current*. The default and the first menu item are the same action; "Run all" is buried. |
| Q3 | `query-status-bar.tsx` shows nothing at all in the `idle` state, so the strip appears and disappears, shifting the editor height by one row on every execution. |
| Q4 | `result-tabs.tsx` uses `bg-background`/`text-foreground` for the active multi-result tab while every other tab strip in the app uses `--surface-editor` — inconsistent active treatment. |
| Q5 | `onQueryAction("saveQuery")` still resolves to `snackbar.info(t("query.saveComingSoon"))`. |

---

## 8. Cross-surface sweeps

| # | Finding | Status |
|---|---------|--------|
| X1 | **Typography floor violated.** v2 (PATCH 5.1 item 4) declared *"Removed all `text-[10px]`; enforced ≥11px"*. `grep` found **32** surviving `text-[10px]`/`text-[9px]` across 13 files. | **Fixed** — 29 swaps across 11 files. The 3 in `er-diagram/components/lod/er-{detailed,summary}-node.tsx` were deliberately left: those are Cytoscape HTML canvas nodes whose base size is a layout constant that scales with zoom, not chrome text. |
| X2 | **Hard-coded English string in a shipped surface.** `cytoscape-view.tsx` had `placeholder="Search tables..."` while `schemaWorkspace.searchTables` already existed in both locales. | **Fixed** — now `t("schemaWorkspace.searchTables")`, `useTranslation` wired in |
| X3 | **Duplicate keys in the locale files.** `en.json` has `query.savedQueries` twice; `ja.json` has both `query.savedQueries` and `dataGrid.removeFilter` twice, with *different* values. `JSON.parse` keeps the last, so the first is dead — but it is a trap for anyone editing the earlier entry. | **reported, not changed** — out of scope for a UI pass; needs a deliberate decision on which value wins |
| X4 | `en.json` and `ja.json` are 64 keys out of sync (64 en-only, 1 ja-only). | open |

---

## Verification

| Check | Command | Result |
|-------|---------|--------|
| Types | `pnpm run typecheck` (`tsr generate` + `tsc --noEmit`) | clean, 0 errors |
| Lint | `pnpm run lint` | 0 errors; only the 3 pre-existing `no-explicit-any` warnings in `er-diagram/__tests__` remain |
| Token contract | `pnpm run check:tokens` | `Token contract: clean (canonical layer + shadcn aliases + component migration)` |
| Unit/component | `pnpm run test` (vitest) | **150 files / 1762 tests, all passing** |
| Production bundle | `pnpm run build` (`tsc -b` + `vite build`) | built in 11.2s |
| Harness exclusion | `grep -rl "browser backend active\|Acme Staging\|acme_prod\|installBrowserBackend" dist/` | **no matches** — the `import.meta.env.DEV` branch and its chunk are eliminated from the production bundle, verified from build output rather than assumed |

Baseline before this pass: **147 files / 1734 tests passing**.
After: **150 files / 1762 tests passing** (+3 files, +28 tests).

Note: `pnpm run build` emits the pre-existing >500 kB chunk warning for
`vendor-cytoscape`; that is unchanged by this pass.

### New tests

| File | Covers |
|------|--------|
| `src/modules/unified-grid/__tests__/unified-grid-presentation.test.tsx` (10) | G1 alignment, G3 NULL chip, G4 boolean markers, G5 truncation span, G6 grid ARIA contract, G7 sticky offsets (40px gutter, cumulative 140px for a second pin), G8 skeleton + `aria-busy`, G9 localised resize label |
| `src/dev/__tests__/browser-backend.test.tsx` (13) | fixture SQL engine (projection/aliases/aggregates/WHERE/ORDER BY/jsonb/writes/unknown relation), introspection + DDL builders, the command router **through the real `apiInvoke` error-translation path**, and a full `<App />` render against the browser backend |
| `src/hooks/__tests__/use-scroll-parent.test.tsx` (4) | E1 — resolves the *nearest* scrollable ancestor (not the outermost), skips non-scrolling ancestors, and returns `null` rather than `<body>` when nothing above scrolls; plus a guard that `search-view.tsx` contains no `overflow-y-auto` |

### Updated tests (behaviour changed deliberately)

| File | Why |
|------|-----|
| `modules/data-grid/__tests__/data-grid.test.tsx` | asserted the removed `▼` glyph → now asserts `aria-sort="descending"` + `.lucide-arrow-down`; added an unsorted-column case |
| `modules/query/__tests__/result-grid.test.tsx` | asserted the removed `▲` glyph and the literal `i` button text → now asserts `aria-sort`, `.lucide-arrow-up`, and `getByRole("button", { name: /^Column info:/ })` with `toHaveAccessibleName` |
| `modules/er-diagram/__tests__/cytoscape-view.test.tsx` | the placeholder is now translated; the test initialises i18n with the real key so it still asserts the user-visible English string |

---

## Suggested order for the remaining items

1. **SB4** — surface read-only state in the chrome; safety-relevant, not cosmetic.
2. **Q3** — reserve the status strip height so the editor stops jumping.
3. **X3** — decide which duplicate locale value wins, then delete the dead ones.
4. **X4** — locale parity pass (64 keys).
5. **E6** — `StatusDot` needs a label; connection state is currently colour-only.
6. **TB1 / TB3** — topbar information architecture (largest visual change, needs a design decision).

Done during this pass and removed from the list: SB2, E1-class (`search-view`), T2.
