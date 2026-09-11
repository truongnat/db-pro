# Performance Budgets Reference

Complete budget table with rationale and measurement methodology.

The UI is native `eframe`/`egui` (`crates/ui` + `crates/native-app`). There is no JS or CSS
bundle to budget; the React frontend was archived under `_archive/frontend/` on 2026-09-11.

## Native UI Budgets

### Binary Size

| Category | Target | Critical | Measurement |
|----------|--------|----------|-------------|
| `db-pro-native` (release) | < 50 MB | > 100 MB | `ls -l target/release/db-pro-native` |
| Stripped distribution binary | < 30 MB | > 60 MB | `strip target/release/db-pro-native` then re-measure |

### Runtime Performance (criterion: `crates/ui/benches/result_grid_benchmarks.rs`)

| Operation | Target | Critical | Benchmark |
|-----------|--------|----------|-----------|
| Grid visible-range computation | < 1ms | > 5ms | `result_grid_benchmarks` |
| Grid hit-testing | < 1ms | > 5ms | `result_grid_benchmarks` |
| Cell codec round-trip | < 1ms | > 5ms | `result_grid_benchmarks` |
| Quick Open index (1k items) | < 5ms | > 20ms | reducer bench |
| Statement split (100 stmts) | < 5ms | > 20ms | reducer bench |

### Frame Time (manual, egui)

| Surface | Target | Critical | Measurement |
|---------|--------|----------|-------------|
| Shell idle frame | < 8ms | > 16ms | `RUST_LOG=db_pro_ui=debug` frame log |
| Grid scroll frame avg (100k rows) | < 8ms | > 16ms | frame log while scrolling |
| Grid scroll frame p95 | < 16ms | > 33ms | frame log |
| Cancellation ack → UI shows stopped | < 250ms | > 500ms | manual timing after Stop |

### ER Diagram

| Metric | 200 tables | 500 tables | 1000 tables | Measurement |
|--------|------------|------------|-------------|-------------|
| Time to interactive | < 2s | < 5s | < 10s | `db_pro_ui=debug` layout + first-frame log |
| Layout computation | < 500ms | < 1.5s | < 3s | layout log |
| Frame avg (pan/zoom) | < 8ms | < 12ms | < 16ms | frame log |
| Frame p95 (pan/zoom) | < 16ms | < 24ms | < 33ms | frame log |

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

### Native UI runtime

- Use the `db_pro_ui` debug frame log; take the median of at least 3 runs
- Test on mid-range hardware (4 cores, 8GB RAM)
- Close other CPU-intensive applications during measurement
- Measure with realistic data: 100k rows in the grid, 200+ tables in the ER diagram
- Remember egui is immediate-mode: a per-frame cost that looks small can still be a
  correctness bug if it recomputes derived state instead of caching it in `AppState`

### Rust benchmarks

- Use Criterion.rs with `--quick` for iteration, full run for baseline
- Run on a dedicated machine (no other CPU-intensive tasks)
- Warm up: 3 iterations, measure: 10 iterations minimum
- Report mean time with 95% confidence interval

### Database queries

- Use `EXPLAIN ANALYZE` for actual execution time
- Run 5x and take median (cold cache first, then warm cache)
- Measure at the application layer (includes the runtime task bridge)
- Test with realistic data volumes (10k-100k rows)

## Severity Levels

- **Target**: Acceptable performance for production use
- **Critical**: Immediate action required, user-visible degradation
- **P0**: Application unusable or data loss risk
- **P1**: Noticeable slowdown, affects workflow
- **P2**: Minor degradation, acceptable for now
