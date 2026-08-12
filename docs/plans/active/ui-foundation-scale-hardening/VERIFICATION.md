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

| Metric | small (20) | medium (100) | large (500) | xlarge (1000) |
|---|---|---|---|---|
| Node build time (pre-index) | < 5ms | < 20ms | < 50ms | < 100ms |
| Dagre layout | < 20ms | < 100ms | < 300ms | < 800ms |
| Initial paint (React commit) | < 50ms | < 200ms | < 500ms | < 1000ms |
| Pan/zoom frame time | < 8ms | < 12ms | < 16ms | < 16ms |
| Search response | < 16ms | < 50ms | < 100ms | < 100ms |
| Edge click (highlight) | < 8ms | < 16ms | < 50ms | < 50ms |
| DOM node count | < 2k | < 10k | < 20k (LOD) | < 25k (LOD) |
| Memory (heap delta) | < 10MB | < 30MB | < 80MB | < 150MB |

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

## Runtime evidence log

| Date | Phase | Evidence | Result |
|---|---|---|---|
| 2026-08-12 | P1.8 | A/B harness full matrix 100/500/1000 (see section above) | Cytoscape: 31 DOM at all scales, layout 0.2/2/36 s · React Flow: 1,143/6,143/11,817 DOM, layout 0.15/8/122 s, overview pan 60/34/18.6 fps |
| 2026-08-12 | P1.7 | Layout worker + cache shipped (see Implementation notes below); quality gates: 1,393 FE tests pass, build emits `er-layout.worker.js` (~92 kB, dagre split out of the main bundle) | dagre runs off the main thread; cold 1,000-table layout no longer freezes the UI (worker), repeat opens are instant (cache) — the 0.15/8/122 s main-thread blocks measured in P1.8 become worker-side durations with the shell interactive |
| 2026-08-12 | P1.9 | CytoscapeErRenderer benchmark against the REAL app source (vite-dev harness `bench-hybrid.html` served from `frontend/public/`, Chrome 150 headless, same protocol as P1.8; harness removed from `public/` after evidence collection to keep the production bundle clean — recoverable from git, standalone A/B harnesses remain in `bench/`) | see table below — 18 DOM elements + ~60 fps overview pan at 500 AND 1,000 tables |
| 2026-08-12 | 6.11-review | Merge-gate invariant mechanically verified: `__tests__/rendering-invariant.test.ts` asserts `graphTables ≠ renderedTables ≠ detailedTables` on the A500 fixture (500 tables, 25×20 grid) via the app's `SpatialIndex` (culling) + `resolveLod` (overview zoom → non-detail). Also documented: at zoom ≥ 0.7 all visible nodes are detailed by design — the strict `≠` is an overview-scale property. Full P1-series pre-merge review: 0 P0 / 0 P1 (see review summary in commit); P2s resolved: dev harness removed from `public/` (was shipping in `dist/`), gates closed. |
| 2026-08-12 | P3.1 | Design Token Contract (F1): canonical vocabulary `--surface-*`/`--text-*`/`--border-*`/`--accent-*`/`--state-*`/`--elevation-*` replaces 21 `--app-*` color tokens across 96 files (604 occurrences → 0); shadcn compat layer is alias-only; `--app-*` reserved for layout metrics; guard `npm run check:tokens` upgraded (theme completeness + value snapshot + alias-only + component scan) and wired into CI + AGENTS.md. 1,406 FE tests pass, typecheck/lint/prettier clean, build OK | **No visual regression:** `bench/token-contract.html` — 96/96 resolved-value pairs identical (48 tokens × light/dark) between pre-migration and canonical CSS in headless Chrome; 0 console errors. shadcn `bg-accent` hover semantics preserved via `@theme inline` remap |
| 2026-08-12 | P3.3/P3.4 | ER Algorithm Phase A (see section above): node building extracted to pure `renderer/er-node-builder.ts` (columns/PK/FK pre-index + `buildTableNodes`), `er-diagram.tsx` consumes it — 0 `.filter()` on full metadata arrays remain. Layout dedup confirmed: `useWorkerLayout` (P1.7) hash-memoized + cache-first, one dagre run per distinct graph; highlight is a `setEdges`-only effect; manual overrides preserved. 1,412 FE tests pass (+6), typecheck/lint/prettier/check:tokens clean, build OK | `er-node-builder.test.ts`: parity (pre-index ≡ naive per-table filter, 100 tables) + 500-table scale invariants (no dropped columns, all FK flags). `benchmark.test.ts` now benchmarks the real pipeline: 20/100/500/1000 tables under 5/20/50/100 ms avg budgets |

### P1.9 renderer verification (real app code)

The (removed) `bench-hybrid.html` harness imported `renderer/cytoscape-renderer.ts` + `renderer/er-graph-model.ts` + `utils/layout.ts` directly via Vite dev — this measured the shipped renderer, not a re-implementation. Same fixture recipes (mulberry32 seed 42) and frame-sampling protocol as the P1.8 standalone harnesses.

**Coverage rationale:** `CytoscapeErRenderer` / `CytoscapeErView` have no unit tests because the renderer is canvas/DOM code (jsdom lacks a 2D context); correctness is carried by this runtime benchmark against the real app source plus the pure-domain tests of `er-graph-model` (the data it renders).

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
