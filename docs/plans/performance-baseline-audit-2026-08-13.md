# Performance Baseline Audit Report

**Date**: 2026-08-13  
**Branch**: main  
**Status**: Completed

## Executive Summary

Frontend bundle size exceeds critical threshold. Performance budget tests pass. Rust backend clean.

## Results

### 1. Frontend Bundle Analysis

**Status**: ⚠️ WARNING — Total JS size exceeds 2MB critical threshold

| Metric | Value | Target | Status |
|--------|-------|--------|--------|
| Total JS (uncompressed) | 2.24 MB | < 1.5 MB | ❌ FAIL |
| Largest chunk (index-Cy2GYGpp.js) | 1,182.90 KB | < 500 KB | ❌ FAIL |
| Second largest (cytoscape-view) | 445.18 KB | < 500 KB | ✅ PASS |
| vendor-tanstack | 156.53 KB | < 300 KB | ✅ PASS |
| vendor-ui | 127.50 KB | < 200 KB | ✅ PASS |
| Total CSS | 125.70 KB | < 100 KB | ⚠️ WARN |
| JS chunk count | 6 | — | ✅ OK |

**Critical Issues**:
- `index-Cy2GYGpp.js` (1.18 MB) contains too much code in a single chunk
- `cytoscape-view-BQlAKhFJ.js` (445 KB) is acceptable but close to threshold
- Total bundle 2.24 MB exceeds 2MB critical threshold

**Recommendations**:
1. Code-split `index-Cy2GYGpp.js` — identify large dependencies and lazy-load
2. Dynamic import for heavy features (ER diagram, export, query editor)
3. Tree-shake audit: run `npx knip` to find unused exports
4. Consider splitting cytoscape into separate chunk if not needed on initial load

### 2. Frontend Performance Budget Tests

**Status**: ✅ PASS — All 8 tests passed

| Test | Budget | Result |
|------|--------|--------|
| Quick Open index (1k items) | < 50ms | ✅ PASS |
| Quick Open rank (1k items) | < 50ms | ✅ PASS |
| Statement split (100 stmts) | < 50ms | ✅ PASS |
| CSV generate (10k rows) | < 200ms | ✅ PASS |
| JSON generate (10k rows) | < 200ms | ✅ PASS |
| SQL INSERT generate (1k rows) | < 200ms | ✅ PASS |
| CSV parse (10k rows) | < 200ms | ✅ PASS |
| Schema tree (500 tables) | < 150ms | ✅ PASS |

**Test execution time**: 104ms total, 875ms including setup

### 3. Rust Backend

**Status**: ✅ PASS — Clean compilation, no warnings

| Check | Result |
|-------|--------|
| cargo check --workspace | ✅ PASS |
| cargo clippy --workspace --all-targets | ✅ PASS (0 warnings) |

**Note**: Criterion benchmarks not run in this audit (requires dedicated machine for accurate results).  
Reference baseline: `docs/architecture/performance-baseline.md`

### 4. ER Diagram Performance

**Status**: ⏸️ NOT TESTED — Requires manual HUD inspection

To test:
1. Open the app
2. Run in browser console: `localStorage.setItem("er-perf-hud", "1")`
3. Load schemas of 200/500/1000 tables
4. Record metrics from HUD overlay

**Reference targets**:
- 200 tables: TTI < 2s, layout < 500ms, frame avg < 8ms
- 500 tables: TTI < 5s, layout < 1.5s, frame avg < 12ms
- 1000 tables: TTI < 10s, layout < 3s, frame avg < 16ms

### 5. Database Query Performance

**Status**: ⏸️ NOT TESTED — Requires live database connection

To test:
```sql
EXPLAIN ANALYZE SELECT * FROM your_table WHERE condition;
```

Check for sequential scans:
```sql
SELECT schemaname, relname, seq_scan, seq_tup_read
FROM pg_stat_user_tables
WHERE seq_scan > 100 ORDER BY seq_tup_read DESC;
```

## Severity Summary

| Severity | Count | Issues |
|----------|-------|--------|
| P0 (Critical) | 0 | — |
| P1 (High) | 2 | Bundle size > 2MB, largest chunk > 800KB |
| P2 (Medium) | 1 | CSS size > 100KB |
| P3 (Low) | 0 | — |

## Action Items

### P1 — Must fix before next major release

1. **Code-split index-Cy2GYGpp.js** (1.18 MB)
   - Identify what's bundled in this chunk
   - Move heavy features to dynamic imports
   - Target: < 500 KB per chunk

2. **Reduce total bundle size**
   - Current: 2.24 MB
   - Target: < 1.5 MB
   - Run `npx knip` to find dead code
   - Audit dependencies: `npm ls --all | sort | uniq -d`

### P2 — Should fix

3. **Optimize CSS bundle**
   - Current: 125.70 KB
   - Target: < 100 KB
   - Check for unused Tailwind classes
   - Purge unused CSS

### Recommended next steps

4. Run ER diagram performance test with HUD
5. Run Rust benchmarks on dedicated machine
6. Set up CI performance gates to prevent regression

## Conclusion

Runtime performance (React rendering, data processing) is within budget.  
Bundle size is the primary concern — requires code-splitting before production deployment.

**Overall Status**: ⚠️ WARNING — Functional but needs optimization
