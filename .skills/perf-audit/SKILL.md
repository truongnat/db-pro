---
name: perf-audit
description: "Performance scanning and evaluation for Tauri + React + Rust applications. Use when the user asks to audit performance, check bundle size, profile rendering, run benchmarks, evaluate ER diagram performance, measure query latency, enforce performance budgets, or detect performance regressions. Triggers on: /perf-audit, performance audit, perf scan, bundle analysis, render profiling, benchmark run, performance budget check."
---

# Performance Audit

Scan, measure, and evaluate performance across the full stack: frontend bundle, React rendering, ER diagram, Rust backend, and database queries.

## Quick Start

Run the full audit script to collect all metrics at once:

```bash
bash ~/.qoder/skills/perf-audit/scripts/perf-scan.sh
```

Or run individual audits by category below.

## 1. Frontend Bundle Analysis

### Bundle size check

```bash
cd frontend
npm run build
ls -lh dist/assets/*.js | sort -k5 -h
```

### Analyze with source-map-explorer (one-time install)

```bash
npx source-map-explorer dist/assets/*.js --no-border-checks
```

### Budget targets

| Chunk | Target | Action threshold |
|-------|--------|------------------|
| Total JS | < 1.5 MB | > 2 MB = P1 |
| Single chunk | < 500 KB | > 800 KB = P1 |
| vendor-tanstack | < 300 KB | > 400 KB = investigate |
| vendor-ui | < 200 KB | > 300 KB = tree-shake audit |

### What to check

- Unused exports via `npx ts-prune` or `npx knip`
- Dynamic imports for heavy modules (SQL parser, export generators)
- Duplicate dependencies: `npm ls --all | sort | uniq -d`

## 2. React Rendering Performance

### Run performance budget tests

```bash
cd frontend
npm run test -- --run src/commons/__tests__/performance-budgets.test.ts
```

### Budget targets (from performance-budgets.test.ts)

| Operation | Budget (ms) |
|-----------|-------------|
| Quick Open index (1000 items) | < 50 |
| Quick Open rank (1000 items) | < 50 |
| Statement split (100 statements) | < 50 |
| CSV generate (10k rows) | < 200 |
| JSON generate (10k rows) | < 200 |
| SQL INSERT generate (1k rows) | < 200 |
| CSV parse (10k rows) | < 200 |
| Schema tree (500 tables) | < 150 |
| Workspace tab cycle (100 tabs) | < 100 |
| Grid state ops (100) | < 50 |

### React Profiler (manual)

1. Open DevTools > React DevTools > Profiler
2. Enable "Record why each render occurred"
3. Perform the action under test
4. Look for:
   - Renders > 16ms (missed frame budget)
   - Unnecessary re-renders (no props/state change)
   - Missing `memo`, `useMemo`, `useCallback` on hot paths

### Key patterns to verify

- Virtual lists for > 100 items (TanStack Virtual)
- `React.memo` on table/grid row components
- No inline object/array literals in render props
- Stable refs for callbacks passed to children

## 3. ER Diagram Performance

### Enable the perf HUD

```javascript
// In browser console:
localStorage.setItem("er-perf-hud", "1");
// Reload the page
```

### Acceptance metrics (P1 series)

| Metric | Target (200 tables) | Target (500 tables) | Target (1000 tables) |
|--------|---------------------|---------------------|----------------------|
| Time to interactive | < 2s | < 5s | < 10s |
| Layout computation | < 500ms | < 1.5s | < 3s |
| Max long task | < 100ms | < 150ms | < 200ms |
| Pan/zoom frame avg | < 8ms | < 12ms | < 16ms |
| Pan/zoom frame p95 | < 16ms | < 24ms | < 33ms |

### What to check

- Spatial index: viewport query must be O(visible) not O(total)
- LOD tiers: detail level drops at zoom < 0.5
- Edge aggregation: parallel edges merge at low zoom
- Worker offload: layout runs in Web Worker for > 100 tables
- Canvas renderer: kicks in for > 200 tables instead of DOM nodes

### Regression detection

Compare HUD metrics before/after a change. Any metric > 20% slower = investigate.

## 4. Rust Backend Benchmarks

### Run benchmarks

```bash
cargo bench --package db-pro-infrastructure
```

### Budget targets (Criterion)

| Operation | Budget |
|-----------|--------|
| SQLite connect/disconnect | < 100 ms |
| Introspect small DB (5 tables) | < 300 ms |
| Introspect large schema (50×20) | < 300 ms |
| Query 10k rows | < 150 ms |
| Cancel acknowledgement | < 200 ms |

### Profile with flamegraph (one-time install)

```bash
cargo install flamegraph
cargo flamegraph --bench sqlite_benchmarks -- --bench
```

### What to check

- N+1 queries in introspection (should batch)
- Unnecessary clones of large schema structs
- `Vec` allocations in hot loops (pre-allocate with `with_capacity`)
- Serde serialization overhead for IPC boundary

## 5. Database Query Performance

### EXPLAIN ANALYZE via query editor

```sql
EXPLAIN ANALYZE SELECT * FROM your_table WHERE condition;
```

### Check for missing indexes

```sql
-- PostgreSQL: sequential scans on large tables
SELECT schemaname, relname, seq_scan, seq_tup_read
FROM pg_stat_user_tables
WHERE seq_scan > 100 AND seq_tup_read > 10000
ORDER BY seq_tup_read DESC;

-- SQLite: query plan
EXPLAIN QUERY PLAN SELECT * FROM your_table WHERE condition;
```

### Connection pool metrics

Check Tauri backend logs for:
- Connection acquire time > 50ms
- Pool exhaustion warnings
- Idle connection count

## 6. Performance Regression Workflow

When investigating a reported regression:

1. **Reproduce**: Confirm the regression with the perf HUD or benchmark
2. **Isolate**: `git bisect` to find the introducing commit
3. **Measure**: Run the specific benchmark/budget test before and after
4. **Fix**: Address root cause, not symptoms
5. **Verify**: Re-run the audit, confirm metrics are within budget
6. **Document**: Update `references/perf-budgets.md` if budgets change

## 7. CI Performance Gates

Recommended CI checks (add to workflow):

```yaml
- name: Performance budgets
  run: cd frontend && npm run test -- --run performance-budgets

- name: Bundle size check
  run: |
    cd frontend && npm run build
    TOTAL=$(du -sb dist/assets/*.js | awk '{sum+=$1} END {print sum}')
    if [ "$TOTAL" -gt 2000000 ]; then
      echo "Bundle size ${TOTAL}B exceeds 2MB limit"
      exit 1
    fi

- name: Rust benchmarks
  run: cargo bench --package db-pro-infrastructure -- --quick
```

## Resources

- `references/perf-budgets.md` — Complete budget table with rationale
- `scripts/perf-scan.sh` — Automated full-stack audit script
