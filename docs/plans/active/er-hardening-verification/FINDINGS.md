# P1 Large-Schema ER Architecture — Findings

## F-1 (P1) Version overflow could cause stale result injection [FIXED]

**Severity:** P1
**Location:** `crates/ui/src/diagram_view.rs:109`
**Evidence:** `diagram_schema_version.wrapping_add(1)` — if version wraps to 0 after `u64::MAX` schema changes, a stale worker result with version 0 could pass the `res.graph_version == self.diagram_schema_version` check.
**Fix:** Changed to `saturating_add(1)` and initial version starts from 1 (not 0). At `u64::MAX`, the version stays `u64::MAX` which is effectively safe (schema won't change again at that point).

## F-2 (P1) Worker thread spawn panic could crash app [FIXED]

**Severity:** P1
**Location:** `crates/ui/src/diagram/layout.rs:71`
**Evidence:** `thread::Builder::new().spawn(...).expect("failed to spawn er layout background worker thread")` — under extreme resource exhaustion, `.expect()` would panic and crash the entire application.
**Fix:** Replaced `.expect()` with graceful error handling. `request_tx` wrapped in `Option<Sender>`. On spawn failure, `request_layout` silently sends to a disconnected channel and `poll_result` never returns. UI retains last valid graph (degraded mode).

## F-3 (P2) Coalescing drain could use unbounded CPU [FIXED]

**Severity:** P2
**Location:** `crates/ui/src/diagram/layout.rs:56`
**Evidence:** Original `while let Ok(newer) = request_rx.try_recv()` loop had no upper bound. Under rapid refresh (e.g., metadata polling every frame), this loop could spin indefinitely before processing.
**Fix:** Added `MAX_COALESCE_DRAIN = 64` cap. After 64 drain iterations, the worker processes the latest request. This is sufficient for coalescing while preventing CPU starvation.

## F-4 (INFO) Renderer architecture verified clean

The renderer (`paint_er_node_lod`, `paint_scene_edges`) only consumes `ErRenderScene.visible_nodes` and `ErRenderScene.visible_edges`. No code path iterates all `UiTableSummary`, walks raw `foreign_keys`, rebuilds adjacency, calculates edge bbox, or rebuilds node geometry during rendering. The `diagram_edge_bounding_box` function exists as dead code (tested only) — potential cleanup candidate.

## F-5 (INFO) Spatial index bounded and deterministic

Long-edge bucket explosion is mitigated by `max_span = 32` in `insert_edge`. A 1000-node dense graph with ~3000 edges builds in <500ms with bucket count <50,000. Edge references are bounded by `edges × 32 × 32`. No fallback scan in `query_edges`.

## F-6 (INFO) BFS terminates deterministically on all graph topologies

Verified: chains, cycles, self-FK, composite FK, isolated nodes, reverse relations. BFS uses `HashSet<usize>` for visited tracking. Ordering is deterministic across runs (VecDeque + sorted adjacency).
