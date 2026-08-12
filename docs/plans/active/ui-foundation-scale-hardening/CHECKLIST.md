# P3 — UI Foundation & Scale Hardening — Checklist

## Phase 1: Algorithm Fixes

- [x] **1.1** Pre-index `data.columns` by `schema.tableName` in `er-diagram.tsx`
- [x] **1.2** Pre-index `data.primaryKeys` by `schema.tableName`
- [x] **1.3** Pre-index `data.foreignKeys` by `schema.tableName` (from/to)
- [x] **1.4** Replace per-table `.filter()` calls with map lookups
- [x] **1.5** Eliminate duplicate `layoutGraph()` call — remove from `useEffect`, keep only in `useMemo`
- [x] **1.6** Separate edge highlighting logic from layout re-computation
- [x] **1.7** Run existing ER diagram tests — all pass
- [x] **1.8** Measure layout + build time before/after with 500-table fixture — DONE via P1.7/P1.8: sync dagre 8.1s main-thread block before → worker-side + cache after (HUD reads worker duration); full 100/500/1000 matrix in VERIFICATION.md (layout 0.15/8/122s pre-worker)

## Phase 2: Design Token Contract

- [x] **2.1** Define canonical token naming convention — locked `--surface-*` / `--text-*` / `--border-*` / `--accent-*` / `--state-*` + `--elevation-*` (shadows); `--app-*` prefix reserved for layout metrics only; canonical layer in `globals.css` defines each color value once per theme (light + dark)
- [x] **2.2** Migrate `--app-*` color tokens → canonical names — 21 tokens renamed across **96 files / 604 occurrences → 0** (perl migration, longest-first to avoid prefix collisions: `--app-primary-hover` before `--app-primary`, etc.); `--app-shadow-*` had zero component consumers; `--app-sidebar-*`/heights kept as layout metrics
- [x] **2.3** Make shadcn tokens (`--background`, `--popover`, etc.) alias to canonical tokens — compat layer is now alias-only; shadcn's `--accent` hover role moved to `@theme inline` (`--color-accent: var(--surface-active)`) so `bg-accent` components keep identical colors while bare `--accent` becomes the canonical brand family
- [x] **2.4** Remove duplicate color definitions from shadcn `:root` / `[data-theme="dark"]` blocks — done; `check-token-drift.mjs` rejects any raw color in the compat layer (allowed primitives only: `--radius`, `--chart-*`, `*-foreground`, `--overlay`)
- [x] **2.5** Verify no visual regression (light + dark) — `bench/token-contract.html` CDP harness: **96/96 resolved-value pairs identical** (48 tokens × both themes) between pre-migration and canonical CSS; screenshot `bench/token-contract.png`; also pinned by the drift-check value snapshot
- [x] **2.6** Document token contract and `shadcn add` safety rule — contract header in `globals.css`, rule in AGENTS.md ("shadcn add must not modify semantic token definitions"), drift guard wired into CI (`npm run check:tokens`)
- [x] **2.7** Add CI check or script to detect token drift — `scripts/check-token-drift.mjs` upgraded to 5 checks (compat alias-only, canonical theme completeness, canonical value snapshot, no `--app-*` color tokens in src **and** shipped `public/` + `index.html` + other css, no raw shadcn vars in components); added to `.github/workflows/ci.yml` frontend job + AGENTS.md gates

## Phase 3: Rendering LOD

- [x] **3.1** Add viewport zoom tracking (`useViewport()` or `onViewportChange`)
- [x] **3.2** Define zoom tier thresholds (initial: <0.3, 0.3-0.7, >0.7)
- [x] **3.3** Pass zoom tier to `TableNode` via node data
- [x] **3.4** Implement tiered rendering in `TableNode`: name-only, count, full columns
- [x] **3.5** Memoize `TableNode` properly — ensure data changes don't break memo
- [x] **3.6** Simplify edges at low zoom (thinner, no labels)
- [x] **3.7** Conditionally disable MiniMap for large graphs (>200 nodes)
- [x] **3.8** Test zoom transitions for visual smoothness

## Phase 4: Large Schema Mode

- [x] **4.1** Detect schema size and select rendering mode (full / compact / overview / large)
- [x] **4.2** Implement search-first default for schemas > 200 tables
- [x] **4.3** Implement neighborhood mode: selected table + N-hop FK neighbors
- [x] **4.4** Add "Show all N tables" explicit action
- [x] **4.5** Cache layout result — don't re-layout on search/selection changes
- [-] **4.6** Debounce resize/layout changes — not needed after layout dedup (P3.4)
- [-] **4.7** Consider Web Worker for Dagre layout — not needed with LOD + neighborhood reducing visible nodes

## Phase 5: Audit + Budgets

- [x] **5.1** Audit schema explorer for O(N) scan patterns
- [x] **5.2** Audit data grid for O(N) scan patterns
- [x] **5.3** Audit connection list for O(N) scan patterns
- [x] **5.4** Fix P0/P1 issues found in audit
- [x] **5.5** Create benchmark fixtures (20, 100, 500, 1000 tables)
- [x] **5.6** Define performance budgets in VERIFICATION.md
- [x] **5.7** Add at least one automated performance regression test

## Phase 6: Large-Schema Rendering Architecture (P1)

- [x] **6.1** P1.1 Runtime instrumentation — ErPerfMonitor (long tasks, frame sampler, DOM counts, measure marks) + dev HUD overlay
- [x] **6.2** P1.2 Enable `onlyRenderVisibleElements` on `<ReactFlow>`
- [x] **6.3** P1.2 MiniMap policy — disabled for large schemas; per-fixture documentation
- [x] **6.4** P1.3 True LOD components — `ErDotNode` / `ErCompactNode` / `ErSummaryNode` / `ErDetailedNode` switch via `ErTableNode` dispatcher, no CSS-hidden DOM; non-detail nodes carry generic handles + edges drop handle ids (edge stays anchored)
- [x] **6.5** P1.4 Edge LOD — zoom tiers: aggregate relations (merged straight edges with count label) < 0.25, simple edges (straight, no markers) 0.25–0.6, full FK edges > 0.6; handle-id stripping follows node LOD; selection cleared outside full band
- [x] **6.6** P1.5 Viewport / spatial-index layer — `SpatialIndex` (uniform-grid hash over node bounding boxes) returns visible node + edge IDs; brute-force reference kept for tests; HUD queries via the index
- [x] **6.7** P1.6 Default neighborhood exploration UX — landing mode (stats + suggested starting points by FK degree) + hop scope `[1][2][3][Domain]` + explicit `All N`; auto-fit on scope change
- [x] **6.8** P1.7 Layout in Web Worker + `schemaHash → positions` cache, atomic commit — `er-layout.worker.ts` (dagre, plain `LayoutInput` over the boundary) + `useWorkerLayout` hook (cache-first by `computeLayoutHash`, stale-discard by requestId, single atomic position commit) + `LayoutCache` (localStorage, node-set integrity, LRU eviction) + sync fallback runner; "Arranging N tables…" overlay while computing; fit-view on commit replaces the fixed-80ms timer
- [x] **6.9** P1.8 Benchmark A/B harness — full matrix 100/500/1000 DONE (VERIFICATION.md): Cytoscape 31 DOM at all scales (layout 0.2/2/36 s) vs React Flow 1,143/6,143/11,817 DOM (layout 0.15/8/122 s); overview pan 60/34/18.6 fps (B) vs 48–60 (A); detail pan 60 fps both
- [x] **6.10** P1.9 Canvas decision — hybrid LANDED: `ErRenderer` abstraction (renderer/types.ts) + `er-graph-model` (pure domain) + `CytoscapeErRenderer` (canvas, selection→neighborhood highlight) mounted for large-schema "All N tables" overview; React Flow keeps ≤threshold + neighborhood/landing modes. Benchmark-verified against the real app source (see VERIFICATION.md): 18 DOM elements + ~60 fps overview pan at 500 AND 1,000 tables (vs React Flow 6,143/11,817 DOM and 34/18.6 fps)
- [x] **6.11** Complexity score replaces hardcoded thresholds — `computeSchemaComplexity` (tables + relations×0.7 + columns×0.08) + `classifySchemaComplexity` (S/M/L/XL) in er-graph-model.ts; `LARGE_SCHEMA_THRESHOLD=200` removed from er-diagram.tsx, `isLargeSchema = tier ∈ {L, XL}` derived from `graphModel.stats`. Boundaries tuned from P1.8 evidence: A100(310.4)→M, A500(1,802.5)→L, A1000(3,765.2)→XL. 6 unit tests (formula, fixture scores, boundaries, lean-medium stays M).

## Gates

- [x] All Phase 1 items complete before merging — P1.1–P1.9 + code-split + 6.11 all `[x]` above; pre-merge review found 0 P0 / 0 P1
- [x] Phase 2 complete before adding new shadcn components — P3.1 + P3.2 landed (canonical vocabulary + shadcn alias layer + drift guard in CI); verified 96/96 token values unchanged
- [ ] Phase 3+4 complete before v0.1 release
- [ ] P0 = 0, P1 = 0 across all phases — holds for the P1 series (review-verified); Phase 2–4 not yet audited
- [x] P1 invariants hold per fixture (`graphTables ≠ renderedTables ≠ detailedTables`) — verified mechanically by `__tests__/rendering-invariant.test.ts` on the A500 fixture (SpatialIndex culling + resolveLod at overview zoom); strict `≠` is an overview-scale property (all visible nodes are detailed at zoom ≥ 0.7 by design)
