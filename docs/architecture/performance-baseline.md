# Performance Benchmarks — Baseline Report

**Status**: Implemented  
**Date**: PATCH 2 P2-11  
**Tool**: Criterion 0.5  
**Run command**: `cargo bench --package db-pro-infrastructure`

## Baseline Results (macOS, Apple Silicon, debug=off)

| Benchmark | Mean Time | Budget Target | Status |
|-----------|-----------|---------------|--------|
| `sqlite_connect_and_disconnect` | ~198 µs | < 100 ms | ✅ PASS |
| `introspect_small_db` (5 tables) | ~54 µs | < 300 ms | ✅ PASS |
| `introspect_large_schema` (50 tables × 20 cols) | ~500 µs | < 300 ms | ✅ PASS |
| `query_rows/select_10k` | ~2.5 ms | < 150 ms | ✅ PASS |
| `query_rows/select_100k` | ~25 ms | — | ✅ baseline |
| `query_json_blob/select_json_metadata_5k` | ~1.1 ms | — | ✅ baseline |
| `serialize_large_text/select_1k_large_text` | ~790 µs | — | ✅ baseline |
| `explain_query` | ~10 µs | — | ✅ baseline |

## Performance Budget Draft

| Operation | Budget | Measured | Headroom |
|-----------|--------|----------|----------|
| SQLite connect | < 100 ms | 0.2 ms | 500× |
| Metadata small DB | < 300 ms | 0.05 ms | 6000× |
| Serialize 10k rows | < 150 ms | 2.5 ms | 60× |
| Cancel acknowledgement | < 200 ms | — | (measured in execution registry) |

## What is Measured

- **Connect/disconnect**: Open + close in-memory SQLite connection.
- **Introspection (small)**: Full schema introspection on 5-table fixture schema.
- **Introspection (large)**: Full schema introspection on 50 tables × 20 columns.
- **Query 10k/100k rows**: `SELECT * FROM orders` with 10k/100k rows, measuring DB → Rust DTO mapping.
- **JSON/metadata**: Query 5k rows with JSON text columns (tags, metadata).
- **Large text**: Query 1k rows with ~4KB text bodies.
- **Explain**: `EXPLAIN QUERY PLAN` for a filtered SELECT.

## What is NOT Measured (Out of Scope)

- Frontend rendering performance (PATCH 1 responsibility).
- PostgreSQL driver benchmarks (requires running PostgreSQL instance).
- Tauri serialization boundary (Rust → IPC → JS).
- Network latency for remote connections.

## Notes

- These are backend-only benchmarks measuring `DB → Rust DTO` path.
- Budget targets are NOT CI gates — they are guidelines for early detection.
- Criterion `--quick` mode used for fast iteration; full runs give more statistical power.
- Throughput reported in Melem/s (million elements per second) for row-based benchmarks.
