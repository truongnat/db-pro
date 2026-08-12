# P3 — UI Foundation & Scale Hardening — Verification

## Commands to execute

### Frontend quality gates
```bash
cd frontend
npm run typecheck
npm run lint
npm run format:check
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
| No duplicate token definitions | Grep `globals.css` for raw color values in shadcn block — should all be `var(--*)` references |
| `npx shadcn add` safety | Run `npx shadcn add button` on a test branch, verify `globals.css` token section unchanged |
| Visual regression | Manual: open app in light + dark, verify all surfaces match pre-migration |

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
| Layout duration | `er:layout:start` → `er:layout:end` measure (main thread until P1.7) |
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

## P1.8 A/B benchmark harness — Cytoscape vs React Flow (500 tables)

Standalone harnesses built from scratch in `bench/` (vendored UMD builds; no app code):

- `bench/fixture-500.js` — deterministic fixture (mulberry32, seed 42): 500 tables / 900 relations / 8,426 columns / 20 domains
- `bench/er-cytoscape-500.html` — Harness A: Cytoscape 3.30.2 + fCoSE 2.2.0 canvas renderer
- `bench/er-reactflow-500.html` — Harness B: React 18 UMD + @xyflow/react 12.11.2 UMD + dagre 0.8.5 layout, `onlyRenderVisibleElements` + zoom LOD (title-only < 0.5, columns >= 0.5)
- `bench/run-bench.js` — CDP runner: launches headless Chrome, waits for `BENCH_READY`, extracts `window.__bench` + screenshots

Protocol (identical on both): fCoSE/dagre layout → initial DOM count → pan @ overview (110 frames) → zoom to 1.0 → DOM count + visible nodes/edges at detail → pan @ detail (110 frames). Long tasks via `PerformanceObserver('longtask')` from page load.

Environment: Chrome 150 headless, software rendering, 1600x1000 viewport. 3 runs (run 1 parallel, runs 2-3 serialized).

| Metric | A · Cytoscape + fCoSE | B · React Flow v12 | Ratio (B/A) |
|---|---|---|---|
| Layout (main-thread block) | 1,889–2,596 ms | 10,747–17,259 ms | ~5–7x |
| DOM elements, initial (fit view) | **29** | **6,141** | **~212x** |
| DOM elements, detail zoom | 29 | 439 | ~15x |
| Pan FPS, overview | 21.6–52.5 | 22.9–30.8 | — (headless variance) |
| Pan FPS, detail zoom | 21.5–60.0 | 60.0 (locked) | B wins (culling) |
| Long-task total | 3.0–6.3 s | 11.6–18.4 s | ~3x |
| Long-task max (layout) | 2.8–3.0 s | 10.7–17.3 s | ~4–6x |
| Heap used | 22–38 MB | 40–58 MB | ~1.5–2x |

### Interpretation (evidence-driven)

1. **DOM-count root cause confirmed.** Cytoscape holds ~29 DOM elements for the whole 500-table graph; React Flow renders ~6,141 (all nodes + 900 SVG edges re-rendered per pan frame at overview zoom). ~212x more DOM to update per frame is the core difference behind the original screenshot paradox.
2. **Layout is a worse bottleneck than expected for React Flow.** dagre on 500 tables blocks the main thread 10.7–17.3 s in this environment (vs fCoSE 1.9–2.6 s). Layout must move off-thread (P1.7) regardless of renderer.
3. **P1.2–P1.4 culling + LOD demonstrably work:** at detail zoom React Flow renders only ~4 nodes / ~74 edges (439 DOM) and locks 60 fps, exactly the locked invariant `renderedTables ≈ viewportTables + overscan`.
4. **Overview pan is React Flow's weak point** (all nodes visible at fit-view ⇒ no culling win ⇒ 6,141 DOM per frame ⇒ 22–31 fps in software rendering). This is precisely the workload Cytoscape's canvas is built for.
5. **Decision input for P1.9:** the hybrid the user locked (Cytoscape canvas for overview/exploration + React DOM for selected-table detail) is supported by this evidence; React Flow remains fine for small/medium schemas and detail-zoom interaction.

Screenshots: `bench/er-cytoscape-500.png`, `bench/er-reactflow-500.png`.

### Caveats

- **DOM counts are the robust claim** (stable across all runs: 29 vs 6,141). Layout numbers are directional: both layouts run at cold-start right after ~500 KB of vendor scripts parse (dagre at top-level script, fCoSE inside the deferred `run()`), so absolute layout/long-task figures are inflated versus a warm page — the ~5-7x ratio is not precise, but the direction (dagre 500 tables blocks the main thread for many seconds in this environment) stands and motivates P1.7 (layout off-thread) regardless of renderer.
- **Frame-sampling asymmetry**: Cytoscape `pan()` is synchronous, React `setViewport()` renders asynchronously (its commit may land one frame after the sample window) — React frame costs may be slightly under-reported. Overview pan (22-31 fps) shows the cost is still captured; detail pan hitting 60 fps is genuine culling/LOD headroom.
- **Scope**: fixture B (500 tables) only. Fixtures A (100) and C (1,000) pending; 1,000 tables is where DOM-vs-canvas divergence is largest and should be run before a final P1.9 call.

## Runtime evidence log

| Date | Phase | Evidence | Result |
|---|---|---|---|
| 2026-08-12 | P1.8 | A/B harness, 500 tables (see section above) | Cytoscape: 29 DOM / 2.5s layout · React Flow: 6,141 DOM / 10.7–17.3s layout block |

## Provider matrix

This program is frontend-only. No database provider changes.

| Provider | Impact | Notes |
|---|---|---|
| PostgreSQL | None | Schema introspection unchanged |
| SQLite | None | Schema introspection unchanged |
