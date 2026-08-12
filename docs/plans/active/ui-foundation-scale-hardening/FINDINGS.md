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

## Audit findings

_To be populated during Phase 5 audit._

## F7: Large-schema ER rendering architecture does not scale (P1)

**Evidence:** At 500 tables the diagram feeds ~500 `TableNode` components (up to 7,500 column rows) into React reconciliation simultaneously. The current LOD tiers (P3.5) switch content inside one component via conditional rendering — the render tree is a single path, not a per-LOD component switch — and React Flow's `onlyRenderVisibleElements` is not enabled, so off-viewport nodes still mount. Every edge keeps labels, markers, and smoothstep paths at every zoom. Dagre layout runs on the main thread with no cache.

**Failure scenario:** Open a 500-table schema → initial layout freeze on main thread (~50–200ms+ uncached), then frame drops during pan/zoom because thousands of DOM elements and SVG paths re-render.

**Severity:** P1 — release prerequisite; the product is a database IDE that must handle production schemas.

**Decision (locked 2026-08-12):** layered architecture `ErGraphModel → Layout Engine (Worker + cache) → Spatial Index → Viewport Engine → ReactFlowErRenderer / future CanvasErRenderer` behind an `ErRenderer` interface. `onlyRenderVisibleElements` is a P1 mitigation only, not the end state. True LOD = render-tree switch (ErDot/ErCompact/ErSummary/ErDetailed), edges get their own LOD tiers, layout moves to a Worker with atomic commit (no streamed unstable positions), thresholds become a complexity score, and full-schema is not the default UX for large graphs. Implementation order P1.1→P1.9; Canvas rewrite only after P1.8 benchmarks prove React Flow misses budget.

**Scope:** smallest coherent fix — this session delivers P1.1 (runtime instrumentation) + P1.2 (`onlyRenderVisibleElements` + MiniMap policy). Remaining P1.3–P1.9 tracked in CHECKLIST Phase 6.

**Provider impact:** frontend-only; PostgreSQL/SQLite introspection unchanged.
