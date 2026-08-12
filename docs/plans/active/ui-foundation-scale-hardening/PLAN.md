# P3 — UI Foundation & Scale Hardening

## Goal

Establish a single-source-of-truth design token architecture and make the ER diagram usable at production schema sizes (500+ tables). This is a **P1 release prerequisite** — a database IDE that only works on small fixtures is not shippable.

## Non-goals

- Rewriting the ER diagram from scratch (React Flow stays)
- Replacing Dagre with a different layout engine in Phase A (may revisit in Phase C if needed)
- Changing the visual design / look of the app
- Adding new features

## Scope

### P3.1 — Design Token Contract

**Problem:** Two parallel token vocabularies coexist in `globals.css`:
- shadcn semantic tokens: `--background`, `--card`, `--popover`, `--primary`, `--border`, `--muted-foreground`, etc.
- DB Pro app tokens: `--app-surface-*`, `--app-text-*`, `--app-border-*`, `--app-primary-*`, etc.

`components.json` writes directly to `globals.css` with `cssVariables: true`. Every `npx shadcn add` can silently reintroduce shadcn-native assumptions.

**Current evidence** *(pre-migration, 2026-08-12 — post-migration state in VERIFICATION.md P3.1):*
- 476 occurrences of `--app-*` tokens across 90 files (now 0 — canonical `--surface-*`/`--text-*`/`--border-*`/`--accent-*`/`--state-*`/`--elevation-*` layer)
- 112 occurrences of shadcn utility classes (`bg-background`, `bg-popover`, `text-muted-foreground`, etc.) across 57 files (kept — they consume the alias layer)
- shadcn `:root` and `[data-theme="dark"]` blocks define raw color values independently of `--app-*` tokens (fixed — alias-only compat layer)

**Target architecture:**

```
Primitive tokens (raw values)
        ↓
Semantic tokens (DB Pro source of truth)
        ↓
shadcn compatibility aliases (mapping layer)
        ↓
Components (consume either vocabulary, same values)
```

**Locked decision (2026-08-12):** the canonical semantic layer is RENAMED off the `--app-*` prefix onto role-based families — `--surface-*` (backgrounds incl. hover/active), `--text-*` (primary/secondary/tertiary), `--border-*` (subtle/default/strong), `--accent-*` (brand + hover + soft + foreground), `--state-*` (success/warning/danger/info), `--elevation-*` (shadows). The `--app-*` prefix is reserved for layout metrics only. shadcn tokens (`--background`, `--popover`, `--border`, etc.) become **aliases** that point to the canonical values, never independent definitions. shadcn's `--accent` hover-highlight role is mapped at the `@theme inline` layer (`--color-accent: var(--surface-active)`), freeing bare `--accent` for the canonical brand family without changing what `bg-accent` components resolve to.

```css
/* Canonical */
--surface-app: #f0f2f5;
--surface-editor: #ffffff;
--surface-floating: #ffffff;
--text-primary: #0f172a;
--text-secondary: #475569;
--border-default: rgba(0, 0, 0, 0.12);

/* shadcn compatibility — derived, never independently defined */
--background: var(--surface-app);
--popover: var(--surface-floating);
--foreground: var(--text-primary);
--muted-foreground: var(--text-secondary);
--border: var(--border-default);
```

**Acceptance criteria:**
- [x] Single set of primitive color values per theme (light/dark), defined once — canonical layer in `globals.css`, value snapshot pinned by `check-token-drift.mjs`
- [x] shadcn tokens are aliases to canonical tokens, not independent values — alias-only compat layer, enforced by CI
- [x] `npx shadcn add` does not break the token contract (documented guard) — rule in `globals.css` header + AGENTS.md + `npm run check:tokens` in CI
- [x] No visual regression across the app — 96/96 resolved-value pairs identical (bench/token-contract.html, light + dark)

### P3.2 — shadcn Integration Safety

**Problem:** No guard prevents `npx shadcn add` from modifying `globals.css` and reintroducing independent token definitions.

**Target:**
- Document the token contract in a way that survives `shadcn add`
- Add a pre-commit or CI check that detects unauthorized changes to the token layer in `globals.css`
- Alternatively, split shadcn's generated CSS into a separate file that is explicitly reviewed

**Acceptance criteria:**
- [x] Documented rule: "shadcn add must not modify semantic token definitions" — token-contract header in `globals.css` + AGENTS.md
- [x] Mechanism (script, CI check, or file split) that catches token drift — `scripts/check-token-drift.mjs` (5 checks) + `npm run check:tokens` in `.github/workflows/ci.yml`

### P3.3 — ER Diagram Algorithm (Phase A)

**Problem:** `er-diagram.tsx` lines 91-98 run `data.columns.filter(...)` and `data.primaryKeys.filter(...)` for every table. With T=500 tables and C=8,000 columns, this is O(T x C).

**Target:** Pre-index metadata once, then look up per table in O(1).

```ts
const columnsByTable = useMemo(() => {
  const map = new Map<string, Column[]>();
  for (const col of data.columns) {
    const key = `${col.schema}.${col.tableName}`;
    if (!map.has(key)) map.set(key, []);
    map.get(key)!.push(col);
  }
  return map;
}, [data.columns]);
```

**Acceptance criteria:**
- [ ] No `.filter()` on full `data.columns` / `data.primaryKeys` / `data.foreignKeys` inside per-table loops
- [ ] Pre-indexed maps for columns, PKs, FKs by `schema.tableName`
- [ ] Existing tests pass

### P3.4 — ER Diagram Duplicate Layout Elimination (Phase A)

**Problem:** `layoutGraph()` is called twice:
1. `useMemo` at line 149 — computes `laidOutNodes`
2. `useEffect` at line 163 — re-computes the same layout and calls `setNodes()`

This means Dagre runs twice for every dependency change.

**Target:** Single layout computation. The `useEffect` should only handle edge highlighting (selectedEdgeId), not re-run layout.

**Acceptance criteria:**
- [ ] `layoutGraph()` called exactly once per layout-relevant change
- [ ] Edge highlighting does not trigger re-layout
- [ ] Manual position override logic preserved

### P3.5 — ER Diagram Rendering LOD (Phase B)

**Problem:** 500 detailed table cards with full column lists render simultaneously. No level-of-detail based on zoom or graph size.

**Target:** Zoom-aware rendering tiers:

| Zoom level | Render |
|---|---|
| Very zoomed out (< 0.3) | Table name only, no columns |
| Medium (0.3 - 0.7) | Table name + column count |
| Zoomed in (> 0.7) | Full column list |
| Any (selected) | Full column list for selected + 1-hop neighbors |

Implementation approach:
- Use React Flow's `useViewport()` hook to read current zoom level
- Pass zoom tier to `TableNode` via node data or context
- `TableNode` conditionally renders based on tier

**Acceptance criteria:**
- [ ] At low zoom, column rows are not rendered (DOM nodes eliminated)
- [ ] At high zoom or when selected, full detail is shown
- [ ] No visual jank when zooming between tiers (smooth transition)

### P3.6 — ER Diagram Large Schema Mode (Phase D)

**Problem:** For schemas with 200+ tables, showing everything at once is not useful.

**Target:** Neighborhood mode — search for a table, show only that table + 1-hop/2-hop related tables.

```
Search: orders

           users
             |
payments - orders - order_items
             |
          shipments
```

**Acceptance criteria:**
- [ ] When table count > threshold (200), default to search-first view
- [ ] Search result shows selected table + configurable hop radius
- [ ] "Show all N tables" available as explicit user action
- [ ] Neighborhood computation is O(hop x avg_fk_per_table), not O(all_tables)

### P3.7 — Performance Budgets

**Target:** Define and enforce measurable budgets:

| Metric | Budget (500 tables) |
|---|---|
| Open workspace → UI usable | < 1s |
| Initial diagram paint | < 500ms |
| Search response | < 100ms |
| Pan/zoom | No main thread freeze (> 16ms frame) |
| Selection | < 50ms |
| Memory | No unbounded growth |

**Acceptance criteria:**
- [ ] Budgets documented in VERIFICATION.md
- [ ] Benchmark fixtures exist for small (20), medium (100), large (500), xlarge (1000) tables
- [ ] At least one automated performance test that fails if budget is violated

### P3.8 — Data Grid / Metadata List Audit

**Problem:** Other list views (schema explorer, data grid, connection list) may have similar O(N) scan patterns.

**Target:** Audit all list/grid components that iterate over schema metadata. Fix any that scan full arrays per-item.

**Acceptance criteria:**
- [ ] Audit completed and documented in FINDINGS.md
- [ ] P0/P1 issues fixed
- [ ] P2 issues tracked

## Execution order

```
Phase 1: Algorithm fixes (P3.3, P3.4)
  → Immediate perf gain, no visual change, low risk

Phase 2: Design token contract (P3.1, P3.2)
  → Foundation for safe future development

Phase 3: Rendering LOD (P3.5)
  → Major perf gain for large schemas

Phase 4: Large schema mode (P3.6)
  → New UX paradigm for 200+ table schemas

Phase 5: Audit + budgets (P3.7, P3.8)
  → Regression prevention
```

## Risk

- Token migration (P3.1) touches 90+ files — must be done incrementally with visual verification at each step
- LOD zoom tiers need tuning — initial thresholds are hypotheses, must be validated with real schemas
- Neighborhood mode is new UX — needs user validation before committing to the interaction model

---

# P1 — Large-Schema ER Rendering Architecture (locked)

## Goal

Make the ER diagram scale past 500 tables. P3.3–P3.8 (pre-index, layout dedup, CSS-tier LOD, neighborhood mode) fixed the O(T×C) algorithm path but **not** the render path: 500 detailed table cards + thousands of edges still enter React reconciliation and frame-drop during pan/zoom.

## Locked architecture (decision, not proposal)

```text
                      ER DIAGRAM CORE
┌───────────────────────────────────────────────────┐
│ ErGraphModel                                      │
│   tables / relations / adjacency index            │
│   degree / centrality metadata / schema statistics│
└───────────────────────┬───────────────────────────┘
                        │
                        ▼
┌───────────────────────────────────────────────────┐
│ Layout Engine                                     │
│   Web Worker · Dagre / future ELK adapter         │
│   layout cache · schemaHash → positions           │
└───────────────────────┬───────────────────────────┘
                        │
                        ▼
┌───────────────────────────────────────────────────┐
│ Spatial Index                                     │
│   node bounding boxes · viewport intersection     │
│   visible node IDs · visible edge IDs             │
└───────────────────────┬───────────────────────────┘
                        │
                        ▼
┌───────────────────────────────────────────────────┐
│ Viewport Engine                                   │
│   pan / zoom · LOD resolver · neighborhood        │
│   selected / hovered · search focus               │
└───────────────┬──────────────────┬────────────────┘
                │                  │
                ▼                  ▼
      ReactFlow Renderer      Future Canvas
        small/medium           very large
```

Renderer abstraction from day one — domain/application layers never know which renderer is mounted:

```ts
interface ErRenderer {
  mount(model: ErRenderModel): void;
  updateViewport(viewport: ErViewport): void;
  updateSelection(selection: ErSelection): void;
  focusNode(nodeId: TableId): void;
  dispose(): void;
}
// ReactFlowErRenderer · CanvasErRenderer (decided by P1.9 benchmark)
```

### Hard rules (locked)

1. **`onlyRenderVisibleElements` is a mitigation, not the answer.** End state is spatial-query → ~20–40 viewport nodes → LOD → render. Never feed all 500 nodes into React and hope React Flow saves us.
2. **LOD changes the render tree, not just CSS.** `switch (lod) { dot → compact → summary → detail }` returning distinct components (`ErDotNode` / `ErCompactNode` / `ErSummaryNode` / `ErDetailedNode`). CSS `hidden` that keeps DOM alive is forbidden.
3. **Edges have LOD too.** zoom < 0.25: no FK labels, no markers, straight/simple lines, aggregate relations (e.g. `Customer ─31─ Order`); 0.25–0.6: simple relation edges; > 0.6: normal FK edges; selected neighborhood: full relation detail.
4. **Dagre runs fully in a Worker, atomically committed.** No fake progressive layout — no streaming un-stable chunk positions to the UI. UI may be progressive (shell → search usable → "Arranging 512 tables…" → commit once), but positions never appear until stable.
5. **Thresholds are complexity scores, not hardcoded counts.** `complexity = tableCount + relationCount * 0.7 + totalColumnCount * 0.08`; classify S (<100) / M (100–300) / L (300–700) / XL (>700). Real thresholds tuned from runtime benchmarks.
6. **Full schema is not the default experience for large schemas.** Default: search-first exploration — `[1 hop] [2 hops] [Domain] [All 512 tables]` — database exploration, not diagram viewer.

## Acceptance criteria (P1)

**Finding:** `P1 — Large-schema ER rendering architecture does not scale`.

QA fixtures:

| Fixture | Tables | Relations | Columns |
|---|---|---|---|
| A | 100 | ~150 | ~1,200 |
| B | 500 | ~900 | ~7,500 |
| C | 1,000 | ~2,000 | ~15,000 |

Metrics (recorded per fixture):

```text
Open diagram
├─ time to interactive shell
├─ layout worker duration
├─ max main-thread long task
└─ initial detailed DOM count

Interaction
├─ pan frame time
├─ zoom frame time
├─ search latency
└─ selection latency

Renderer
├─ graph node count
├─ viewport node count
├─ rendered node count
├─ detailed node count
└─ rendered edge count
```

Invariants (must hold, never: `graphTables = renderedTables = detailedTables`):

```text
graphTables = 512
viewportTables ≈ 26
renderedTables ≈ 26 + overscan
detailedTables <= visibleNearZoom + selected
```

## Implementation order (locked)

```text
P1.1  Runtime instrumentation                    ← this session
P1.2  onlyRenderVisibleElements + MiniMap policy ← this session
P1.3  true LOD components (render-tree switch)
P1.4  edge LOD
P1.5  viewport / spatial-index layer
P1.6  default neighborhood exploration UX
P1.7  move layout to Worker + cache
P1.8  benchmark 100 / 500 / 1000 tables
P1.9  decide whether Canvas renderer is necessary
```

No Canvas/WebGL rewrite until P1.1–P1.7 are implemented and benchmarked. If React Flow + real viewport culling + true LOD passes the budget at 500/1000 tables, `CanvasErRenderer` stays a future option.

## Non-goals (locked)

- Rewriting the renderer before P1.8 benchmarks exist
- Fake progressive layout (streaming unstable positions)
- CSS-only LOD that keeps hidden DOM alive
