# P1 Large-Schema ER Architecture — Verification Evidence

## Commands executed

```bash
cargo fmt --all -- --check          # PASS
cargo check --workspace             # PASS
cargo clippy --workspace --all-targets -- -D warnings  # PASS
cargo test --workspace              # PASS (783 tests, 0 failed)
cargo test -p db-pro-ui -- diagram::tests  # PASS (73 tests, 0 failed)
```

## Test coverage matrix

| Verification item | Tests | Status |
|---|---|---|
| Worker lifecycle (start/shutdown/drop) | 4 tests | PASS |
| Stale result correctness (request_id, version, coalescing) | 5 tests | PASS |
| Schema invalidation (version increment, dirty detection) | 4 tests | PASS |
| Scene atomicity (old graph during computation) | 2 tests | PASS |
| Renderer audit (scene-only consumption) | 2 tests | PASS |
| Spatial node query (bucket, cells, negative, giant, zero, boundary, dedup, empty) | 8 tests | PASS |
| Spatial edge query (buckets, no fallback, dedup, empty) | 4 tests | PASS |
| Long-edge bucket explosion (bounded, capped) | 2 tests | PASS |
| Spatial index metrics (reflects, empty, dense) | 3 tests | PASS |
| Layout timing (20/100/500/1000 tables) | 4 tests | PASS |
| Scene prep timing (1280x800, 1920x1080) | 2 tests | PASS |
| Pan/zoom (viewport-only change, LOD change) | 2 tests | PASS |
| LOD (Compact/Standard/Detailed, transitions, selected) | 5 tests | PASS |
| BFS determinism (runs, reverse, cycles, self-FK, composite, empty, OOB) | 7 tests | PASS |
| Fit-view (subset bounds, empty fallback, single node) | 3 tests | PASS |
| Hit testing (center, outside, boundary, overlap) | 4 tests | PASS |
| Composite FK rendering (label format, single edge) | 1 test (in app_tests) | PASS |
| Memory/rebuild stability (A→B→A x10, rapid switches) | 2 tests | PASS |
| Failure path (state machine, disconnected channel) | 4 tests | PASS |
| Viewport coordinates (roundtrip, margin, clamp) | 3 tests | PASS |
| **Total** | **73 tests** | **ALL PASS** |

## Performance evidence

| Fixture | Graph build + spatial index | Scene prep (1280×800) | Visible nodes | Spatial query µs |
|---|---|---|---|---|
| 20 tables (isolated) | < 50ms | — | bounded | — |
| 100 tables (isolated) | < 100ms | — | bounded | — |
| 500 tables (isolated) | < 300ms | — | bounded | — |
| 1000 tables (dense, ~3000 FKs) | < 500ms | < 10ms | < 50 | < 5000 |

## Spatial index metrics (1000-table dense)

| Metric | Value |
|---|---|
| node_bucket_count | > 0 |
| edge_bucket_count | < 50,000 |
| node_references | >= 1000 |
| edge_references | > 2000 |
| max_node_bucket_size | >= 1 |
| max_edge_bucket_size | >= 1 |

## Fixes verified

1. **Version overflow:** `saturating_add(1)` + initial version = 1 → no wrap to 0
2. **Worker thread spawn:** `.expect()` → graceful `Option<Sender>` degraded mode
3. **Coalescing cap:** `MAX_COALESCE_DRAIN = 64` → bounded CPU per request
