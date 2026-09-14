# P1 Large-Schema ER Architecture — Verification Evidence

## Commit range

```
aac23ff..db21666
```

Key commits:
- `aac23ff` perf(ui): add viewport frustum culling and composite FK formatting
- `81dc45f` feat(er): implement large-schema ER architecture (spatial index, viewport, LOD, BFS)
- `6a5c3e9` feat(er): harden architecture (async layout worker, bounded spatial index)
- `a9fcf59` test(er): 73 verification tests, version overflow fix, worker degraded mode
- `f32b1b9` fix(er): worker liveness semantics, 26 additional verification tests
- `db21666` refactor(er): remove dead diagram_edge_bounding_box

## Commands executed

```bash
cargo fmt --all -- --check          # PASS
cargo check --workspace             # PASS
cargo clippy --workspace --all-targets -- -D warnings  # PASS
cargo test --workspace              # PASS (808 tests, 0 failed)
cargo test -p db-pro-ui -- diagram::tests  # PASS (98 tests, 0 failed)
```

## Test coverage matrix

| Verification item | Tests | Status |
|---|---|---|
| Worker lifecycle (start/shutdown/drop) | 4 | PASS |
| Worker liveness semantics (is_alive, dispatch_succeeded) | 3 | PASS |
| Stale result correctness (request_id, version, coalescing) | 5 | PASS |
| Request ID overflow invariant | 2 | PASS |
| Schema version overflow invariant | 2 | PASS |
| App-boundary stale commit integration | 3 | PASS |
| Atomic scene commit | 2 | PASS |
| Schema invalidation (version increment, dirty detection) | 4 | PASS |
| Scene atomicity (old graph during computation) | 2 | PASS |
| Renderer audit (scene-only consumption) | 3 | PASS |
| Spatial node query (8 scenarios) | 8 | PASS |
| Spatial edge query (4 scenarios) | 4 | PASS |
| Long-edge bucket explosion (bounded, capped) | 2 | PASS |
| Long-edge queryability after cap | 2 | PASS |
| Spatial index metrics (reflects, empty, dense) | 3 | PASS |
| Spatial query dedup (node, edge) | 2 | PASS |
| Layout timing (20/100/500/1000 tables) | 4 | PASS |
| Scene prep timing (1280x800, 1920x1080) | 2 | PASS |
| Pan/zoom (viewport-only, LOD change) | 2 | PASS |
| Pan/zoom no-rebuild integration | 2 | PASS |
| LOD (Compact/Standard/Detailed, transitions, selected) | 5 | PASS |
| BFS determinism (7 graph topologies) | 7 | PASS |
| Search/neighborhood reuses existing graph | 1 | PASS |
| Fit-view (subset bounds, empty fallback, single node) | 3 | PASS |
| Hit testing (center, outside, boundary, overlap) | 4 | PASS |
| Composite FK (label format, single edge) | 1 | PASS |
| Memory/rebuild stability (A→B→A x10, rapid switches) | 2 | PASS |
| Failure path (state machine, disconnected channel) | 4 | PASS |
| Viewport coordinates (roundtrip, margin, clamp) | 3 | PASS |
| Performance evidence (20/100/500/1000) | 4 | PASS |
| Frame path performance (100 scene preps) | 1 | PASS |
| **Total diagram tests** | **98** | **ALL PASS** |
| **Total workspace tests** | **808** | **ALL PASS** |

## Performance evidence

| Fixture | Graph + index build | Scene prep (1280×800) | Visible nodes | Spatial query µs |
|---|---|---|---|---|
| 20 tables (isolated) | < 50ms | sub-ms | bounded | — |
| 100 tables (isolated) | < 100ms | sub-ms | bounded | — |
| 500 tables (isolated) | < 300ms | sub-ms | bounded | — |
| 1000 tables (dense, ~3000 FKs) | < 500ms | < 10ms | < 50 | < 5000 |

100 scene preparations on 10-table chain: < 50ms total (sub-ms each).

## Spatial index metrics (1000-table dense)

| Metric | Value |
|---|---|
| node_bucket_count | > 0 |
| edge_bucket_count | < 50,000 |
| node_references | >= 1000 |
| edge_references | > 2000 |
| max_node_bucket_size | >= 1 |
| max_edge_bucket_size | >= 1 |

## Architecture invariants verified

1. **5-layer pipeline:** graph model → async layout worker → spatial index → viewport scene → renderer
2. **Worker coalescing:** rapid requests → latest processed, stale discarded
3. **Stale result safety:** BOTH request_id AND graph_version must match
4. **Atomic commit:** graph + spatial_index assigned in same code path, always consistent
5. **No frame-path rebuild:** pan/zoom/search only change viewport/LOD/filter, never graph/index
6. **Bounded spatial index:** max_span=32 caps edge expansion; no bucket explosion
7. **BFS deterministic:** VecDeque + HashSet visited, sorted adjacency → reproducible order
8. **LOD render-tree switch:** Compact → no columns; Standard → bounded; Detailed → full
9. **Worker degraded mode:** spawn failure → request_tx=None, is_alive()=false, UI shows Failed
10. **Dead code removed:** diagram_edge_bounding_box eliminated; edge bboxes precomputed in ErGraph::build

## Fixes applied during verification

| ID | Severity | Fix |
|---|---|---|
| F-1 | P1 | Version overflow: wrapping_add → saturating_add, initial version = 1 |
| F-2 | P1 | Worker spawn panic: .expect() → graceful Option&lt;Sender&gt; degraded mode |
| F-3 | P2 | Coalescing drain: added MAX_COALESCE_DRAIN = 64 cap |
| F-4 | P1 | Worker liveness: request_tx = None on spawn failure, dispatch_succeeded() added |
| F-5 | P2 | Request ID overflow: documented invariant (rely on graph_version at MAX) |

## Native runtime evidence (pending)

Native runtime verification requires running `db-pro-native` against actual database connections. The following items need live evidence:

- 1000-table initial open + pan + zoom
- Worker lifecycle under rapid schema switches
- LOD transitions at 1280×800, 1440×900, 1920×1080
- Search/BFS neighborhood exploration
- Fit-view behavior
- Idle CPU stability

These are recorded as PENDING in the provider/runtime evidence matrix. Automated test evidence (source + automated) is sufficient for RUNTIME_VERIFY state per FEATURE_LIFECYCLE.md.
