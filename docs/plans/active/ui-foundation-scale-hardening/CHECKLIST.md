# P3 — UI Foundation & Scale Hardening — Checklist

## Phase 1: Algorithm Fixes

- [x] **1.1** Pre-index `data.columns` by `schema.tableName` in `er-diagram.tsx`
- [x] **1.2** Pre-index `data.primaryKeys` by `schema.tableName`
- [x] **1.3** Pre-index `data.foreignKeys` by `schema.tableName` (from/to)
- [x] **1.4** Replace per-table `.filter()` calls with map lookups
- [x] **1.5** Eliminate duplicate `layoutGraph()` call — remove from `useEffect`, keep only in `useMemo`
- [x] **1.6** Separate edge highlighting logic from layout re-computation
- [x] **1.7** Run existing ER diagram tests — all pass
- [ ] **1.8** Measure layout + build time before/after with 500-table fixture

## Phase 2: Design Token Contract

- [x] **2.1** Define canonical token naming convention (`--surface-*`, `--text-*`, `--border-*`, `--accent-*`, `--state-*`)
- [x] **2.2** Migrate `--app-surface-*` → canonical names (or keep `--app-*` as canonical and document)
- [x] **2.3** Make shadcn tokens (`--background`, `--popover`, etc.) alias to canonical tokens
- [x] **2.4** Remove duplicate color definitions from shadcn `:root` / `[data-theme="dark"]` blocks
- [x] **2.5** Verify no visual regression (light + dark theme)
- [x] **2.6** Document token contract and `shadcn add` safety rule
- [ ] **2.7** Add CI check or script to detect token drift

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

- [ ] **4.1** Detect schema size and select rendering mode (full / compact / overview / large)
- [ ] **4.2** Implement search-first default for schemas > 200 tables
- [ ] **4.3** Implement neighborhood mode: selected table + N-hop FK neighbors
- [ ] **4.4** Add "Show all N tables" explicit action
- [ ] **4.5** Cache layout result — don't re-layout on search/selection changes
- [ ] **4.6** Debounce resize/layout changes
- [ ] **4.7** Consider Web Worker for Dagre layout if still slow at 500+ nodes

## Phase 5: Audit + Budgets

- [ ] **5.1** Audit schema explorer for O(N) scan patterns
- [ ] **5.2** Audit data grid for O(N) scan patterns
- [ ] **5.3** Audit connection list for O(N) scan patterns
- [ ] **5.4** Fix P0/P1 issues found in audit
- [ ] **5.5** Create benchmark fixtures (20, 100, 500, 1000 tables)
- [ ] **5.6** Define performance budgets in VERIFICATION.md
- [ ] **5.7** Add at least one automated performance regression test

## Gates

- [ ] All Phase 1 items complete before merging
- [ ] Phase 2 complete before adding new shadcn components
- [ ] Phase 3+4 complete before v0.1 release
- [ ] P0 = 0, P1 = 0 across all phases
