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

### Benchmarks that exist (from `crates/ui/benches/result_grid_benchmarks.rs`)

The criterion ids below are the ones the file actually registers; the measured baseline is in
`docs/architecture/performance-baseline.md`. Operations with no registered benchmark are listed as
such — they are **not** measured by this suite, and no budget row here may read as if they were.

| Benchmark id (group / function) | Budget | Status |
|-----------|--------|--------|
| `result_grid_million_rows / project_without_filter_or_sort` | < 5 ms | measured (~3.15 ms baseline) |
| `result_grid_scroll_window / materialize_100_visible_rows` | < 1 ms | measured (~38 ns at a mid-list offset) |
| `result_grid_requested_sizes / build_visual_maps_{1_000,10_000}_rows_50_columns` | no budget | baseline only |
| Cell formatting / codec round-trip | < 1 ms | **no benchmark registered** (hand-measured at most) |
| Quick Open index + rank | < 5 ms | **no benchmark registered** |
| Statement split (100 statements) | < 5 ms | **no benchmark registered** |

The last three rows are React-era budget entries kept for reference; nothing in `crates/ui/benches/`
exercises them, so a reader must not treat them as enforced. Check the criterion ids in the bench file
(`grep -n 'bench_function' crates/ui/benches/result_grid_benchmarks.rs`) before quoting a row.

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

The ER diagram is a custom `egui::Painter`: the view lives in `crates/ui/src/diagram_view.rs` and the
graph, layout, LOD, spatial index and viewport code in `crates/ui/src/diagram/` (`layout.rs`, `lod.rs`,
`model.rs`, `scene.rs`, `spatial.rs`, `viewport.rs`). There is no React Flow, cytoscape or dagre in the
shipping path — those belonged to the archived frontend.

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

The real criterion ids are `sqlite_connect_and_disconnect`, `introspect_small_db`,
`introspect_large_schema`, `query_rows/select_10k`, `query_rows/select_100k`,
`query_json_blob/select_json_metadata_5k`, `serialize_large_text/select_1k_large_text` and
`explain_query` (`grep -n 'bench_function' crates/infrastructure/benches/sqlite_benchmarks.rs`).
Measured baseline: `docs/architecture/performance-baseline.md`.

| Operation (criterion id) | Budget |
|-----------|--------|
| SQLite connect/disconnect (`sqlite_connect_and_disconnect`) | < 100 ms |
| Introspect small DB, 5 tables (`introspect_small_db`) | < 300 ms |
| Introspect large schema, 50×20 (`introspect_large_schema`) | < 300 ms |
| Query 10k rows (`query_rows/select_10k`) | < 150 ms |
| Query 100k rows (`query_rows/select_100k`) | < 500 ms |
| JSON blob 5k rows (`query_json_blob/select_json_metadata_5k`) | < 100 ms |
| Large text 1k rows × 4 KB (`serialize_large_text/select_1k_large_text`) | < 200 ms |
| Cancel acknowledgement | < 200 ms — **not benchmarked here**; measured in the execution registry |

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

**No performance check runs in CI today, and none is claimed to.** `.github/workflows/ci.yml` has a
single `Rust checks` job (`fmt`, `check`, `clippy -D warnings`, `test`) and
`.github/workflows/release.yml` runs the same pre-flight gates before building the three platform
archives — neither invokes `perf-scan.sh` or `cargo bench`. The perf scan is a **local** release gate
whose output is recorded in the release evidence (`docs/release/0.1.0-handoff.md` marks it
"local gate; not run in CI"), and `docs/architecture/performance-baseline.md` states the same:
"Budget targets are NOT CI gates — they are guidelines for early detection."

This is the deliberate position, not an omission: a criterion run on a shared runner is noisy enough
that a red build would not reliably mean a regression. If the position changes, wire the checks below
into `ci.yml` **and** update the release documents that currently say they are not wired.

```yaml
# candidate checks — NOT wired into any workflow today
- name: Rust benchmarks
  run: cargo bench --package db-pro-infrastructure -- --quick

- name: Native UI benchmarks
  run: cargo bench --package db-pro-ui -- --quick

- name: Native binary build
  run: cargo build --release --locked -p db-pro-native
```

Never claim performance improved without measurement evidence.

## 8. Output semantics, provenance and exit codes

`perf-scan.sh` is used as release evidence, so its output contract is explicit and self-tested:

| Status | Exit | Meaning |
|---|---:|---|
| `PASS` | 0 | every executed check passed, the tree is committed, and no section was skipped |
| `PASS (partial)` | 0 | every executed check passed, but the run deliberately did not execute sections — **the status names them** (`all` skips the two benchmark sections and the ER/DB runtime sections) |
| `WARN` | **2** | at least one warning and no failure. Exit 2 exists so an automated gate that only reads the exit status can never certify a warned run as green |
| `FAIL` | 1 | at least one failed check |

A run whose working tree is not committed is qualified in the status line with
`[source <sha>+dirty(N): working tree not committed]`, so its numbers describe the tree rather than the
recorded commit. Every run also prints, in the header and again in the summary:

- `Source revision: <sha>[+dirty(N)]`
- `Measured artifact: target/release/db-pro-native sha256 <digest> (<size>MB)`

Provenance rule: **a budget number is only quotable together with the artifact digest and the source
revision it was measured at.** The size check reads the binary produced by the `cargo build` in the same
run; when that build fails, the scan reports the failure and **no size at all** — a stale binary is
never measured. Quote the revision and the digest when copying a result into a document or an issue.

```bash
# assert the status/exit-code contract itself (no build, no benchmark, ~0 s)
bash .skills/perf-audit/scripts/perf-scan.sh --self-test
```

The self-test covers: a clean full pass reads as an unqualified `PASS` and exits 0; an uncommitted tree
is named and does not fail; a warning reads as `WARN` and exits 2; a failure reads as `FAIL` and exits
1; a skipped section makes the pass partial and is named; `skip()` accounts and records the section; and
`sha256_of` agrees with `openssl dgst -sha256` on a known file.

## Resources

- `references/perf-budgets.md` — Complete budget table with rationale
- `scripts/perf-scan.sh` — Automated audit script (`native|er|rust|db|all`, plus `--self-test` for the
  result/exit-code contract described in §8)
- `docs/architecture/performance-baseline.md` — the measured baseline the budget tables refer to
