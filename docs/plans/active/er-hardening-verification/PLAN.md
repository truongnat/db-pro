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
