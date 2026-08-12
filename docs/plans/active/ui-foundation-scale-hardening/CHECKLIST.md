# P3 — UI Foundation & Scale Hardening — Checklist

## Phase 1: Algorithm Fixes

- [x] **1.1** Pre-index `data.columns` by `schema.tableName` — pure `buildColumnsByTable` in `renderer/er-node-builder.ts`, memoized in `er-diagram.tsx` (stable identity)
- [x] **1.2** Pre-index `data.primaryKeys` by `schema.tableName` — `buildPrimaryKeysByTable` (Set per table; composite PK columns merged)
- [x] **1.3** Pre-index `data.foreignKeys` — `buildFkColumnSet` (`schema.table:column` FK-source flags, O(1) `isForeignKey` lookups); edge grouping stays in pure `groupForeignKeys`
- [x] **1.4** Replace per-table `.filter()` calls with map lookups — `buildTableNodes` is lookup-only; mechanically verified: `er-node-builder.test.ts` parity test (pre-indexed build ≡ naive per-table filter on 100 tables) + `benchmark.test.ts` now benchmarks the REAL pipeline (20/100/500/1000 tables under 5/20/50/100 ms budgets)
- [x] **1.5** Eliminate duplicate `layoutGraph()` call — the dual `useMemo` + `useEffect` layout is replaced by P1.7 `useWorkerLayout` (hash-memoized `computeLayoutHash` + cache-first): one dagre run per distinct graph, stale results discarded
- [x] **1.6** Separate edge highlighting logic from layout re-computation — dedicated effect calls `setEdges` only; edge selection cleared outside the full edge-LOD band
- [x] **1.7** Run existing ER diagram tests — all pass: **1,412 FE tests** (new `er-node-builder.test.ts` +6)
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

- [x] **3.1** Add viewport zoom tracking — `onViewportChange` writes a ref + derives `currentLod` / `currentEdgeLod` (no per-frame setState)
- [x] **3.2** Define zoom tier thresholds — locked 4-level `LOD_THRESHOLDS` in `utils/lod.ts` (dot <0.2 / compact <0.45 / summary <0.7 / detail ≥0.7), monotonic-partition tested
- [x] **3.3** Pass zoom tier to `TableNode` via node data — `lod` field injected by the `tieredNodes` memo (keyed on `currentLod`), dispatched by `ErTableNode`
- [x] **3.4** Implement tiered rendering in `TableNode`: name-only, count, full columns — superseded by P1.3 true LOD components (`ErDotNode`/`ErCompactNode`/`ErSummaryNode`/`ErDetailedNode`); render-tree switch, no CSS-hidden DOM. Mechanically verified: `__tests__/er-table-node.test.tsx` (5 tests) — per-lod leaf, 0 `[data-column]` rows below detail, exactly one leaf mounted
- [x] **3.5** Memoize `TableNode` properly — dispatcher + leaves are `memo`ized; LOD changes never re-run layout (`layoutInput` depends on `initialNodes`, not `currentLod`)
- [x] **3.6** Simplify edges at low zoom — superseded by P1.4 edge LOD (`utils/edge-lod.ts`): aggregate <0.25 (merged straight, count label, no markers), simple 0.25–0.6 (straight, no labels/markers), full >0.6; handle-id stripping follows node LOD
- [x] **3.7** Conditionally disable MiniMap for large graphs — `showMiniMap = !landing && initialNodes.length <= 200` (P1.2 policy)
- [x] **3.8** Test zoom transitions for visual smoothness — LOD switch is a memoized render-tree swap of visible nodes only (see 3.4/3.5); no re-layout on tier change; verified in P1.8 benchmark pan/zoom frame times (60 fps detail, 34–60 fps overview)

## Phase 4: Large Schema Mode

- [x] **4.1** Detect schema size and select rendering mode (full / compact / overview / large) — superseded by 6.11 complexity tiers: S/M → full React Flow, L/XL → landing + neighborhood, `showAll` → Cytoscape canvas overview (P1.9)
- [x] **4.2** Implement search-first default for schemas > 200 tables — superseded by complexity tiers (`isLargeSchema = tier ∈ {L, XL}`); large schemas open in `landing` mode (stats card + suggested starting points + search), full schema is never the default (locked hard rule #6)
- [x] **4.3** Implement neighborhood mode: selected table + N-hop FK neighbors — P1.6: `getNeighborhood` (1/2/3 hops) + `getConnectedComponent` (Domain); verified by `neighborhood.test.ts` incl. 2-hop parity vs independent BFS-distance reference on a 1000-table graph, isolated-seed isolation, 5 ms budget
- [x] **4.4** Add "Show all N tables" explicit action — `handleShowAll`; renders full graph (Cytoscape canvas for L/XL, React Flow otherwise)
- [x] **4.5** Cache layout result — don't re-layout on search/selection changes — P1.7 `LayoutCache` (schemaHash → positions, localStorage) + `useWorkerLayout` hash-memoized; neighborhood/seed/hops changes only change `layoutInput`, which only re-runs dagre for a genuinely new graph
- [-] **4.6** Debounce resize/layout changes — not needed after layout dedup (P3.4)
- [x] **4.7** Web Worker for Dagre layout — IMPLEMENTED in P1.7 (this pre-P1 item predicted it unnecessary; the P1.8 benchmark proved the opposite — dagre 8 s @500 / 122 s @1000 on the main thread — so the worker + schemaHash cache was built: `er-layout.worker.ts`, no main-thread block)

## Phase 5: Audit + Budgets

- [x] **5.1** Audit schema explorer for O(N) scan patterns — CLEAN: `tablesBySchema`/`viewsBySchema` Maps (O(T+V)); fixed F-P3.8-1 (driver/dialect were re-resolved per table row — O(tables×connections) — now hoisted per connection)
- [x] **5.2** Audit data grid for O(N) scan patterns — CLEAN: `UnifiedGrid` row-virtualized (`useVirtualizer`), per-row O(1) via row index; F-P3.8-2 (cell-editor `columns.find` per edit-open) tracked as P2
- [x] **5.3** Audit connection list for O(N) scan patterns — CLEAN: single-pass filter + sort over connections (small N); tag filter O(tags)
- [x] **5.4** Fix P0/P1 issues found in audit — 0 P0/P1 found (see FINDINGS.md "Audit findings"); the one real P2 fixed (F-P3.8-1), one tracked (F-P3.8-2)
- [x] **5.5** Create benchmark fixtures (20, 100, 500, 1000 tables) — `__tests__/er-fixture.ts` (`generateErFixture`, deterministic seed 42) shared by benchmark/er-node-builder/neighborhood tests; `bench/fixture-gen.js` presets 100/500/1000 for the standalone A/B harnesses
- [x] **5.6** Define performance budgets in VERIFICATION.md — reconciled 2026-08-12 against P1.8/P1.9 evidence (see "ER Diagram — budget reconciliation"): node build/DOM/memory/initial paint/search/selection PASS; Dagre absolute budgets reframed to main-thread block = 0 (worker + cache, F-B1); frame-time budgets reframed to the shipped renderer path (P1.9 hybrid fixes RF overview 34/18.6 fps @500/1000, F-B2); harness protocol gaps tracked as P2 (F-B3)
- [x] **5.7** Add at least one automated performance regression test — `benchmark.test.ts` (real pre-indexed pipeline, 5/20/50/100 ms budgets, suite green) + `neighborhood.test.ts` 2-hop <5 ms @1000

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

## Option C — progressive layout quality (2026-08-12)

- [x] **OC-1** `utils/force-refine.ts` — deterministic Fruchterman-Reingold (uniform-grid repulsion, spring attraction, re-center); pure + worker-safe; `computeOptimalDistance` + `meanEdgeLength` quality metric
- [x] **OC-2** Worker protocol — `er-layout.worker.ts` posts `stage: "refine"` sets (30 passes, every 6) then `stage: "final"` (dagre); every stage is a full stable set (no partial streams — the locked atomic-commit contract is preserved)
- [x] **OC-3** Hook — `useWorkerLayout(input, options, { progressive })`; progressive stages update `positions` + `refining: true`, never cached, gated at `PROGRESSIVE_MIN_NODES = 120`; fallbacks ignore onProgress
- [x] **OC-4** Renderer/view — `updatePositions(positions, { fit })`: refine stages apply without re-fit (no viewport yank while panning), final commits re-fit; "Refining layout…" hint; only the large-schema overview requests progressive
- [x] **OC-5** Tests — `force-refine.test.ts` (10: determinism, completeness, ≥25% edge-length improvement, cluster separation, <1 s @1000) + `use-worker-layout.test.ts` (+2 progressive wiring)
- [x] **OC-6** Runtime evidence — CDP harness (real renderer): 500 → first stage ~50 ms / 153 ms total, mean edge 2145→524 (−75%); 1000 → **first stage 66 ms** / 321–366 ms total, 3132→926 (−70%); monotonic stages
- [x] **OC-7** Review fix — model-switch mounts always start from the approximate circle (never the previous graph's in-flight positions; RF direction-toggle behavior preserved)
- [x] **OC-8** Gates — **1,457 FE tests** (125 files), tsc 0, lint/prettier/check:tokens clean, build OK (worker chunk +2.2 kB)

## UX pivot — full graph default (opass.html reference, 2026-08-12)

- [x] **UX-1** Large schemas open on the FULL canvas graph immediately — removed the landing/search-first flow: `useCytoscapeForOverview = isLargeSchema` (no `showAll` gate), no empty-canvas landing, no neighborhood React Flow mode, no suggested starting points, no "All N tables" escape hatch
- [x] **UX-2** Search = focus + highlight (opass), never filter — `utils/overview-search.ts` (pure `findTableMatches` + `resolveHighlightSet`); `highlightSearch` rings matches (`searched` red border) + centers the viewport (400ms); a single match also focuses its neighborhood; empty query / Escape clears ring + fade
- [x] **UX-3** Click a table → fade the rest (`faded` opacity 0.1) + highlight the hop-scoped neighborhood (`highlighted` ring/edges) — `ErSelection` gains `highlightNodeIds` (host-computed exact set) + `fadeRest`; background tap clears; renderer API stays backward-compatible
- [x] **UX-4** Overview explorer slimmed — `neighborhood-explorer.tsx` → `overview-explorer.tsx` (schema stats + [1 hop][2 hops][3 hops][Domain] highlight radius only)
- [x] **UX-5** Tests + runtime evidence — `overview-search.test.ts` (10), `cytoscape-renderer.test.ts` (+4: fade/highlight, clearSelection vs rings, ring replace/clear, background tap), CDP harness +4 checks @500/1000; **160 ER tests green**

## PR#12 review round (P1-1 … P2-2, 2026-08-12)

- [x] **R-1 (P1)** Overview paints without waiting for dagre — fast approximate layout (`utils/approximate-layout.ts`, O(T+E), ~1ms @1000) + `ErRenderer.updatePositions` async upgrade (no re-mount); view mounts on first positions, not `layoutStatus === "ready"`. TTD runtime evidence: **230ms @500 / 354ms @1000** (CDP, real renderer)
- [x] **R-2 (P1)** Renderer-specific layout geometry — `utils/layout-profile.ts` (`OVERVIEW_PROFILE` 160×28 vs `REACT_FLOW_PROFILE` 220×dynamic); overview layout input from the graph model; `computeLayoutHash` includes the profile (separate cache namespaces)
- [x] **R-3 (P1)** Large graphs never sync-dagre on the main thread — `createWorkerLayoutRunner` throws without a Worker global; runners kind-tagged (`worker`/`dagre-sync`/`approximate`) so the degraded flag follows the actual runner (creation- AND runtime-failure paths); node count > `SYNC_DAGRE_MAX_NODES` → `createApproximateLayoutRunner` + `degraded` flag; approximate result skipped from the schemaHash cache (cannot poison repeat opens); small graphs keep the sync fallback
- [x] **R-4 (P2)** Canvas colors resolve from canonical CSS tokens — `ErThemeTokens` + `ErRenderer.updateTheme()` (getComputedStyle at mount + on theme-store change; no graph destroy); computed-style repaint asserted in the CDP harness (hex → `rgb()` normalization); `updatePositions` batched via `cy.batch`
- [x] **R-5 (P2)** Real-renderer runtime test — `__tests__/cytoscape-renderer.test.ts` (headless cytoscape) + `bench/er-renderer-runtime.html` CDP harness (esbuild bundle of the real renderer): 7/7 checks PASS @500/1000 (incl. theme repaint), 0 console errors
- [x] **R-6** Docs updated — TTI/TTD split, geometry contract (shared dependency-free `overview-geometry.ts`), fallback policy, theme evidence (FINDINGS F-R1..5, VERIFICATION "P1 review fixes")
- [x] **R-7** Gates closed — **1,445 FE tests** (124 files), tsc 0, lint/prettier/check:tokens clean, build OK

## Gates

- [x] All Phase 1 items complete before merging — P1.1–P1.9 + code-split + 6.11 all `[x]` above; pre-merge review found 0 P0 / 0 P1
- [x] Phase 2 complete before adding new shadcn components — P3.1 + P3.2 landed (canonical vocabulary + shadcn alias layer + drift guard in CI); verified 96/96 token values unchanged
- [x] Phase 3+4 complete before v0.1 release — P3.3/P3.4 (Phase A) closed with parity + benchmark tests; P3.5/P3.6 closed with dispatcher + neighborhood scale tests; P3.7 budgets reconciled against P1.8/P1.9 evidence; P3.8 list-view audit clean (0 O(N²), F-P3.8-1 fixed); P1 series + 6.11 cover the rendering architecture. Final pre-merge review (2026-08-12): 0 P0 / 0 P1
- [x] P0 = 0, P1 = 0 across all phases — verified by the final pre-merge review of the full branch (18 commits, 163 files, +22,151/−1,328): Phase 1 (P1 series) 0/0; Phase 2 (P3.1/P3.2) 0/0 (96/96 token regression); Phase 3 (P3.3–P3.8 audits) 0/0; provider matrix None/None (0 backend/Rust files changed). P2s fixed during the branch: dev harness out of public/, explorer driver/dialect hoist (F-P3.8-1), unmemoized `tablesInSchema` (review fix). Remaining P2s tracked in FINDINGS.md (F-P3.8-2, F-B3, F-MR-1..4)
- [x] P1 invariants hold per fixture (`graphTables ≠ renderedTables ≠ detailedTables`) — verified mechanically by `__tests__/rendering-invariant.test.ts` on the A500 fixture (SpatialIndex culling + resolveLod at overview zoom); strict `≠` is an overview-scale property (all visible nodes are detailed at zoom ≥ 0.7 by design)

- [x] **R-8 (P1)** Overview click-to-focus is not swallowed by navigation — single tap = focus ONLY (`setSeed`); new `onNodeDoubleClick` (renderer `dbltap`) + side-inspector **Open table** button are the only navigation paths (`onOpenTable` prop; ErDiagram renamed `onNodeClick` → `onOpenTable`). App-level tests: `cytoscape-view.test.tsx` (6: single tap → exact fade+highlight `updateSelection`, never `onOpenTable`; dbltap + Open-table navigate; background tap keeps ring + query; re-tap after background focuses) + `er-diagram-navigation.test.tsx` (2: workspace stays active until the explicit action; `openDbObject` receives the correct db-object tab). Real-renderer dbltap routing test added
- [x] **R-9 (P2)** Background tap ≠ search clear — `onBackgroundTap` clears focus/fade only; `searched` rings are search state and survive a non-empty query (Escape clears both). Harness +1 check pair at 500 & 1000 (`singleTapFocus` / `doubleTapOpens`); 16/16 checks PASS, 0 console errors
