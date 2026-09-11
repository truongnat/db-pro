---
name: perf-audit
description: "Performance scanning and evaluation for the native Rust + egui DB Pro application. Use when the user asks to audit performance, profile native UI rendering, run Criterion benchmarks, evaluate ER diagram performance, measure query latency, enforce performance budgets, or detect performance regressions. Triggers on: /perf-audit, performance audit, perf scan, render profiling, benchmark run, performance budget check."
---

# Performance Audit

Scan, measure, and evaluate performance across the native stack: egui rendering, the
virtualized grid, ER diagram painting, the Rust runtime, and database queries.

There is **no JS bundle, no React, and no Node toolchain** in this repository. The React
frontend was archived under `_archive/frontend/` on 2026-09-11 and is not a valid audit
target.

## Quick Start

Run the full audit script to collect all metrics at once:

```bash
bash .skills/perf-audit/scripts/perf-scan.sh
```

Or run individual audits by category below.

## 1. Native UI Binary

### Build and size check

```bash
cargo build --release --locked -p db-pro-native
ls -lh target/release/db-pro-native
```

### Budget targets

| Artifact | Target | Action threshold |
|----------|--------|------------------|
| `db-pro-native` binary | < 50 MB | > 100 MB = P1 |

There is no bundle-size metric anymore. A binary-size regression usually means a new heavy
dependency (image codecs, embedded fonts, crypto backends) — check `cargo tree -p db-pro-native`
and `cargo bloat --release -p db-pro-native` before accepting it.

### What to check

- New large dependencies in `crates/ui` / `crates/native-app` Cargo manifests
- Duplicate or unused feature flags pulling extra crates
- Debug symbols not stripped in release profile (only matters for distribution size)

## 2. Native UI Rendering Performance

### Run the UI benchmarks

```bash
cargo bench --package db-pro-ui
```

### Budget targets (from `crates/ui/benches/result_grid_benchmarks.rs`)

| Operation | Budget |
|-----------|--------|
| Grid visible-range computation | < 1 ms |
| Grid hit-testing | < 1 ms |
| Cell formatting / codec round-trip | < 1 ms |
| Quick Open index + rank | < 5 ms |
| Statement split (100 statements) | < 5 ms |

### Frame-time measurement (manual)

egui is immediate-mode, so measure at the frame level rather than with a component profiler:

1. Run with `RUST_LOG=db_pro_ui=debug cargo run -p db-pro-native`
2. Watch the frame-time log for the surface under test
3. Look for:
   - Frame time > 16 ms on the shell or grid
   - Work performed inside the `update()` closure that should be in the reducer
   - Recomputing layout, formatting, or filtering every frame instead of on state change
   - Allocating inside the paint loop (per-row `String`/`Vec` churn)

### Key patterns to verify

- Virtualized painting over the visible range only — never iterate the full row set
- Derived values cached in `AppState`, recomputed on `UiEvent` rather than per frame
- No blocking I/O or `.await` on the UI thread
- Bounded channels and request IDs so stale batches cannot repaint a newer tab
- `ctx.request_repaint()` only while work is pending

## 3. ER Diagram Performance

The ER diagram is a custom `egui::Painter` in `crates/ui/src/diagram_view.rs`.

### Acceptance metrics

| Metric | Target (200 tables) | Target (500 tables) | Target (1000 tables) |
|--------|---------------------|---------------------|----------------------|
| Time to interactive | < 2s | < 5s | < 10s |
| Layout computation | < 500ms | < 1.5s | < 3s |
| Pan/zoom frame avg | < 8ms | < 12ms | < 16ms |
| Pan/zoom frame p95 | < 16ms | < 24ms | < 33ms |

### What to check

- Spatial index: viewport query must be O(visible), not O(total nodes)
- LOD tiers: detail level drops at low zoom
- Edge aggregation: parallel edges merge at low zoom
- Layout runs off the UI thread for > 100 tables
- Neighborhood/search-first default for large schemas (> 200 tables)

### Regression detection

Compare frame-time logs before/after a change. Any metric > 20% slower = investigate.

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
- Unbounded growth of result batches crossing the task bridge

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

### Runtime worker metrics

Check `db-pro-runtime` logs for:
- Connection acquire time > 50ms
- Pool exhaustion warnings
- Cancellation acknowledgement latency

## 6. Performance Regression Workflow

When investigating a reported regression:

1. **Reproduce**: Confirm the regression with the frame-time log or a Criterion benchmark
2. **Isolate**: `git bisect` to find the introducing commit
3. **Measure**: Run the specific benchmark before and after
4. **Fix**: Address root cause, not symptoms
5. **Verify**: Re-run the audit, confirm metrics are within budget
6. **Document**: Update `references/perf-budgets.md` if budgets change

## 7. CI Performance Gates

Recommended CI checks:

```yaml
- name: Rust benchmarks
  run: cargo bench --package db-pro-infrastructure -- --quick

- name: Native UI benchmarks
  run: cargo bench --package db-pro-ui -- --quick

- name: Native binary build
  run: cargo build --release --locked -p db-pro-native
```

Never claim performance improved without measurement evidence.

## Resources

- `references/perf-budgets.md` — Complete budget table with rationale
- `scripts/perf-scan.sh` — Automated audit script (`native|er|rust|db|all`)
