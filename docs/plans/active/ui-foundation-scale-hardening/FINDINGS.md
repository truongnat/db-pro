# P3 — UI Foundation & Scale Hardening — Findings

## F1: Dual token vocabulary (P1 Foundation)

**Evidence:** `globals.css` lines 78-166 define shadcn tokens with raw color values. Lines 168-281 define `--app-*` tokens with the same conceptual roles but different names and independent values.

**Example of duplication:**
```css
/* shadcn */
--background: #ffffff;        /* light */
--background: #0b0f17;        /* dark */

/* app */
--app-surface-0: #f0f2f5;    /* light — deepest background */
--app-surface-3: #ffffff;    /* light — editor surface */
--app-surface-0: #090d14;    /* dark */
--app-surface-3: #10151d;    /* dark */
```

These are not aligned. `--background` maps to neither `--app-surface-0` nor `--app-surface-3` cleanly — it's a third definition.

**Impact:** Every `npx shadcn add` can reintroduce shadcn-native assumptions. Components using `bg-background` and components using `var(--app-surface-3)` will diverge visually when theme values change.

**Severity:** P1 — foundation risk. Not a visible bug today, but becomes worse with every new component.

**Resolution (2026-08-12, P3.1 + P3.2):** single canonical vocabulary landed — `--surface-*` / `--text-*` / `--border-*` / `--accent-*` / `--state-*` / `--elevation-*` replace all 21 `--app-*` color tokens (604 occurrences across 96 files → 0); the shadcn compatibility layer is now alias-only; `--app-*` is reserved for layout metrics. `npm run check:tokens` (CI) enforces theme completeness, value snapshots, alias-only shadcn layer, and bans `--app-*` color tokens / raw shadcn vars in components. Visual regression: 96/96 resolved token pairs identical (bench/token-contract.html, light + dark).

## F2: O(T x C) column indexing in ER diagram (P1 Performance)

**Evidence:** `er-diagram.tsx` lines 91-98:
```ts
const cols = data.columns.filter(
  (c) => c.tableName === table.name && c.schema === table.schema,
);
const pkCols = new Set(
  data.primaryKeys
    .filter((pk) => pk.tableName === table.name && pk.schema === table.schema)
    .flatMap((pk) => pk.columns),
);
```

This runs inside `tables.map(...)`, so for T tables and C total columns:
- Column filter: T x C comparisons
- PK filter: T x P comparisons (P = total PK entries)

With T=500, C=8000: ~4,000,000 comparisons per render.

**Severity:** P1 — causes measurable lag at 500 tables.

## F3: Duplicate Dagre layout computation (P1 Performance)

**Evidence:** `er-diagram.tsx`:
- Line 149: `const autoLaid = layoutGraph(initialNodes, initialEdges, ...)` inside `useMemo`
- Line 163: `const relaid = layoutGraph(initialNodes, initialEdges, ...)` inside `useEffect`

Both execute when `initialNodes`, `initialEdges`, or `layoutDirection` change. The `useEffect` version additionally handles `selectedEdgeId` and `manualPositions`, but the layout computation itself is identical.

Dagre layout for 500 nodes + 700 edges takes ~50-200ms on main thread. Running it twice doubles this cost.

**Severity:** P1 — directly contributes to UI freeze on layout changes.

## F4: No level-of-detail rendering (P1 Performance/UX)

**Evidence:** `table-node.tsx` renders all columns for every visible table node regardless of zoom level. The `compact` prop is a manual toggle, not zoom-aware.

With 500 tables x 16 columns average = 8,000 column row DOM elements, plus handles, icons, and text nodes. Total DOM nodes for the diagram easily exceeds 50,000.

**Severity:** P1 — React reconciliation of 50k+ DOM nodes causes frame drops during pan/zoom.

## F5: MiniMap always enabled (P2 Performance)

**Evidence:** `er-diagram.tsx` line 349: `<MiniMap ... />` is always rendered. MiniMap re-renders on every viewport change and draws all nodes.

For 500+ node graphs, MiniMap adds measurable overhead.

**Severity:** P2 — contributes to frame drops but not the primary bottleneck.

## F6: Edge highlighting triggers full re-layout (P1 Performance)

**Evidence:** `er-diagram.tsx` `useEffect` at line 162 has `selectedEdgeId` in its dependency array, along with `initialNodes`, `initialEdges`, `layoutDirection`, `manualPositions`. When `selectedEdgeId` changes, the entire effect re-runs including `layoutGraph()`.

Clicking an edge should only change edge styles, not recompute the positions of 500 nodes.

**Severity:** P1 — every edge click triggers unnecessary Dagre computation.

## Audit findings (P3.8 — Data Grid / Metadata List audit, 2026-08-12)

Scope: every list view that iterates schema metadata (schema explorer, data grid / result grid, connection list, search view, saved queries tree, column/index lists, quick-open index). Pattern hunted: the P3.3 smell — `.filter()` / `.find()` / `.includes()` over a full metadata array inside a per-item loop (O(N²)).

**Verdict: no P0/P1 O(N²) patterns remain.** All P3.3-style scans were already gone:

- **Schema explorer** (`explorer-view.tsx`) — tables/views grouped via `tablesBySchema` / `viewsBySchema` Maps built once (O(T + V)); no `data.columns.filter(...)` per table (the tree shows tables/views only).
- **Data grid** (`unified-grid.tsx`) — row-virtualized (`useVirtualizer`); only visible rows render; per-row work is O(1) (`selection.has(rowIdx)`); column filters are O(C) once per render.
- **Connection list** (`connection-list.tsx`) — single-pass `filter` + sort over connections (small N); tag filter is O(tags).
- **Search view / saved-queries-tree / quick-open index** — O(T) filter per query change, memoized; quick-open pre-indexes `connNameMap`.
- **Column / index lists** (`column-list.tsx`, `index-list.tsx`) — single object's arrays (O(C) per object).
- **Schema catalog store** — `columnsByTable` pre-indexed (P3.3 pattern already applied on the query side).

### F-P3.8-1: Explorer re-resolved driver/dialect per table row (P2, FIXED)

**Evidence:** `explorer-view.tsx` `tables.map(...)` called `getDriverForConnection(conn.id)` — a `connections.find(...)` over the connections array — plus `getSqlDialect(driver)` and `generateCountSQL(...)` **per table row** on every render of an open tables group. With T=500 tables × N connections: 500 linear scans + 500 dialect/string constructions per render, none of which depend on the table (the count SQL string does; the driver/dialect do not).

**Severity:** P2 — measurable but small (connections count is tiny); wasteful per-item work in the exact shape this audit targets.

**Resolution (2026-08-12):** driver + dialect hoisted to once per connection row (`connDriver` / `connDialect`); per-table code keeps only `generateCountSQL` (table-dependent). Typecheck + 1,421 FE tests green.

### F-P3.8-2: Cell-editor column lookup is a linear scan (P2, TRACKED)

**Evidence:** `data-grid.tsx` `renderCellEditor` resolves the edited column's type via `columns.find((c) => c.name === colName)`. Runs once per editor open (not per cell render), so O(C) per edit — negligible today.

**Severity:** P2 — polish. Fix if the grid ever renders many simultaneous editors (e.g., bulk edit UX): pre-index `ColumnMeta` by name once per render.

**Tracked:** keep in the P2 backlog; not worth a code change now.

### Merge-review P2s (tracked, 2026-08-12 — final pre-merge review of the full branch)

- **F-MR-1:** fit-view triggers via a synthetic `keydown "1"` dispatch on `.react-flow` (P1.7) — depends on React Flow's default fit-view shortcut; if defaults change, auto-fit silently breaks (the manual fit button remains). **RESOLVED (2026-08-12):** the mounted `ReactFlowInstance` is now captured in `onInit` (`rfInstanceRef`); the fit-on-commit effect, the initial fit and the manual Fit button all call `instance.fitView({ padding: 0.2 })` — no more synthetic keydown.
- **F-MR-2:** `useWorkerLayout`'s module-level worker singleton — a one-time worker-creation failure permanently downgrades the session to the synchronous fallback runner (sticky catch). **RESOLVED (2026-08-12):** runner selection extracted to a pure `resolveLayoutRunner(current, create, fallback)` — a failed `create()` returns `{ sticky: false }`, so the fallback is used for that attempt only and the next run retries worker creation (a transient failure no longer downgrades the session; a later success is cached). Covered by `__tests__/use-worker-layout.test.ts` (4 tests: cached runner, non-sticky fallback, recovery, sticky semantics).
- **F-MR-3:** `format:check` drift on `frontend/scripts/generate-fixtures.mjs` was pre-existing on main (CI red there too) — fixed on this branch to unblock the merge gate; the root cause (main CI being red) is tracked so the drift does not silently recur. **RESOLVED (2026-08-12):** root cause was the CI gate scope itself — `format:check` was `prettier --check src/`, so `scripts/`, `eslint.config.js` and `public/` were never checked and drift accumulated silently. The gate now runs `prettier --check .` over the whole frontend (`.prettierignore` covers `dist`/`node_modules`/`routeTree.gen.ts`/generated `test/fixtures`), and the pre-existing drift on the tracked files (`eslint.config.js`, `public/splashscreen.html`) was formatted. Verified: injecting drift into `scripts/generate-fixtures.mjs` now fails the gate.
- **F-MR-4:** the token migration did not mechanically verify string/comment drift (the perl rename + `check-token-drift` guard cover CSS / `var()` usage, and the 96/96 visual regression covers resolved values) — stray token names inside comments are harmless; optionally scan comments in the guard later. **RESOLVED (2026-08-12):** `check-token-drift.mjs` check 6 — every `--<token>` mention anywhere in src / public / index.html (comments, strings, className arbitrary values) must be defined in globals.css (Radix `--radix-*`, Tailwind spacing and a jsdom CLI flag are allow-listed). The scan immediately caught 2 real leftovers the migration missed: `--text-primary-primary` (rename artifact in `er-perf-hud.tsx`, undefined token → browser fallback) and a vestigial `--app-editor-bg,var(--surface-editor)` fallback chain in `query-editor.tsx` — both fixed to `var(--text-primary)` / `var(--surface-editor)`. Verified: injecting a stale token name into a comment fails the guard.
- **Fixed during review:** `tablesInSchema` was unmemoized in `er-diagram.tsx` (recreated every render, defeating `schemaStats` O(F+C) and `suggestedPoints` O(T log T) memoization) — wrapped in `useMemo([data.tables, schema])`.

## PR#12 external review round (2026-08-12) — 3 P1 + 2 P2, all RESOLVED

External review of the merged branch (PR #12 at `db781ef`) verified every finding against the real code. All closed with shipped code + runtime evidence (VERIFICATION.md "P1 review fixes").

### F-R1: Cold first-open waited on dagre (P1, RESOLVED)

**Evidence:** `cytoscape-view.tsx` mounted only when `layoutStatus === "ready"`, and the canvas had no fallback before dagre finished (8,110 ms @500 / 122,084 ms @1000 cold worker layout). The old acceptance line "Open workspace → UI usable < 1s" measured shell interactivity only — the *diagram* itself stayed blank for seconds/minutes.

**Resolution:** fast **approximate layout** (`utils/approximate-layout.ts`, degree-ordered placement, O(T+E), ~1 ms @1000) paints immediately; when dagre finishes in the worker the renderer upgrades positions in place via the new `ErRenderer.updatePositions` (no re-mount). View mounts on first positions (approx or dagre), not on `layoutStatus === "ready"`. Budget split into **TTI shell < 1s** + **TTD (time-to-diagram) < 2s @500 / 3–5s @1000**; CDP evidence: **230 ms @500 / 354 ms @1000**. Dagre upgrade is now an enhancement, not a prerequisite.

### F-R2: Layout geometry used the wrong renderer profile (P1, RESOLVED)

**Evidence:** the layout input passed ReactFlow card geometry (220 × 32+cols·20+8) while `CytoscapeErRenderer` paints 160 × 28. Dagre (and the schemaHash cache) computed against geometry the canvas never uses → stretched overview, long edges, zoomed-out fit.

**Resolution:** `utils/layout-profile.ts` — `OVERVIEW_PROFILE` (160×28, no column rows) vs `REACT_FLOW_PROFILE` (220×dynamic) with shared `LAYOUT_PROFILE_*` constants; overview layout input is built from the graph model, detail from column-aware data; `computeLayoutHash` now includes the profile (separate cache namespaces). Covered by `layout-profile.test.ts`.

### F-R3: Worker failure could push 122 s dagre back onto the main thread (P1, RESOLVED)

**Evidence:** `useWorkerLayout`'s catch path called `getFallbackRunner()` → `createFallbackLayoutRunner()` → synchronous `computeLayoutPositions` — an 8 s / 122 s main-thread freeze if the worker ever failed to load. Code review found two additional gaps in the first fix attempt: (1) `createWorkerLayoutRunner` silently returned the sync dagre fallback when `Worker` was undefined — a large graph could run dagre on the main thread with the size gate completely bypassed; (2) the worker-**creation**-failure path committed with `degraded: false` — the approximate positions were written to the schemaHash cache (poisoning repeat opens) and no degraded notice was shown.

**Resolution (three-layer hard gate):** (1) `createWorkerLayoutRunner` **throws** when `Worker` is unavailable, routing through `resolveLayoutRunner` to the size-gated fallback; (2) every runner is **kind-tagged** (`worker` / `dagre-sync` / `approximate`) and the degraded flag is derived from the runner that actually produced the result — creation-failure and runtime-failure paths alike; (3) node count > `SYNC_DAGRE_MAX_NODES` (large graphs) is hard-forbidden from the sync dagre fallback — a failed worker selects `createApproximateLayoutRunner` (main-thread-safe, ~1 ms) and surfaces a `degraded` flag to the view; the approximate result is **not** written to the schemaHash cache (a degraded layout must never poison repeat opens). Small graphs keep the sync dagre fallback (fast and safe there). Covered by `use-worker-layout.test.ts` (kind tags, throw-without-Worker, size gate, determinism).

### F-R4: Canvas colors hardcoded for the dark theme (P2, RESOLVED)

**Evidence:** `cytoscape-renderer.ts` hardcoded `#1e293b` / `#7dd3fc` / … — broke the light theme and the token contract (F1).

**Resolution:** `ErThemeTokens` + `ErRenderer.updateTheme()`; the renderer resolves tokens from canonical CSS custom properties via `getComputedStyle` (`--surface-panel`, `--border-default`, `--text-primary`, `--accent`, …) with fallbacks; `cytoscape-view.tsx` resolves at mount and re-applies on theme-store change — switching themes does not destroy the graph. Verified in `cytoscape-renderer.test.ts` + the CDP harness (`updateTheme` swaps colors, graph stays alive).

### F-R5: No runtime test on the real cytoscape path (P2, RESOLVED)

**Evidence:** `CytoscapeErRenderer` had no automated coverage (jsdom lacks a canvas 2D context), and the P1.9 harness was served through vite-dev then removed.

**Resolution:** (a) `__tests__/cytoscape-renderer.test.ts` — real renderer + real cytoscape in headless mode (no canvas needed): mount, positions, theme, selection, updatePositions; (b) `bench/er-renderer-runtime.html` + `bench/build-er-renderer-runtime.mjs` — esbuild bundle of the **real app renderer + approximate layout** (cytoscape included) run in real Chrome via the CDP runner: 6/6 checks PASS at 500 and 1000 tables, 0 console errors, TTD 230/354 ms. Screenshots `bench/er-runtime-{500,1000}.png`.

## F7: Large-schema ER rendering architecture does not scale (P1)

**Evidence:** At 500 tables the diagram feeds ~500 `TableNode` components (up to 7,500 column rows) into React reconciliation simultaneously. The current LOD tiers (P3.5) switch content inside one component via conditional rendering — the render tree is a single path, not a per-LOD component switch — and React Flow's `onlyRenderVisibleElements` is not enabled, so off-viewport nodes still mount. Every edge keeps labels, markers, and smoothstep paths at every zoom. Dagre layout runs on the main thread with no cache.

**Failure scenario:** Open a 500-table schema → initial layout freeze on main thread (~50–200ms+ uncached), then frame drops during pan/zoom because thousands of DOM elements and SVG paths re-render.

**Severity:** P1 — release prerequisite; the product is a database IDE that must handle production schemas.

**Decision (locked 2026-08-12):** layered architecture `ErGraphModel → Layout Engine (Worker + cache) → Spatial Index → Viewport Engine → ReactFlowErRenderer / future CanvasErRenderer` behind an `ErRenderer` interface. `onlyRenderVisibleElements` is a P1 mitigation only, not the end state. True LOD = render-tree switch (ErDot/ErCompact/ErSummary/ErDetailed), edges get their own LOD tiers, layout moves to a Worker with atomic commit (no streamed unstable positions), thresholds become a complexity score, and full-schema is not the default UX for large graphs. Implementation order P1.1→P1.9; Canvas rewrite only after P1.8 benchmarks prove React Flow misses budget.

**Scope:** smallest coherent fix — this session delivers P1.1 (runtime instrumentation) + P1.2 (`onlyRenderVisibleElements` + MiniMap policy). Remaining P1.3–P1.9 tracked in CHECKLIST Phase 6.

**Provider impact:** frontend-only; PostgreSQL/SQLite introspection unchanged.
