# Performance Budgets Reference

Complete budget table with rationale and measurement methodology.

## Frontend Budgets

### Bundle Size

| Category | Target | Critical | Measurement |
|----------|--------|----------|-------------|
| Total JS (gzipped) | < 500 KB | > 1 MB | `gzip -c dist/assets/*.js \| wc -c` |
| Total JS (uncompressed) | < 1.5 MB | > 2 MB | `du -sb dist/assets/*.js` |
| Largest single chunk | < 500 KB | > 800 KB | `ls -lS dist/assets/*.js \| head -1` |
| CSS total | < 100 KB | > 200 KB | `du -sb dist/assets/*.css` |
| Initial load (critical path) | < 300 KB | > 500 KB | Sum of chunks for first route |

### Runtime Performance

| Operation | Target | Critical | Test file |
|-----------|--------|----------|-----------|
| Quick Open index (1k items) | < 50ms | > 100ms | `performance-budgets.test.ts` |
| Quick Open rank (1k items) | < 50ms | > 100ms | `performance-budgets.test.ts` |
| Statement split (100 stmts) | < 50ms | > 100ms | `performance-budgets.test.ts` |
| CSV generate (10k rows) | < 200ms | > 500ms | `performance-budgets.test.ts` |
| JSON generate (10k rows) | < 200ms | > 500ms | `performance-budgets.test.ts` |
| SQL INSERT generate (1k rows) | < 200ms | > 500ms | `performance-budgets.test.ts` |
| CSV parse (10k rows) | < 200ms | > 500ms | `performance-budgets.test.ts` |
| Schema tree (500 tables) | < 150ms | > 300ms | `performance-budgets.test.ts` |
| Workspace tab cycle (100) | < 100ms | > 200ms | `performance-budgets.test.ts` |
| Grid state ops (100) | < 50ms | > 100ms | `performance-budgets.test.ts` |

### ER Diagram (P1 series)

| Metric | 200 tables | 500 tables | 1000 tables | Measurement |
|--------|------------|------------|-------------|-------------|
| Time to interactive | < 2s | < 5s | < 10s | `er-perf-hud.tsx` init time |
| Layout computation | < 500ms | < 1.5s | < 3s | `er-perf-hud.tsx` layout time |
| Max long task | < 100ms | < 150ms | < 200ms | `er-perf-hud.tsx` long tasks |
| Frame avg (pan/zoom) | < 8ms | < 12ms | < 16ms | `er-perf-hud.tsx` frame stats |
| Frame p95 (pan/zoom) | < 16ms | < 24ms | < 33ms | `er-perf-hud.tsx` frame stats |

## Backend Budgets (Rust/Criterion)

| Operation | Target | Critical | Benchmark |
|-----------|--------|----------|-----------|
| SQLite connect/disconnect | < 100ms | > 500ms | `sqlite_connect_and_disconnect` |
| Introspect small (5 tables) | < 300ms | > 1s | `introspect_small_db` |
| Introspect large (50×20) | < 300ms | > 1s | `introspect_large_schema` |
| Query 10k rows | < 150ms | > 500ms | `query_rows/select_10k` |
| Query 100k rows | < 500ms | > 2s | `query_rows/select_100k` |
| JSON blob (5k rows) | < 100ms | > 300ms | `query_json_blob` |
| Large text (1k rows × 4KB) | < 200ms | > 500ms | `serialize_large_text` |
| Cancel acknowledgement | < 200ms | > 500ms | Execution registry |

## Database Query Budgets

| Query type | Target | Critical | Notes |
|------------|--------|----------|-------|
| Simple SELECT (indexed) | < 10ms | > 50ms | Single table, WHERE on indexed col |
| JOIN (2-3 tables) | < 50ms | > 200ms | With proper indexes |
| Aggregation (COUNT/SUM) | < 100ms | > 500ms | On indexed columns |
| Full table scan | < 1s | > 5s | Only acceptable for small tables |
| Schema introspection | < 300ms | > 1s | All tables, columns, constraints |

## Measurement Methodology

### Frontend runtime

- Use `performance.now()` for sub-millisecond precision
- Run benchmarks 3x and take median
- Test on mid-range hardware (4 cores, 8GB RAM)
- Disable browser extensions during measurement

### Rust benchmarks

- Use Criterion.rs with `--quick` for iteration, full run for baseline
- Run on dedicated machine (no other CPU-intensive tasks)
- Warm up: 3 iterations, measure: 10 iterations minimum
- Report mean time with 95% confidence interval

### Database queries

- Use `EXPLAIN ANALYZE` for actual execution time
- Run 5x and take median (cold cache first, then warm cache)
- Measure at application layer (includes network/IPC)
- Test with realistic data volumes (10k-100k rows)

## Severity Levels

- **Target**: Acceptable performance for production use
- **Critical**: Immediate action required, user-visible degradation
- **P0**: Application unusable or data loss risk
- **P1**: Noticeable slowdown, affects workflow
- **P2**: Minor degradation, acceptable for now
