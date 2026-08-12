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

### Caveats

- **DOM counts are the robust claim** (stable across all runs/cells). Layout numbers are directional: both layouts run at cold-start right after vendor scripts parse, so absolute figures are inflated versus a warm page — but the scaling trend (fCoSE 0.2→36 s, dagre 0.15→122 s) and the conclusion (layout off-thread is mandatory) hold.
- **Frame-sampling asymmetry**: Cytoscape `pan()` is synchronous, React `setViewport()` renders asynchronously (its commit may land one frame after the sample window) — React frame costs may be slightly under-reported. Overview pan degradation (60→34→18.6 fps) still captures it.
- Software rendering only (headless). GPU-backed canvas painting would favor Cytoscape's repaint-heavy path on real hardware; DOM transform cost for React Flow is GPU-agnostic.

## Runtime evidence log

| Date | Phase | Evidence | Result |
|---|---|---|---|
| 2026-08-12 | P1.8 | A/B harness full matrix 100/500/1000 (see section above) | Cytoscape: 31 DOM at all scales, layout 0.2/2/36 s · React Flow: 1,143/6,143/11,817 DOM, layout 0.15/8/122 s, overview pan 60/34/18.6 fps |

## Provider matrix

This program is frontend-only. No database provider changes.

| Provider | Impact | Notes |
|---|---|---|
| PostgreSQL | None | Schema introspection unchanged |
| SQLite | None | Schema introspection unchanged |
