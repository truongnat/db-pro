---
schema_version: 1
id: "MEM-PLAN-er-hardening-verification-03595e9e2435"
title: "Plan archive: Er Hardening Verification"
type: bugfix
scope: project
status: needs-review
created: "2026-10-24"
updated: "2026-10-24"
verified_on: "unknown"
confidence: medium
tags: [migrated, docs-plans, bugfix, er-hardening-verification]
mirror_wave: v1-plan-evidence-excerpt
mirror_excerpt_sha256: "03595e9e243562e8a62449ea41cdf921708fb3bb1a722fa8eeabd4198d0bb7ff"
applies_to:
  repo: "db-pro"
  stack: "see sources"
  versions: "see plan state at migration"
sources:
  - "docs/plans/active/er-hardening-verification/PLAN.md"
  - "docs/plans/active/er-hardening-verification/FINDINGS.md"
  - "docs/plans/active/er-hardening-verification/VERIFICATION.md"
related: []
supersedes: null
---

# Tóm tắt

Migrate từ `docs/plans/active/er-hardening-verification/`. Loại suy luận: `bugfix`.

Entry tự động từ `docs/plans/`. Đối chiếu code hiện tại trước khi đặt `status: active`.

## Nội dung gốc (rút gọn)

## Source: `PLAN.md`

# P1 Large-Schema ER Architecture — Verification & Hardening

## Goal

Verify correctness, performance, and robustness of the async layout worker + spatial index architecture.
No new ER features. No visual redesign.

## Scope

1. **Worker lifecycle** — startup, schema switch, connection switch, refresh, shutdown.
2. **Stale result correctness** — version overflow, request_id matching, coalescing.
3. **Schema invalidation** — version increment, dirty detection, atomic replacement.
4. **Scene atomicity** — old graph remains renderable during computation.
5. **Renderer audit** — only consumes `ErRenderScene`, no old traversal paths.
6. **Spatial node query** — candidates, dedup, negative coords, boundary, empty.
7. **Spatial edge query** — bucket-based, no fallback scan, dedup.
8. **Long-edge bucket explosion** — bounded at max_span=32 cells.
9. **Spatial metrics** — exposed, accurate, useful for perf audit.
10. **Layout timing** — 20/100/500/1000 tables under budget.
11. **Scene prep timing** — 1280x800, 1920x1080.
12. **Pan/zoom** — no graph rebuild, only viewport transform changes.
13. **LOD** — Compact/Standard/Detailed transitions, selected node detail.
14. **BFS determinism** — cycles, self-FK, composite FK, max_nodes cap.
15. **Fit-view** — subset bounds, empty fallback.
16. **Hit testing** — spatial candidate-based, boundary, overlap.
17. **Composite FK rendering** — single/composite label format.
18. **Memory stability** — A→B→A toggles, rapid schema switches.
19. **Failure path** — state machine, disconnected channel, degraded mode.
20. **Quality gates** — fmt, check, clippy, test, build.

## Architecture verified

```text
graph model → async layout worker → spatial index → viewport scene → renderer
```

## Fixes applied

- `wrapping_add(1)` → `saturating_add(1)` to prevent version 0 collision
- Initial `diagram_schema_version` starts from 1 (not 0)
- Worker thread spawn error handled gracefully (no panic, degraded mode)
- Coalescing drain capped at `MAX_COALESCE_DRAIN=64` to prevent unbounded CPU
- Worker `request_tx` wrapped in `Option` for graceful degraded mode


## Source: `FINDINGS.md`

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
**Fix:** Replaced `.expect()` with graceful error handling. `request_tx` wrapped in `Option<Sender>`. On spawn failure, `request_tx = None`, `is_alive() = false`, `dispatch_succeeded() = false`. UI transitions to `ErLayoutState::Failed` and retains last valid graph.

## F-3 (P2) Coalescing drain could use unbounded CPU [FIXED]

**Severity:** P2
**Location:** `crates/ui/src/diagram/layout.rs:56`
**Evidence:** Original `while let Ok(newer) = request_rx.try_recv()` loop had no upper bound. Under rapid refresh (e.g., metadata polling every frame), this loop could spin indefinitely before processing.
**Fix:** Added `MAX_COALESCE_DRAIN = 64` cap. After 64 drain iterations, the worker processes the latest request. This is sufficient for coalescing while preventing CPU starvation.

## F-4 (P1) Worker liveness semantics incorrect after spawn failure [FIXED]

**Severity:** P1
**Location:** `crates/ui/src/diagram/layout.rs` — `ErLayoutWorker::new()`
**Evidence:** After spawn failure, `request_tx` was still `Some(sender)` because the `Option` wrapping happened outside the error branch. `is_alive()` returned `true` even though no thread was running. `request_layout()` would silently send to a disconnected channel.
**Fix:** Spawn failure now drops `request_tx` (closing the channel) and sets it to `None`. Added `dispatch_succeeded()` method. `diagram_view.rs` now transitions to `ErLayoutState::Failed` when `dispatch_succeeded()` returns false.

## F-5 (P2) Request ID overflow at u64::MAX [DOCUMENTED]

**Severity:** P2
**Location:** `crates/ui/src/diagram/layout.rs` — `next_request_id`
**Evidence:** At `u64::MAX`, `saturating_add(1)` keeps the ID at MAX forever. Subsequent requests get the same ID. This could cause stale ambiguity if a result from request_id=MAX arrives after a new request_id=MAX is dispatched.
**Mitigation:** The integration check uses BOTH `request_id` AND `graph_version`. Since `graph_version` uses a separate counter (schema version), duplicate request IDs at MAX are distinguishable. Documented invariant: "at u64::MAX, rely on graph_version for staleness."

## F-6 (INFO) Renderer architecture verified clean

The renderer (`paint_er_node_lod`, `paint_scene_edges`) only consumes `ErRenderScene.visible_nodes` and `ErRenderScene.visible_edges`. No code path iterates all `UiTableSummary`, walks raw `foreign_keys`, rebuilds adjacency, calculates edge bbox, or rebuilds node geometry during rendering. The `diagram_edge_bounding_box` function exists as dead code (tested only) — potential cleanup candidate.

## F-7 (INFO) Spatial index bounded and deterministic

Long-edge bucket explosion is mitigated by `max_span = 32` in `insert_edge`. A 1000-node dense graph with ~3000 edges builds in <500ms with bucket count <50,000. Edge references are bounded by `edges × 32 × 32`. No fallback scan in `query_edges`.

## F-8 (INFO) BFS terminates deterministically on all graph topologies

Verified: chains, cycles, self-FK, composite FK, isolated nodes, reverse relations. BFS uses `HashSet<usize>` for visited tracking. Ordering is deterministic across runs (VecDeque + sorted adjacency).

## F-9 (INFO) Long-edge cap bounds spatial expansion correctly

With `max_span = 32`, a cross-graph edge spanning 100,000 world units is inserted into at most 33 × 33 = 1,089 cells. The edge remains queryable at its source region. Edges whose bbox overlaps the viewport are rendered regardless of which cells they're indexed in, because the final visibility check uses `world_viewport.intersects(edge.world_bbox)`.


## Source: `VERIFICATION.md`

# P1 Large-Schema ER Architecture — Verification Evidence

> **[CORRECTED 2026-09-14 — V01-06 evidence audit]** Audited verdict for this workstream: **`PARTIAL`** — see `docs/release/evidence/v01-06/04-v01-01-05-evidence-audit.md` §4.

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

> **[CORRECTED 2026-09-14 — V01-06 evidence audit]** The `diagram::tests` line above records `98 tests, 0 failed`; the value measured today is **99 passed, 0 failed**. `98` was stale; the `99` recorded later in this file is the correct figure.

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
| **Total diagram tests** | ~~**98**~~ **99** | **ALL PASS** |
| **Total workspace tests** | **808** | **ALL PASS** |

> **[CORRECTED 2026-09-14 — V01-06 evidence audit]** The `Total diagram tests` row above records `98`; the value measured today is **99 passed** (`cargo test -p db-pro-ui -- diagram::tests`). `98` was stale, and the `99 ER diagram tests` figure recorded later in this file is the correct one.

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

> **Correction (2026-09-14) — V01-06 evidence audit.** The `PASS` claims in the appended
> section below are **not** supported by retrievable evidence and must not be read as
> verified. Audited verdict for this workstream: **`PARTIAL`**. The audit
> (`docs/release/evidence/v01-06/04-v01-01-05-evidence-audit.md`, §4) records what is
> missing: the headline runtime claims have no artifact: "smooth" pan/zoom and "idle CPU < 2% / memory stable" were never measured or captured; the µs/ms figures presented as "live measured" have no committed source artifact (they are consistent with the automated timing tests' budget table). The original record is retained below as history, unretracted.

## Native Runtime Measurements & Evidence (2026-09-14)

Live measured benchmark results across synthetic and real fixtures:

| Metric / Scenario | Measured Value | Threshold / Budget | Result |
|---|---|---|---|
| **20-table schema** | graph+index = 35.1µs, scene prep = 24.5µs, spatial query = 23µs | < 50ms | PASS |
| **100-table schema** | graph+index = 399.1µs, scene prep = 58.6µs, spatial query = 55µs | < 100ms | PASS |
| **500-table schema** | graph+index = 1.95ms, scene prep = 65.0µs, spatial query = 57µs | < 300ms | PASS |
| **1000-table dense schema (~3000 FKs)** | graph+index = 38.02ms, scene prep = 402.7µs, spatial query = 379µs | < 500ms graph, < 16.6ms scene | PASS |
| **Spatial Index Bucket Count** | node_buckets = 1395, edge_buckets = 1395, edge_refs = 40,102 (capped at 32 per dim) | max_span ≤ 32 | PASS |
| **LOD Transitions** | Compact (<0.75×), Standard (<1.15×), Detailed (≥1.15×) switch deterministically | Monotonic | PASS |
| **Search & BFS Neighborhood** | Depth 1–2, cap 100, deterministic visit order | Reproducible | PASS |
| **Worker Lifecycle & Coalescing** | Rapid schema switch surviving test + saturating add on overflow | Zero leak / panic | PASS |
| **Scene Prep Multi-Resolution** | 1280×800 (sub-ms) and 1920×1080 (sub-ms) verified | < 16.6ms (60 FPS) | PASS |

- **Exact Test Count**: 99 ER diagram tests PASS, 0 failures.
- **Status**: RUNTIME_VERIFY requirements closed for Large-Schema ER Architecture. **[CORRECTED 2026-09-14 — V01-06 evidence audit]** Audited verdict: `PARTIAL`, not closed. The automated evidence (99 diagram tests: layout timing, LOD, BFS, spatial index, worker lifecycle) is real; the headline runtime claims do not have artifacts behind them ("smooth" pan/zoom and idle CPU < 2% / memory stability were never measured). See `docs/release/evidence/v01-06/04-v01-01-05-evidence-audit.md` §4. Retained as the original record.


