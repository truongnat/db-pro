# P1 Large-Schema ER Architecture — Verification Checklist

## Worker lifecycle

- [x] Worker starts cleanly and polls without results initially
- [x] Worker shutdown via drop — channel closes, thread exits naturally
- [x] New worker instance after drop works fine
- [x] Coalescing: 5 rapid requests → worker processes latest
- [x] Worker thread spawn failure → degraded mode (no panic)

## Stale result correctness

- [x] Stale request_id rejected by integration check
- [x] Stale version rejected by integration check
- [x] Default graph version (0) never matches app version (1)
- [x] saturating_add never wraps to 0
- [x] Worker latest result always wins over earlier requests

## Schema invalidation

- [x] Version increments correctly with saturating_add
- [x] Graph dirty detection by node count mismatch
- [x] Graph clean when version and count match
- [x] Spatial index consistent with graph after build

## Scene atomicity

- [x] Old graph still renders during computation
- [x] Computing state does not clear previous graph

## Renderer audit

- [x] Scene contains only visible usize IDs (type system enforced)
- [x] Scene metrics accurate
- [x] paint_er_node_lod uses scene.visible_nodes (no raw table iteration)
- [x] paint_scene_edges uses scene.visible_edges (no raw FK walk)

## Spatial node query

- [x] Query inside one bucket
- [x] Query crosses multiple cells
- [x] Query with negative world coordinates
- [x] Giant viewport returns all nodes
- [x] Zero-size viewport at node center
- [x] Node exactly on cell boundary
- [x] No duplicate node IDs
- [x] Empty index returns empty

## Spatial edge query

- [x] Uses edge buckets (not node buckets)
- [x] No fallback scan of all graph.edges
- [x] Deduplicates across cells
- [x] Empty index returns empty

## Long-edge bucket explosion

- [x] 1000-table dense graph: bucket count < 50,000
- [x] Edge references bounded by edges × 32 × 32
- [x] Single long edge capped at 33 × 33 cells max

## Spatial index metrics

- [x] Metrics reflect actual index state
- [x] Empty index metrics all zero
- [x] Dense graph: node_references >= 1000, edge_references > 2000

## Layout timing

- [x] 20 tables: < 50ms
- [x] 100 tables: < 100ms
- [x] 500 tables: < 300ms
- [x] 1000 tables dense: < 500ms

## Scene preparation timing

- [x] 1000 tables at 1280×800: < 10ms, visible < 50
- [x] 1000 tables at 1920×1080: < 15ms, visible < 80

## Pan/zoom performance

- [x] Pan only changes viewport — total nodes/edges unchanged
- [x] Zoom changes LOD without rebuilding graph

## LOD

- [x] Compact: no columns, no data types, no edge labels
- [x] Standard: 6 columns max
- [x] Detailed: 12 columns max
- [x] Monotonic transitions at correct thresholds
- [x] Selected node at Compact shows detail (renderer match arm)

## BFS determinism

- [x] Identical results across 3 runs
- [x] Reverse relations terminate correctly
- [x] Cycles terminate without infinite loop
- [x] Self-FK terminates with single node
- [x] Composite FK creates adjacency
- [x] Empty seed returns empty
- [x] Out-of-bounds seed returns empty

## Fit-view behavior

- [x] Subset bounds smaller than full
- [x] Empty subset falls back to world bounds
- [x] Single node subset includes canvas margin

## Hit testing

- [x] Hit center of node
- [x] Outside returns None
- [x] On boundary returns the node
- [x] Overlap deterministic

## Composite FK rendering

- [x] Single FK: "col → col"
- [x] Composite FK: "[col1, col2] → [col1, col2]"
- [x] Composite FK produces single edge in graph

## Memory/rebuild stability

- [x] A→B→A toggles 10 times — stable
- [x] Worker survives 20 rapid schema switches

## Failure path

- [x] Layout state Idle is initial
- [x] Computing carries request_id and graph_version
- [x] Failed carries message
- [x] Disconnected channel no panic

## Viewport coordinates

- [x] World-to-screen roundtrip
- [x] Visible rect expands by margin
- [x] Zoom clamped to [0.5, 2.0]

## Quality gates

- [x] cargo fmt --all -- --check
- [x] cargo check --workspace
- [x] cargo clippy --workspace --all-targets -- -D warnings
- [x] cargo test --workspace (783 tests, 0 failed)
