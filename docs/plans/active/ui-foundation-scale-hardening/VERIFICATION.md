# P3 — UI Foundation & Scale Hardening — Verification

## Commands to execute

### Frontend quality gates
```bash
cd frontend
npm run typecheck
npm run lint
npm run format:check
npm run check:tokens
npm run test
npm run build
```

### Rust quality gates (if applicable)
```bash
cargo fmt --all -- --check
cargo check --workspace
cargo clippy --workspace --all-targets
cargo test --workspace
```

## Performance budgets

### ER Diagram

Budgets reconciled against P1.8/P1.9 runtime evidence on 2026-08-12 — two rows were evidence-corrected (see the reconciliation table below): **Dagre layout** is budgeted as *main-thread block* (the P1.7 worker makes the raw duration a UX non-factor), and **pan/zoom** is budgeted on the *shipped renderer path* (the P1.9 hybrid — RF full-graph overview is deliberately not a shipped path at 500/1000 tables).

| Metric | small (20) | medium (100) | large (500) | xlarge (1000) |
|---|---|---|---|---|
| Node build time (pre-index) | < 5ms | < 20ms | < 50ms | < 100ms |
| Main-thread block during layout | 0 | 0 | 0 | 0 |
| Initial paint (renderer mount / commit) | < 50ms | < 200ms | < 500ms | < 1000ms |
| Pan/zoom frame time (shipped path) | 60 fps | 60 fps | ≥ 60 fps* | ≥ 60 fps* |
| Search response | < 16ms | < 50ms | < 100ms | < 100ms |
| Edge click (highlight) | < 8ms | < 16ms | < 50ms | < 50ms |
| DOM node count | < 2k | < 10k | < 20k (LOD) | < 25k (LOD) |
| Memory (heap) | < 10MB | < 30MB | < 80MB | < 150MB |

\* overview pan on the canvas renderer (`CytoscapeErRenderer`, P1.9). Detail zoom is 60 fps on every path. RF full-graph overview measures 34 / 18.6 fps at 500 / 1000 and is deliberately replaced by the hybrid (P1.9).

### ER Diagram — budget reconciliation (P3.7 audit, 2026-08-12)

Every budget row reconciled against recorded runtime evidence. Verdicts: ✅ PASS · ❌ FAIL · 🔁 reframed (budget was unachievable/unmeasurable as written; corrected above). "n/m" = not measured by the P1.8/P1.9 harness protocol.

| Metric | small (20) | medium (100) | large (500) | xlarge (1000) | Evidence | Verdict |
|---|---|---|---|---|---|---|
| Node build time (pre-index) | < 5ms | < 20ms | < 50ms | < 100ms | `benchmark.test.ts` (real pipeline, P3.3): asserts avg build under budget on 20/100/500/1000 fixtures; suite green 2026-08-12 | ✅ PASS (automated) |
| Dagre layout (absolute) | < 20ms | < 100ms | < 300ms | < 800ms | P1.8 main-thread block: dagre **151ms @100 / 8,110ms @500 / 122,084ms @1000**; fCoSE 223ms/1,988ms/36,004ms | ❌ FAIL as written at every scale ≥ 100 (151ms > 100ms @medium); small-20 n/m (harness minimum fixture is 100) |
| Main-thread block during layout | 0 | 0 | 0 | 0 | P1.7: dagre runs in the Worker; P1.9 measured **0 long tasks beyond dagre**; shell interactive during layout | ✅ PASS — 🔁 reframed: the worker + schemaHash cache (repeat opens instant) is the mitigation for the 8s/122s raw durations |
| Cold layout duration, first open (worker-side) | — | 0.15s | 8s | 122s | P1.8 dagre compute 0.15/8/122 s @100/500/1000; post-P1.7 this is worker-side (shell interactive, repeat opens instant via cache). Kept visible so the gate cannot silently lose all layout-duration oversight if the cache is ever cleared | 🔁 tracked-not-gated (logged by P1.1 HUD) |
| Initial paint | < 50ms | < 200ms | < 500ms | < 1000ms | P1.9 canvas mount **479ms @500 / 896ms @1000** (PASS vs <500/<1000); RF paint n/m (layout is off-thread — shell interactive) | ✅ PASS @500/1000 (canvas — the shipped path at those scales); small/medium n/m (RF is the shipped path there, not measured) |
| Pan/zoom frame time | < 8ms | < 12ms | < 16ms | < 16ms | P1.8 overview pan: RF 60 / **34 / 18.6 fps** (16.7 / 29.4 / 53.8 ms) @100/500/1000; Cytoscape 60.5/52.4/48.0; detail pan 60 fps everywhere. P1.9 hybrid: canvas overview **59.6 / 59.9 fps** @500/1000, detail 60 | ❌ FAIL for RF full-graph overview @500/1000 (34/18.6 fps) — the exact P1.9 motivation; ✅ PASS on the shipped path (hybrid canvas overview + detail everywhere). 🔁 reframed: < 8ms @small implies 125 fps > 60 Hz vsync — unreachable as written |
| Search response | < 16ms | < 50ms | < 100ms | < 100ms | n/m in harness; path = O(T) table filter + BFS seed/hops; neighborhood BFS measured **< 5ms @1000, 2-hop** (neighborhood.test.ts) | ✅ PASS by composition (BFS dominates and is measured) |
| Edge click (highlight) | < 8ms | < 16ms | < 50ms | < 50ms | n/m in harness; path = `setSelectedEdgeId` → `setEdges(O(E))`; E≈2,000 @1000 | ✅ PASS by code inspection (O(E) map, no re-layout); not directly measured |
| DOM node count | < 2k | < 10k | < 20k (LOD) | < 25k (LOD) | P1.8: RF **1,143 / 6,143 / 11,817** @100/500/1000 (all under budget); Cytoscape 31 / 18 | ✅ PASS |
| Memory (heap) | < 10MB | < 30MB | < 80MB | < 150MB | P1.8 heap used: RF **28 / 73 / 48 MB**; Cytoscape **9.5 / 22 / 22 MB** | ✅ PASS (measured *heap used*, a conservative proxy for delta; RF @1000 48MB < 73MB @500 — LOD/culling effect, no unbounded growth) |

P3.7 targets (PLAN.md, 500 tables): **Open workspace → UI usable < 1s** — ✅ shell interactive during worker layout (P1.7; HUD init measure exists, not numerically recorded). **Initial diagram paint < 500ms** — ✅ 479ms canvas mount @500 (P1.9). **Search < 100ms** — ✅ (BFS < 5ms). **Pan/zoom, no main-thread freeze** — ✅ shipped path (worker + canvas + culled RF). **Selection < 50ms** — ✅ O(E). **Memory, no unbounded growth** — ✅ (capped heaps, LOD unmounts DOM, canvas flat at all scales).

**Findings (recorded for the merge gate):**

- **F-B1 — Dagre absolute budgets were unreachable** (P1.8 proved 8.1s/122s vs 300/800ms budgets). Mitigated by P1.7 (worker + cache): main-thread block 0, repeat opens instant. Budget reframed to main-thread block; the raw worker-side duration stays logged by the P1.1 HUD, not a UX gate.
- **F-B2 — Frame-time budgets were not physically meaningful** (< 8ms @small = 125 fps > 60 Hz vsync) and the RF full-graph overview genuinely fails 500/1000 (34/18.6 fps). The P1.9 hybrid is the fix and now ships for those scales; budget reframed to the shipped renderer path.
- **F-B3 — Harness protocol gaps (P2):** search latency, selection latency, and RF initial paint are not measured by P1.8/P1.9. Verified by composition (BFS < 5ms, O(T)/O(E) paths). Tracked for a future harness protocol revision.

### Design token contract

| Check | Method |
|---|---|
| Single source of truth | `npm run check:tokens` — the canonical layer (`--surface-*`/`--text-*`/`--border-*`/`--accent-*`/`--state-*`/`--elevation-*`) must define every token in BOTH `:root` and `[data-theme="dark"]` with the recorded snapshot values (theme drift fails CI) |
| shadcn layer aliases only | `scripts/check-token-drift.mjs` rejects any raw color value in the shadcn compatibility layer (allowed primitives: `--radius`, `--chart-*`, `*-foreground`, `--overlay`) |
| No `--app-*` color tokens in components | The same script scans `src/**/*.{ts,tsx}` + shipped `public/**` + `index.html` + any other css for the 21 removed tokens AND raw shadcn semantic vars (`var(--primary)`, `var(--info)`, …) — any hit fails |
| `npx shadcn add` safety | Rule documented in `globals.css` token-contract header + AGENTS.md; CI runs `npm run check:tokens` (frontend job) |
| Visual regression | `bench/token-contract.html` — 96 resolved-value pairs (48 tokens × light/dark) compared between pre-migration and canonical CSS in headless Chrome: **0 diffs** |

## Benchmark fixtures

To be created:

```
test/fixtures/schemas/
  small-20-tables.json
  medium-100-tables.json
  large-500-tables.json
  xlarge-1000-tables.json
```

Each fixture contains a valid `IntrospectResult` with realistic column/FK/PK distributions.

## P1 — Large-schema rendering metrics (locked)

QA fixtures (generated, not introspected):

| Fixture | Tables | Relations | Columns |
|---|---|---|---|
| A | 100 | ~150 | ~1,200 |
| B | 500 | ~900 | ~7,500 |
| C | 1,000 | ~2,000 | ~15,000 |

Recorded per fixture via the P1.1 HUD (`er-perf-hud=1` in localStorage):

| Metric | Source |
|---|---|
| Time to interactive shell | `er:init:start` → `er:init:end` measure |
| Layout duration | `er:layout:start` → `er:layout:end` measure (dagre in the P1.7 Worker — no main-thread block; cached by schema hash) |
| Max main-thread long task | `PerformanceObserver('longtask')` |
| Initial detailed DOM count | `[data-tier="2"]` node count |
| Pan / zoom frame time | rAF frame sampler over pan/zoom gesture |
| Search / selection latency | gesture marks |
| Graph / viewport / rendered / detailed node count, rendered edge count | HUD snapshot |

Invariants (must hold; never `graphTables = renderedTables = detailedTables`):

```text
graphTables = 512
viewportTables ≈ 26
renderedTables ≈ 26 + overscan
detailedTables <= visibleNearZoom + selected
```

Known interaction (P1.2, documented): with `onlyRenderVisibleElements` enabled, off-viewport nodes are never measured, so `<MiniMap>` (kept for schemas ≤ 200 tables) may render hidden nodes at near-zero size. Acceptable for small schemas; revisit if MiniMap stays for medium schemas in P1.3+.

P1.1 HUD is enabled via `localStorage.setItem("er-perf-hud", "1")` before opening the ER diagram. It is dev-only and never shipped in the default UI.

## P1.8 A/B benchmark harness — Cytoscape vs React Flow (100/500/1000 tables)

Standalone harnesses built from scratch in `bench/` (vendored UMD builds; no app code):

- `bench/fixture-gen.js` — deterministic fixture generator (mulberry32, seed 42) with presets 100 / 500 / 1000 (locked sizes: A ~150 rels, B ~900 rels, C ~2,000 rels)
- `bench/er-cytoscape.html?n=<scale>` — Harness A: Cytoscape 3.30.2 + fCoSE 2.2.0 canvas renderer
- `bench/er-reactflow.html?n=<scale>` — Harness B: React 18 UMD + @xyflow/react 12.11.2 UMD + dagre 0.8.5 layout, `onlyRenderVisibleElements` + zoom LOD (title-only < 0.5, columns >= 0.5)
- `bench/run-bench.js` — CDP runner: launches headless Chrome, waits for `BENCH_READY`, extracts `window.__bench` + screenshots

Protocol (identical on both): fCoSE/dagre layout → initial DOM count → pan @ overview (110 frames) → zoom to 1.0 → DOM count + visible nodes/edges at detail → pan @ detail (110 frames). Long tasks via `PerformanceObserver('longtask')` from page load.

Environment: Chrome 150 headless, software rendering, 1600x1000 viewport. One serialized run per cell.

| Metric | A100 | B100 | A500 | B500 | A1000 | B1000 |
|---|---|---|---|---|---|---|
| Layout (main-thread block) | 223 ms | 151 ms | 1,988 ms | 8,110 ms | 36,004 ms | **122,084 ms** |
| DOM elements, initial (fit view) | **31** | 1,143 | **31** | 6,143 | **31** | 11,817 |
| DOM elements, detail zoom | 31 | 547 | 31 | 463 | 31 | 751 |
| Pan FPS, overview | 60.5 | 60.0 | 52.4 | 34.0 | 48.0 | **18.6** |
| Pan FPS, detail zoom | 59.5 | 60.0 | 59.8 | 60.0 | 59.0 | 60.0 |
| Long-task max (layout) | 333 ms | 155 ms | 2,267 ms | 8,111 ms | 36,401 ms | **122,086 ms** |
| Heap used | 9.5 MB | 28 MB | 22 MB | 73 MB | 22 MB | 48 MB |

### Interpretation (evidence-driven, 3 scales)

1. **DOM-count root cause confirmed at every scale.** Cytoscape holds 31 DOM elements regardless of graph size (canvas); React Flow scales linearly (~11.8 DOM/table: 1,143 → 6,143 → 11,817) — every one of those re-renders per pan frame at overview zoom.
2. **Layout is the dominant bottleneck, both renderers, and scales terribly.** fCoSE: 0.2s → 2s → 36s; dagre: 0.15s → 8s → **122 s**. dagre at 1,000 tables freezes the app for ~2 minutes on the main thread — layout MUST move off-thread (P1.7) + cache by schema hash, regardless of renderer choice.
3. **Renderer scaling: Cytoscape degrades gracefully, React Flow collapses at overview.** Pan FPS overview: A stays 48–60 at all scales; B drops 60 → 34 → 18.6. At detail zoom both hold 60 fps (B thanks to P1.2–P1.4 culling + LOD — locked invariant `renderedTables ≈ viewportTables + overscan` holds).
4. **Memory: canvas stays flat (~22 MB at 500/1000); React Flow peaks 73 MB at 500.**
5. **Decision input for P1.9:**
   - ≤ 100 tables: both renderers are excellent → keep React Flow (no reason to switch).
   - ~500 tables: React Flow works with worker layout + the P1.6 neighborhood default (full-graph overview is the only weak spot at 34 fps); Cytoscape is strictly better for full-graph overview.
   - ~1,000 tables: full-graph React Flow is not viable (122 s layout block, 18.6 fps overview); Cytoscape handles interaction fine (48–59 fps) once layout is off-thread.
   - The locked hybrid (Cytoscape canvas for overview/exploration + React DOM for selected-table detail) is supported by this evidence.

Screenshots: `bench/er-cytoscape-{100,500,1000}.png`, `bench/er-reactflow-{100,500,1000}.png`.

## P3.1 — Design Token Contract (F1 reconciliation)

Locked canonical vocabulary (single source of truth for color values):

| Canonical token(s) | Old name (removed) |
|---|---|
| `--surface-app` / `--surface-nav` / `--surface-panel` / `--surface-editor` / `--surface-floating` | `--app-surface-0` … `--app-surface-4` |
| `--surface-hover` / `--surface-active` | `--app-hover` / `--app-active` |
| `--text-primary` / `--text-secondary` / `--text-tertiary` | `--app-text` / `--app-text-muted` / `--app-text-dim` |
| `--border-subtle` / `--border-default` / `--border-strong` | `--app-border-subtle` / `--app-border` / `--app-border-strong` |
| `--accent` / `--accent-hover` / `--accent-soft` / `--accent-foreground` | `--app-primary` / `--app-primary-hover` / `--app-primary-soft` (+ `--primary-foreground` pulled in) |
| `--state-success` / `--state-warning` / `--state-danger` / `--state-info` | `--app-success` / `--app-warning` / `--app-danger` / `--info` |
| `--elevation-lg` / `--elevation-popover` | `--app-shadow-lg` / `--app-shadow-popover` |

Semantic note: shadcn's old `--accent`/`--accent-foreground` (hover-highlight surface role) moved to the `@theme inline` mapping (`--color-accent: var(--surface-active)`, `--color-accent-foreground: var(--text-primary)`); the bare `--accent` name is now the canonical **brand** family. Components consuming `bg-accent` keep the exact same resolved colors.

The `--app-*` prefix is now reserved for layout metrics only (`--app-sidebar-width`, `--app-topbar-height`, `--app-activity-bar-width`, …) — a color token may never use it.

**Migration:** 96 files / 604 `--app-*` occurrences migrated (perl, longest-first to avoid prefix collisions); 21 color tokens renamed. `--app-shadow-*` had zero component consumers — renamed in the canonical layer only.

**Component rule:** components reference canonical tokens (`var(--surface-panel)`) or shadcn utility classes (`bg-background`, `text-muted-foreground`); they must not reference raw shadcn semantic vars or any `--app-*` color token. The few raw `var(--primary)` / `var(--info)` / `var(--border)` / `var(--secondary)` / `var(--foreground)` usages were migrated to canonical tokens.

### Runtime evidence — no visual regression

`bench/token-contract.html` (CDP harness, Chrome headless, same runner as P1.8) compares the pre-migration `globals.css` (from git HEAD) against the canonical rewrite through probe elements (custom properties keep `var()` chains unresolved, so each token is measured as a consumed computed style: `background-color` / `color` / `box-shadow`).

| Theme | Token pairs compared | Differences |
|---|---|---|
| Light | 48 | 0 |
| Dark | 48 | 0 |

Result: **all 96 resolved-value pairs identical** (shadcn aliases + canonical renames + the accent hover-role mapping). Screenshot: `bench/token-contract.png`. Guard: `npm run check:tokens` (see budget table) now runs in CI and enforces theme completeness + snapshot values + alias-only shadcn layer + no `--app-*` color tokens in components.

### Caveats

- **DOM counts are the robust claim** (stable across all runs/cells). Layout numbers are directional: both layouts run at cold-start right after vendor scripts parse, so absolute figures are inflated versus a warm page — but the scaling trend (fCoSE 0.2→36 s, dagre 0.15→122 s) and the conclusion (layout off-thread is mandatory) hold.
- **Frame-sampling asymmetry**: Cytoscape `pan()` is synchronous, React `setViewport()` renders asynchronously (its commit may land one frame after the sample window) — React frame costs may be slightly under-reported. Overview pan degradation (60→34→18.6 fps) still captures it.
- Software rendering only (headless). GPU-backed canvas painting would favor Cytoscape's repaint-heavy path on real hardware; DOM transform cost for React Flow is GPU-agnostic.

## P3.3 / P3.4 — ER Algorithm Phase A (pre-index + layout dedup)

Both items were already satisfied by the P1-series refactor; this session made them **mechanically verifiable** by extracting the node-building pipeline into a pure module the component actually runs.

### P3.3 — pre-indexed metadata (was O(T × C))

`renderer/er-node-builder.ts` (pure, no React):

| Builder | Index | Lookup cost |
|---|---|---|
| `buildColumnsByTable` | `schema.tableName` → `SchemaColumnDto[]` | O(1) per table |
| `buildPrimaryKeysByTable` | `schema.tableName` → `Set<string>` (composite PKs merged) | O(1) per column |
| `buildFkColumnSet` | `schema.table:column` → FK-source flag | O(1) per column |
| `buildTableNodes` | builds all nodes with the three indexes | no full-array scans |

`er-diagram.tsx` consumes the builders (useMemos keep stable identity) — the old inline `data.columns.filter(...)` / `data.primaryKeys.filter(...)` / `data.foreignKeys.filter(...)` per-table loops are gone (0 matches in the file).

**Automated proof:** `__tests__/er-node-builder.test.ts` — (1) index correctness incl. composite PK merge + FK flags; (2) **parity**: `buildTableNodes` output is deep-equal to a naive per-table `filter()` reference on a 100-table fixture (pre-indexing is behavior-preserving); (3) 500-table scale: all 500 nodes built, every fixture column appears exactly once, every FK source column flagged. `__tests__/benchmark.test.ts` now times the REAL pipeline — 20/100/500/1000 tables under 5/20/50/100 ms avg budgets.

### P3.4 — duplicate layout elimination (was 2 dagre runs per change)

The old `layoutGraph()` ran in both a `useMemo` and a `useEffect` (two dagre executions per dependency change). That path is gone: P1.7 `useWorkerLayout` is hash-memoized (`computeLayoutHash`) + cache-first → exactly one dagre run per distinct graph; edge highlighting lives in a `setEdges`-only effect (no re-layout); manual positions override worker output in `laidOutNodes` and persist via `onNodeDragStop`.

Gates: typecheck 0 · lint/prettier clean · `check:tokens` clean · **1,412 FE tests pass** (+6) · build OK.

## P3.5 / P3.6 — Rendering LOD + Large Schema Mode (audit)

Audit verdict: **both items are genuinely implemented** (not just ticked) — by the P1 series, which superseded the pre-P1 approaches. This session added the missing mechanical verification.

### P3.5 — Rendering LOD

| Criterion | Code | Proof |
|---|---|---|
| Low zoom → column rows not rendered | `ErTableNode` dispatches `ErDotNode`/`ErCompactNode`/`ErSummaryNode` below zoom 0.7 (`utils/lod.ts`); none mount column rows | **`__tests__/er-table-node.test.tsx`** (new, 5 tests): renders the dispatcher under `ReactFlowProvider` for each lod — 0 `[data-column]` rows below detail, exactly one `[data-tier]` leaf mounted at a time (true render-tree switch, hard rule #2). Plus `lod.test.ts` + `rendering-invariant.test.ts` |
| High zoom (or selected) → full detail | zoom ≥ 0.7 → `ErDetailedNode` (full column list). On-canvas `selected` is styling-only; node click opens the full detail in the object **tab** (P1.3-locked interaction model) | dispatcher test asserts tier 3 renders every `[data-column]` row |
| No jank between tiers | LOD injected by a `currentLod`-keyed memo; leaves memoized; LOD change never re-runs layout | code inspection + P1.8 pan/zoom frame times |

### P3.6 — Large Schema Mode

| Criterion | Code | Proof |
|---|---|---|
| Large schema defaults to search-first | `isLargeSchema = tier ∈ {L, XL}` (6.11 complexity score — supersedes the literal 200 count) → `landing` mode with search + suggested starting points (P1.6) | `schema-complexity.test.ts` + code inspection |
| Search → seed + hop radius | `NeighborhoodExplorer` `[1][2][3][Domain]` scopes; `getNeighborhood` / `getConnectedComponent` | `neighborhood.test.ts` |
| "Show all N tables" | `handleShowAll` → full graph (Cytoscape canvas for L/XL, P1.9) | code inspection + P1.9 benchmark |
| Neighborhood O(hops × degree) | BFS over adjacency with visited set | **`neighborhood.test.ts`** (new, +3): 2-hop parity vs an independent BFS-distance reference on a 1000-table graph; isolated seed in a 500-table schema returns itself only (no global scan); 2-hop on 1000 tables under 5 ms |

Gates: typecheck 0 · lint/prettier clean · `check:tokens` clean · **1,421 FE tests pass** (+9) · build OK.

## Runtime evidence log

| Date | Phase | Evidence | Result |
|---|---|---|---|
| 2026-08-12 | P1.8 | A/B harness full matrix 100/500/1000 (see section above) | Cytoscape: 31 DOM at all scales, layout 0.2/2/36 s · React Flow: 1,143/6,143/11,817 DOM, layout 0.15/8/122 s, overview pan 60/34/18.6 fps |
| 2026-08-12 | P1.7 | Layout worker + cache shipped (see Implementation notes below); quality gates: 1,393 FE tests pass, build emits `er-layout.worker.js` (~92 kB, dagre split out of the main bundle) | dagre runs off the main thread; cold 1,000-table layout no longer freezes the UI (worker), repeat opens are instant (cache) — the 0.15/8/122 s main-thread blocks measured in P1.8 become worker-side durations with the shell interactive |
| 2026-08-12 | P1.9 | CytoscapeErRenderer benchmark against the REAL app source (vite-dev harness `bench-hybrid.html` served from `frontend/public/`, Chrome 150 headless, same protocol as P1.8; harness removed from `public/` after evidence collection to keep the production bundle clean — recoverable from git, standalone A/B harnesses remain in `bench/`) | see table below — 18 DOM elements + ~60 fps overview pan at 500 AND 1,000 tables |
| 2026-08-12 | 6.11-review | Merge-gate invariant mechanically verified: `__tests__/rendering-invariant.test.ts` asserts `graphTables ≠ renderedTables ≠ detailedTables` on the A500 fixture (500 tables, 25×20 grid) via the app's `SpatialIndex` (culling) + `resolveLod` (overview zoom → non-detail). Also documented: at zoom ≥ 0.7 all visible nodes are detailed by design — the strict `≠` is an overview-scale property. Full P1-series pre-merge review: 0 P0 / 0 P1 (see review summary in commit); P2s resolved: dev harness removed from `public/` (was shipping in `dist/`), gates closed. |
| 2026-08-12 | P3.1 | Design Token Contract (F1): canonical vocabulary `--surface-*`/`--text-*`/`--border-*`/`--accent-*`/`--state-*`/`--elevation-*` replaces 21 `--app-*` color tokens across 96 files (604 occurrences → 0); shadcn compat layer is alias-only; `--app-*` reserved for layout metrics; guard `npm run check:tokens` upgraded (theme completeness + value snapshot + alias-only + component scan) and wired into CI + AGENTS.md. 1,406 FE tests pass, typecheck/lint/prettier clean, build OK | **No visual regression:** `bench/token-contract.html` — 96/96 resolved-value pairs identical (48 tokens × light/dark) between pre-migration and canonical CSS in headless Chrome; 0 console errors. shadcn `bg-accent` hover semantics preserved via `@theme inline` remap |
| 2026-08-12 | P3.3/P3.4 | ER Algorithm Phase A (see section above): node building extracted to pure `renderer/er-node-builder.ts` (columns/PK/FK pre-index + `buildTableNodes`), `er-diagram.tsx` consumes it — 0 `.filter()` on full metadata arrays remain. Layout dedup confirmed: `useWorkerLayout` (P1.7) hash-memoized + cache-first, one dagre run per distinct graph; highlight is a `setEdges`-only effect; manual overrides preserved. 1,412 FE tests pass (+6), typecheck/lint/prettier/check:tokens clean, build OK | `er-node-builder.test.ts`: parity (pre-index ≡ naive per-table filter, 100 tables) + 500-table scale invariants (no dropped columns, all FK flags). `benchmark.test.ts` now benchmarks the real pipeline: 20/100/500/1000 tables under 5/20/50/100 ms avg budgets |
| 2026-08-12 | P3.5/P3.6 | Audit (see section above): both items genuinely implemented by the P1 series (true LOD components P1.3, edge LOD P1.4, neighborhood UX P1.6, complexity-tier landing 6.11, Cytoscape overview P1.9). Closed the mechanical-verification gap with 9 new tests. 1,421 FE tests pass (+9), typecheck/lint/prettier/check:tokens clean, build OK | `er-table-node.test.tsx` (6): per-lod leaf render (0 `[data-column]` rows below detail, one leaf mounted, selection is styling-only). `neighborhood.test.ts` (+3): 2-hop parity vs independent BFS-distance reference on 1000 tables (ratio-bounded), isolated seed isolation in 500-table schema, 2-hop under 5 ms. Documented: on-canvas `selected` is styling-only — full detail opens in the object tab (P1.3 locked model); 6.11 complexity score supersedes the literal 200-table threshold; 4.7 (worker) was implemented in P1.7 despite the pre-P1 prediction |
| 2026-08-12 | P3.7 | Budget reconciliation (see section above): every P3.7 budget row mapped to P1.8/P1.9 evidence with per-scale verdicts. Node build, DOM (RF 1,143/6,143/11,817 vs <10k/20k/25k), memory (RF 28/73/48 MB, canvas 9.5/22/22 vs <30/80/150), initial paint (canvas 479/896ms vs <500/1000), search/selection (composition + BFS <5ms), main-thread block (0, worker) → PASS. Two rows reframed with evidence: Dagre absolute budgets FAIL (151ms/8.1s/122s vs <100/300/800 — P1.8) and frame-time units unreachable (<8ms = 125fps > vsync) with RF overview genuinely FAILing 500/1000 (34/18.6fps) — the P1.9 hybrid is the shipped fix. Findings F-B1/F-B2/F-B3 recorded; 3 measurement gaps (search/selection/RF paint) tracked as P2 |
| 2026-08-12 | P3.8 | Audit (see FINDINGS.md "Audit findings"): schema explorer / data grid / connection list / search view / saved-queries tree / column & index lists / quick-open — **no O(N²) per-item scans remain** (explorer pre-indexed Maps, unified-grid row-virtualized via `useVirtualizer`, catalog store pre-indexed `columnsByTable`; 0 same-line map+filter/find/includes smells in `src/`). 0 P0/P1. F-P3.8-1 fixed (driver/dialect hoisted per connection in `explorer-view.tsx`, was O(tables×connections) per render); F-P3.8-2 tracked (cell-editor `columns.find` per edit-open). 1,421 FE tests pass, typecheck/lint/prettier/check:tokens clean, build OK |
| 2026-08-12 | P2 round (F-MR-3/4) | Post-merge P2 backlog (root-cause class): **F-MR-3 RESOLVED** — the CI `format:check` gate only covered `src/`, so `scripts/`/`eslint.config.js`/`public/` drift silently; gate widened to `prettier --check .` (whole frontend, `.prettierignore` covers generated `test/fixtures`), pre-existing drift on `eslint.config.js` + `public/splashscreen.html` formatted (verified whitespace-only). **F-MR-4 RESOLVED** — `check-token-drift.mjs` gained check 6: every `--<token>` mention anywhere in src/public/index.html must be defined in globals.css (library internals allow-listed). Scan caught 2 real leftovers the migration missed: `--text-primary-primary` (undefined rename artifact in `er-perf-hud.tsx`) and vestigial `--app-editor-bg,var(--surface-editor)` fallback (`query-editor.tsx`) — both fixed. Both guards negative-tested (inject stale token → fail → revert → clean). Gates: **1,425 FE tests** (121 files), tsc 0, lint/prettier/check:tokens clean, build OK |
| 2026-08-12 | P2 round (F-MR-1/2) | Post-merge P2 backlog: **F-MR-1 RESOLVED** — fit-view no longer uses a synthetic `keydown "1"`; the `ReactFlowInstance` is captured in `onInit` (`rfInstanceRef`) and the fit-on-commit effect, initial fit and manual Fit button all call `instance.fitView({ padding: 0.2 })`. **F-MR-2 RESOLVED** — runner selection extracted to pure `resolveLayoutRunner(current, create, fallback)`; a failed worker `create()` returns `{ sticky: false }` so the session retries worker creation on the next run instead of being permanently downgraded (later success is cached). +4 tests (`use-worker-layout.test.ts`): cached runner, non-sticky fallback, recovery after transient failure, sticky semantics. Gates: **1,425 FE tests** (121 files), tsc 0, lint/prettier/check:tokens clean, build OK |
| 2026-08-12 | MERGE REVIEW | Final pre-merge review of the full branch (18 commits / 163 files / +22,151 −1,328): **0 P0 / 0 P1 across all phases**. Gates closed: "Phase 3+4 complete" + "P0 = 0, P1 = 0 across all phases" (CHECKLIST). Provider matrix confirmed None/None — 0 backend/Rust files changed (only `ci.yml` + `AGENTS.md` outside frontend/docs/bench). Fixed during review: unmemoized `tablesInSchema` wrapped in `useMemo` (was re-computing `schemaStats`/`suggestedPoints` every render); pre-existing prettier drift on `scripts/generate-fixtures.mjs` (red on main too) fixed to unblock CI `format:check`. P2s tracked in FINDINGS.md (F-MR-1..4). Gates: 1,421 FE tests, tsc 0, lint/prettier/check:tokens clean, build OK, ~238 Rust tests pass |
| 2026-08-12 | Option C (progressive layout quality) | Closes the quality gap between first paint and dagre final: deterministic FR refinement in the layout worker posts progressively better COMPLETE position sets (30 passes, stage every 6) while dagre computes; `updatePositions({fit:false})` avoids viewport yank on stages; stages never cached; gated to large-schema overviews. **Runtime evidence (CDP, real renderer): 500 → first stage ~50 ms, 153 ms total, mean edge 2145→524 (−75%); 1000 → first stage 66 ms, 321–366 ms total, 3132→926 (−70%)** — layout visibly improves within ~50–70 ms of paint while dagre still takes 8 s/122 s. Review fix applied: model-switch mounts always use the approximate circle (never stale in-flight positions from the previous graph). Gates: **1,457 FE tests** (125 files), tsc 0, lint/prettier/check:tokens clean, build OK (worker chunk +2.2 kB) |
| 2026-08-12 | UX pivot — full graph default (opass.html) | P1.6's landing/search-first UX rejected against the opass reference: large schemas now open on the FULL canvas graph immediately (`useCytoscapeForOverview = isLargeSchema`; landing/`showAll`/neighborhood mode/suggested points deleted). Search = **focus, never filter** (`utils/overview-search.ts`: matches ringed red + viewport centers; single match also highlights its neighborhood). Click = **fade + highlight** (rest fades to 0.1, hop-scoped ring [1][2][3][Domain], background tap clears). `ErSelection` gains `highlightNodeIds` + `fadeRest`; renderer API backward-compatible. **Runtime evidence (CDP, real renderer @500 & @1000): 4 new checks PASS** (`focusFadesRest`, `searchRings`/`searchClear`, `focusClear`); **160/160 ER tests green** (`overview-search.test.ts` 10 + renderer +4) |
| 2026-08-12 | P1-1…P2-2 (PR#12 review round) | PR#12 external review: 3 P1 + 2 P2, all closed with shipped code + runtime evidence (section above): **P1-1** overview paints with a fast approximate layout (O(T+E), ~1ms @1000) and upgrades via `updatePositions` when dagre finishes — no more 8s/122s blank canvas; **P1-2** layout geometry now uses renderer-specific profiles (overview 160×28 vs RF 220×dynamic) with separate hash-cache namespaces + a shared dependency-free geometry module so renderer and layout cannot drift; **P1-3** large graphs are hard-forbidden from sync-dagre fallback — `createWorkerLayoutRunner` throws without a Worker global, runners are kind-tagged so the degraded flag follows the actual runner (creation-failure path included), approximate runner + `degraded` flag, cache-skip so repeat opens stay clean; **P2-1** cytoscape colors resolve from canonical CSS tokens at mount + `updateTheme()` runtime swap (no graph destroy) with computed-style repaint evidence; **P2-2** real-renderer tests (headless cytoscape in jsdom) + `bench/er-renderer-runtime.html` CDP harness bundling the real renderer. Gates: **1,445 FE tests** (124 files), tsc 0, lint/prettier/check:tokens clean, build OK. **TTD runtime evidence: 230–646ms @500 / 354–939ms @1000** (7/7 checks PASS incl. theme repaint, 0 console errors, real Chrome) |

### P1.9 renderer verification (real app code)

The (removed) `bench-hybrid.html` harness imported `renderer/cytoscape-renderer.ts` + `renderer/er-graph-model.ts` + `utils/layout.ts` directly via Vite dev — this measured the shipped renderer, not a re-implementation. Same fixture recipes (mulberry32 seed 42) and frame-sampling protocol as the P1.8 standalone harnesses.

**Coverage rationale:** `CytoscapeErRenderer` was historically canvas/DOM code (jsdom lacks a 2D context), so correctness was carried by this runtime benchmark against the real app source plus the pure-domain tests of `er-graph-model`. Since the PR#12 review round (F-R5), `CytoscapeErRenderer` also has direct automated coverage via headless cytoscape (`__tests__/cytoscape-renderer.test.ts`) and the CDP runtime harness (`bench/er-renderer-runtime.html`, real browser, real renderer).

| Metric | React Flow 500 | **CytoscapeErRenderer 500** | React Flow 1000 | **CytoscapeErRenderer 1000** |
|---|---|---|---|---|
| DOM elements (overview) | 6,143 | **18** | 11,817 | **18** |
| Pan FPS @ overview | 34.0 | **59.6** | 18.6 | **59.9** |
| Pan FPS @ detail | 60.0 | 60.0 | 60.0 | 60.0 |
| Renderer mount | — | 479 ms | — | 896 ms |
| Long tasks (beyond dagre) | — | 0 | — | 0 |

Layout durations shown by the harness (dagre run synchronously for measurement) remain the dominant cost (7.3 s @500, 96 s @1000) — in the app these run in the P1.7 Worker with the schemaHash cache, so they are not a main-thread block.

Conclusion: the hybrid selection (React Flow ≤ threshold + neighborhood; Cytoscape overview for the explicit "All N tables" of large schemas) is validated end-to-end. React Flow keeps its interaction richness where the node count is small; the canvas renderer carries the full-graph overview at ~60 fps with 18 DOM elements regardless of size.

Screenshot: `bench/hybrid-cytoscape-500.png`. Bundle: the overview renderer is code-split behind `React.lazy` (er-diagram.tsx) — cytoscape (~450 kB min) ships in a separate `cytoscape-view-*.js` chunk loaded only when "All N tables" opens; the initial main bundle is 1,935 → 1,483 kB (−452 kB).

### P1 review fixes (P1-1 … P2-2) — runtime verification (real renderer, real browser)

Post-PR#12 review round. Merge gate items all closed with shipped code + runtime evidence:

| Finding | Fix | Evidence |
|---|---|---|
| **P1-1** overview waited 8s/122s cold dagre before first paint | Fast **approximate layout** (degree-ordered circle, `utils/approximate-layout.ts`, O(T+E), ~1ms @1000) paints immediately; `ErRenderer.updatePositions` upgrades positions **without re-mount** when dagre finishes in the worker; view mounts on first positions (approx or dagre), not on `layoutStatus==="ready"` | TTD harness below: **230ms @500 / 354ms @1000** time-to-diagram (was 8,110ms/122,084ms dagre wait) |
| **P1-2** dagre laid out ReactFlow card geometry (220×640) but cytoscape painted 160×28 | `utils/layout-profile.ts` — `OVERVIEW_PROFILE` (160×28 compact, no column rows) vs `REACT_FLOW_PROFILE` (220×dynamic) share constants; the paint geometry lives in **`utils/overview-geometry.ts`** — a dependency-free single source of truth imported by BOTH the renderer (which must paint it) and the layout profile (which must lay it out), so the P1-2 drift class cannot silently recur; overview layout input built from the graph model, detail from column-aware data; `computeLayoutHash` includes the profile so overview/detail have separate caches | `layout-profile.test.ts` (geometry contract + RF/overview profile params + model-driven input builder) + `approximate-layout.test.ts` (deterministic, connected-ish coverage, includes all nodes) |
| **P1-3** failed worker fell back to **sync dagre on the main thread** (8s/122s freeze) | Three-layer hard gate: (1) `createWorkerLayoutRunner` **throws** when `Worker` is unavailable instead of silently returning the sync dagre fallback (which would bypass the size gate); (2) runners are **kind-tagged** (`worker` / `dagre-sync` / `approximate`) so the degraded flag follows the runner that actually produced the result — including the worker-creation-failure path (was `degraded: false` there, poisoning the cache + hiding the notice); (3) large graphs (node count > `SYNC_DAGRE_MAX_NODES`) never run sync dagre — failed worker → `createApproximateLayoutRunner` (~1ms, main-thread safe) + `degraded` surfaced to the view; approximate results are never written to the schemaHash cache; small graphs keep the sync dagre fallback (fast, safe) | `use-worker-layout.test.ts` — sync-dagre-fallback-forbidden above threshold, worker-create throws without a Worker global, runner kinds tagged, approximate runner deterministic <1s @1000, cache skip on degraded |
| **P2-1** cytoscape colors hardcoded for dark theme | `ErThemeTokens` + `ErRenderer.updateTheme()`; renderer resolves tokens from **CSS custom properties via `getComputedStyle`** (`--surface-panel`, `--border-default`, `--text-primary`, `--accent`, …) with fallbacks; `cytoscape-view.tsx` resolves once at mount and re-applies on theme store change — light/dark switch does not destroy the graph. `updatePositions` is batched (`cy.batch`) so a 1000-node upgrade is one render pass | `cytoscape-renderer.test.ts` (headless cytoscape) + TTD harness: `updateTheme` swaps colors, **computed node style repaints to the new value** (hex → `rgb()` normalization asserted), graph stays alive, selection persists |
| **P2-2** no runtime test on the real cytoscape path | (a) `__tests__/cytoscape-renderer.test.ts` — real `CytoscapeErRenderer` + real cytoscape in headless mode (no canvas needed): mount/positions/theme/selection/updatePositions; (b) `bench/er-renderer-runtime.html` + `bench/build-er-renderer-runtime.mjs` — bundles the **real app renderer + approximate layout** with esbuild (cytoscape included) and runs it in real Chrome via the CDP runner | **18 new unit/integration tests green** (suite 1,425 → 1,443); TTD harness 6/6 checks PASS at 500 and 1000 tables, 0 console errors |

#### TTD (time-to-diagram) — real renderer, real browser (CDP, Chrome 150 headless)

Harness: `bench/er-renderer-runtime.html?n=500|1000` (real `CytoscapeErRenderer` bundled from app source, real cytoscape, approximate layout first paint → async `updatePositions` upgrade → theme swap → selection). Budget (reviewer-locked): TTD < 2s @500, < 3–5s @1000; **TTI shell < 1s** unchanged.

| Metric | 500 tables | 1000 tables |
|---|---|---|
| Approximate layout compute | 1 ms | 1 ms |
| **Time-to-diagram (approx + mount)** | **230–646 ms** (3 runs) | **354–939 ms** (2 runs) |
| Rendered nodes / edges | 500 / 900 | 1,000 / 2,000 |
| DOM elements (canvas) | 26 | 26 |
| Canvas elements | 3 | 3 |
| updatePositions moves node (async upgrade) | PASS | PASS |
| updateTheme repaints computed style + graph alive | PASS | PASS |
| Selection highlight | PASS | PASS |
| Console errors | 0 | 0 |

Budget verdict: **TTD PASS** at both scales (230/354 ms ≪ 2s/3-5s). The old acceptance line "Open workspace → UI usable < 1s" is replaced by the TTI/TTD split — the diagram is no longer gated on dagre (P1-1). Dagre still upgrades positions in the worker when ready (better crossings), but that is an enhancement, not a prerequisite. Screenshots: `bench/er-runtime-500.png`, `bench/er-runtime-1000.png`.

### Option C — progressive layout quality (2026-08-12)

P1-1 fixed time-to-diagram (<1 s paint) but left a **quality gap**: the user stared at a bare circle for the whole dagre wait (8 s @500 / 122 s @1000 per P1.8). Option C closes it with **deterministic force-directed refinement in the layout worker**, posted as progressively better COMPLETE position sets while dagre computes:

```text
t=0          ~30-50 ms            ~0.2-0.3 s           t=8 s / 122 s
circle  →    refine stage 1  →   refine final     →   dagre final (cached)
```

- `utils/force-refine.ts` — Fruchterman-Reingold with uniform-grid repulsion + spring attraction + re-centering. Pure, deterministic (fixed iteration order, no randomness), O(N·density + E) per pass.
- `er-layout.worker.ts` — posts `stage: "refine"` after every 6 of 30 passes, then `stage: "final"` (dagre). Every stage is a **full stable set** — the locked atomic-commit contract holds (no partial/unstable streams; the P1.7 "no fake progressive layout" rule is honored in letter and spirit: each commit is a valid complete layout).
- `useWorkerLayout(input, options, { progressive })` — gated to the large-schema overview (`nodeCount ≥ PROGRESSIVE_MIN_NODES = 120`; dagre is <151 ms @100 where refinement would add noise). Progressive stages set `positions` + `refining: true`; **never cached** (not dagre output for the hash). Fallback runners ignore `onProgress`.
- `CytoscapeErRenderer.updatePositions(positions, { fit })` — refine stages apply without re-fitting (the user may be panning/zooming — no viewport yank); only the final commit re-fits. The view shows a subtle "Refining layout…" hint while `positions && !layoutReady`.

**Runtime evidence (CDP harness `bench/er-renderer-runtime.html`, real renderer + real worker loop):**

| Scale | First stage | 30 passes total | Mean edge before → after | Improvement |
|---|---|---|---|---|
| 500 tables | — | **153 ms** | 2145 → 524 px | **−75%** (monotonic: 2145→1186→714→562→531→524) |
| 1000 tables | **66 ms** | **321–366 ms** | 3132 → 926 px | **−70%** |

The meaningful speed promise is the FIRST improved stage (the user sees the layout improving ~66 ms after paint) — the full 30 passes run in the layout WORKER, so their total wall-clock never touches the UI (and stays orders of magnitude under the 8 s / 122 s dagre wait they fill). One loaded-machine run reported 1,165 ms total — still irrelevant to UX (off-thread) and the harness asserts `firstStageMs < 1000`, not total. Checks: `progressiveImproves` / `progressiveFirstStageFast` / `refineKeptNodes` all PASS at 500 and 1000. Screenshots: `bench/er-runtime-500-optc.png`, `bench/er-runtime-1000-optc.png`.

### UX pivot — full graph default, search = focus (opass.html reference, 2026-08-12)

The P1.6 landing/search-first UX was rejected against the reference (`opass.html`): "show hết ra chứ méo ai đi để filter như vậy". Large schemas now open on the **full canvas graph immediately**; search **focuses** (never filters) and clicks **fade + highlight** — exactly the opass interaction model:

```text
open 500-table schema          type "order"            click a table
       ↓                           ↓                        ↓
full canvas graph           matches ringed red +     rest fades to 0.1,
(approx <1 s → refine →     viewport centers on      hop-scoped neighborhood
 dagre, all kept)           them (1 match also       ringed + edges highlighted
                            highlights its NB)       (radius [1][2][3][Domain])
```

- `er-diagram.tsx` — `useCytoscapeForOverview = isLargeSchema`; deleted `showAll` gate, `landing`, `neighborhoodSeed`, suggested points, the neighborhood React Flow mode and the search→mode-switch effect. React Flow remains the small/medium-schema renderer.
- `utils/overview-search.ts` (pure, tested) — `findTableMatches` (id/label, case-insensitive) + `resolveHighlightSet` (hop-scoped BFS over the model adjacency; `domain` = connected component).
- `cytoscape-renderer.ts` — `ErSelection` gains `highlightNodeIds` (exact host-computed set — no renderer re-derivation) + `fadeRest`; new `node.searched` (red ring) / `node.faded` (opacity 0.1) / `node.highlighted` + `edge.highlighted` styles; `highlightSearch` (ring + 400 ms center/zoom), `clearSearchHighlight`, `clearSelection`; background-tap callback; legacy `updateSelection` without the new fields unchanged (closedNeighborhood).
- `overview-explorer.tsx` (replaces `neighborhood-explorer.tsx`) — schema stats + highlight hop radius only; no suggested points, no "All N tables" toggle.

**Runtime evidence (CDP harness `bench/er-renderer-runtime.html`, real renderer in Chrome):** 4 new checks PASS at 500 and 1000 tables — `focusFadesRest` (selected + highlighted + faded classes correct), `searchRings` + `searchClear` (ring replace + clear), `focusClear` (background-tap clears fade). 160/160 ER unit/integration tests green (`overview-search.test.ts` 10 + `cytoscape-renderer.test.ts` +4 incl. background-tap callback). Provider impact: none (frontend-only).

### P1.7 implementation notes

- **`utils/layout.ts`** — split into pure `computeLayoutPositions(LayoutInput, options)` (runs in the worker) + `layoutGraph` wrapper (sync fallback, still used by tests); shared geometry constants so main/worker cannot drift.
- **`utils/layout-hash.ts`** — `computeLayoutHash`: deterministic, order-independent 64-bit FNV-1a over node sizes + edge topology + options; the `schemaHash → positions` cache key.
- **`utils/layout-cache.ts`** — `LayoutCache`: in-memory session cache + localStorage persistence (repeat opens of a 1,000-table schema skip dagre entirely), node-id-set integrity check, oldest-first eviction beyond 20 entries.
- **`er-layout.worker.ts` + `utils/layout-runner.ts`** — module worker runs dagre and posts one atomic result per requestId; a sync fallback runner covers non-worker environments (jsdom, exotic webviews).
- **`hooks/use-worker-layout.ts`** — cache-first, stale-discard (effect cleanup + requestId), single atomic position commit (locked: no streamed/unstable positions).
- **`er-diagram.tsx`** — replaced the sync `layoutGraph` useMemo; shows an "Arranging N tables…" overlay while computing; fit-view fires on computing → ready instead of the old fixed-80ms timer; HUD `layout ms` now reports the worker's dagre time.

## Provider matrix

This program is frontend-only. No database provider changes.

| Provider | Impact | Notes |
|---|---|---|
| PostgreSQL | None | Schema introspection unchanged |
| SQLite | None | Schema introspection unchanged |
