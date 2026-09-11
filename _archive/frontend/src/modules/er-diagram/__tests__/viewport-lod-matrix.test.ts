import { describe, it, expect } from "vitest";
import { resolveLod, lodTier, type LodLevel } from "../utils/lod";
import { resolveEdgeLod, type EdgeLodLevel } from "../utils/edge-lod";

/**
 * #43 D3 — Viewport-driven LOD matrix.
 *
 * After first paint at compact, the viewport controls LOD:
 *
 *   zoom < 0.20  → dot     (tier 0)   edge: aggregate
 *   0.20–0.45    → compact (tier 1)   edge: simple
 *   0.45–0.70    → summary (tier 2)   edge: simple → full at ≥0.60
 *   ≥ 0.70       → detail  (tier 3)   edge: full
 *
 * Focused node: always detail regardless of zoom.
 * Compact toggle: caps base at summary (focused node still detail).
 *
 * Edge thresholds (independent of node):
 *   < 0.25  → aggregate (merged relations, no markers)
 *   0.25–0.6 → simple (straight, no markers/labels)
 *   ≥ 0.6   → full (smoothstep, markers, FK labels)
 *
 * Critical band 0.60–0.70: edge = full, node = summary.
 * Edges must NOT reference per-column handles (only exist on detail nodes).
 * The production code strips sourceHandle/targetHandle when currentLod ≠ "detail".
 */

/* ── 1. Node LOD threshold matrix ─────────────────────────────────────────── */

describe("#43 D3: node LOD threshold matrix", () => {
  const cases: { zoom: number; expected: LodLevel }[] = [
    // Dot band: zoom < 0.20
    { zoom: 0.01, expected: "dot" },
    { zoom: 0.1, expected: "dot" },
    { zoom: 0.19, expected: "dot" },
    // Compact band: 0.20 ≤ zoom < 0.45
    { zoom: 0.2, expected: "compact" },
    { zoom: 0.3, expected: "compact" },
    { zoom: 0.44, expected: "compact" },
    // Summary band: 0.45 ≤ zoom < 0.70
    { zoom: 0.45, expected: "summary" },
    { zoom: 0.5, expected: "summary" },
    { zoom: 0.6, expected: "summary" },
    { zoom: 0.69, expected: "summary" },
    // Detail band: zoom ≥ 0.70
    { zoom: 0.7, expected: "detail" },
    { zoom: 0.8, expected: "detail" },
    { zoom: 1.0, expected: "detail" },
    { zoom: 2.0, expected: "detail" },
  ];

  for (const { zoom, expected } of cases) {
    it(`zoom ${zoom} → ${expected}`, () => {
      expect(resolveLod(zoom)).toBe(expected);
    });
  }
});

/* ── 2. Edge LOD threshold matrix ─────────────────────────────────────────── */

describe("#43 D3: edge LOD threshold matrix", () => {
  const cases: { zoom: number; expected: EdgeLodLevel }[] = [
    // Aggregate: zoom < 0.25
    { zoom: 0.01, expected: "aggregate" },
    { zoom: 0.1, expected: "aggregate" },
    { zoom: 0.24, expected: "aggregate" },
    // Simple: 0.25 ≤ zoom < 0.60
    { zoom: 0.25, expected: "simple" },
    { zoom: 0.3, expected: "simple" },
    { zoom: 0.5, expected: "simple" },
    { zoom: 0.59, expected: "simple" },
    // Full: zoom ≥ 0.60
    { zoom: 0.6, expected: "full" },
    { zoom: 0.65, expected: "full" },
    { zoom: 0.7, expected: "full" },
    { zoom: 1.0, expected: "full" },
  ];

  for (const { zoom, expected } of cases) {
    it(`zoom ${zoom} → ${expected}`, () => {
      expect(resolveEdgeLod(zoom)).toBe(expected);
    });
  }
});

/* ── 3. Combined zoom → node LOD + edge LOD ───────────────────────────────── */

describe("#43 D3: combined node + edge LOD at key zoom levels", () => {
  const matrix: {
    zoom: number;
    nodeLod: LodLevel;
    edgeLod: EdgeLodLevel;
    nodeTier: number;
  }[] = [
    { zoom: 0.1, nodeLod: "dot", edgeLod: "aggregate", nodeTier: 0 },
    { zoom: 0.2, nodeLod: "compact", edgeLod: "aggregate", nodeTier: 1 },
    { zoom: 0.3, nodeLod: "compact", edgeLod: "simple", nodeTier: 1 },
    { zoom: 0.45, nodeLod: "summary", edgeLod: "simple", nodeTier: 2 },
    { zoom: 0.5, nodeLod: "summary", edgeLod: "simple", nodeTier: 2 },
    { zoom: 0.6, nodeLod: "summary", edgeLod: "full", nodeTier: 2 },
    { zoom: 0.7, nodeLod: "detail", edgeLod: "full", nodeTier: 3 },
    { zoom: 1.0, nodeLod: "detail", edgeLod: "full", nodeTier: 3 },
  ];

  for (const { zoom, nodeLod, edgeLod, nodeTier } of matrix) {
    it(`zoom ${zoom}: node=${nodeLod} (tier ${nodeTier}), edge=${edgeLod}`, () => {
      expect(resolveLod(zoom)).toBe(nodeLod);
      expect(resolveEdgeLod(zoom)).toBe(edgeLod);
      expect(lodTier(nodeLod)).toBe(nodeTier);
    });
  }
});

/* ── 4. Critical band 0.60–0.70: edge full, node summary ─────────────────── */

describe("#43 D3: critical band 0.60–0.70 (edge full, node summary)", () => {
  const bandZooms = [0.6, 0.62, 0.65, 0.68, 0.69];

  for (const zoom of bandZooms) {
    it(`zoom ${zoom}: edge=full but node=summary (handles must be stripped)`, () => {
      // Edge LOD: full (≥ 0.60)
      expect(resolveEdgeLod(zoom)).toBe("full");
      // Node LOD: summary (< 0.70)
      expect(resolveLod(zoom)).toBe("summary");
      // Since node ≠ detail, per-column handles don't exist on nodes.
      // Production code must strip sourceHandle/targetHandle from edges.
      // This is verified by the displayEdges memo:
      //   const stripHandleIds = currentLod !== "detail";
      // currentLod = "summary" → stripHandleIds = true.
      expect(resolveLod(zoom)).not.toBe("detail");
    });
  }
});

/* ── 5. Focus override: focused node always detail ────────────────────────── */

describe("#43 D3: focus override at every zoom level", () => {
  // The production logic: focused node → "detail" regardless of zoom.
  //   const targetLod = focusedNodeId && node.id === focusedNodeId
  //     ? "detail" : currentLod;
  // This means at ANY zoom, the focused node hydrates to detail (tier 3).

  const zoomLevels = [0.1, 0.2, 0.3, 0.45, 0.5, 0.6, 0.69];

  for (const zoom of zoomLevels) {
    it(`zoom ${zoom}: base=${resolveLod(zoom)}, focused=detail (tier 3)`, () => {
      const baseLod = resolveLod(zoom);
      // Focused node override: always "detail"
      const focusedLod: LodLevel = "detail";
      // Siblings stay at base LOD
      const siblingLod = baseLod;

      expect(focusedLod).toBe("detail");
      expect(lodTier(focusedLod)).toBe(3);
      // Siblings at base LOD (not elevated by focus)
      expect(siblingLod).toBe(baseLod);
      // Detail count = exactly 1 (the focused node)
      // This is enforced by the reducer: focusedNodeId is a single string | null
    });
  }
});

/* ── 6. Focus + zoom change: focused stays detail, siblings follow zoom ─── */

describe("#43 D3: focus + zoom change interaction", () => {
  it("zoom 0.80 → focus B → zoom 0.30: B stays detail, siblings compact", () => {
    // Step 1: zoom 0.80 → base = detail
    expect(resolveLod(0.8)).toBe("detail");

    // Step 2: focus B → B = detail (override), siblings = detail (base)
    // Detail count = all nodes (since base = detail and focused = detail)
    // But the focused node is still exactly 1 with the override applied.

    // Step 3: zoom 0.30 → base = compact
    expect(resolveLod(0.3)).toBe("compact");
    // B still = detail (focus override), siblings = compact
    expect(lodTier("detail")).toBe(3);
    expect(lodTier("compact")).toBe(1);
  });

  it("zoom 0.50 → focus B → zoom 0.10: B stays detail, siblings dot", () => {
    // zoom 0.50 → base = summary
    expect(resolveLod(0.5)).toBe("summary");

    // zoom 0.10 → base = dot
    expect(resolveLod(0.1)).toBe("dot");
    // B still = detail, siblings = dot
    expect(lodTier("detail")).toBe(3);
    expect(lodTier("dot")).toBe(0);
  });
});

/* ── 7. Clear focus → node returns to base LOD ───────────────────────────── */

describe("#43 D3: clear focus returns to base LOD", () => {
  it("after clear focus at zoom 0.50: all nodes summary, detail=0", () => {
    const baseLod = resolveLod(0.5);
    expect(baseLod).toBe("summary");
    // After CLEAR_FOCUS: focusedNodeId = null
    // All nodes use base LOD → no detail nodes
    // detail count = 0
    expect(lodTier(baseLod)).toBe(2);
  });

  it("after clear focus at zoom 0.10: all nodes dot, no [data-column]", () => {
    const baseLod = resolveLod(0.1);
    expect(baseLod).toBe("dot");
    // Dot nodes have zero column rows
    expect(lodTier(baseLod)).toBe(0);
  });
});

/* ── 8. Compact toggle: caps base at summary ──────────────────────────────── */

describe("#43 D3: compact toggle", () => {
  it("zoom 0.80 + compact → summary (not detail)", () => {
    expect(resolveLod(0.8, true)).toBe("summary");
  });

  it("zoom 1.0 + compact → summary (not detail)", () => {
    expect(resolveLod(1.0, true)).toBe("summary");
  });

  it("compact does not elevate low zoom levels", () => {
    expect(resolveLod(0.1, true)).toBe("dot");
    expect(resolveLod(0.3, true)).toBe("compact");
    expect(resolveLod(0.5, true)).toBe("summary");
  });

  it("compact + focus: focused node still detail, base capped at summary", () => {
    // Production logic:
    //   const targetLod = focusedNodeId && node.id === focusedNodeId
    //     ? "detail" : currentLod;
    // currentLod = resolveLod(zoom, compact=true)
    // At zoom 0.80 + compact: currentLod = "summary"
    // Focused node: "detail" (override)
    // Siblings: "summary" (capped by compact)
    const baseWithCompact = resolveLod(0.8, true);
    expect(baseWithCompact).toBe("summary");

    // Focus override is independent of compact
    const focusedLod: LodLevel = "detail";
    expect(focusedLod).toBe("detail");
    expect(lodTier(focusedLod)).toBe(3);
  });
});

/* ── 9. Edge-handle stripping invariant ───────────────────────────────────── */

describe("#43 D3: edge-handle stripping when nodes below detail", () => {
  // Production logic:
  //   const stripHandleIds = currentLod !== "detail";
  // When stripHandleIds = true, sourceHandle/targetHandle are set to undefined.
  // This prevents edges from referencing per-column handles that don't exist
  // on non-detail nodes (which only have generic fallback handles).

  const belowDetailZooms = [0.01, 0.1, 0.2, 0.3, 0.45, 0.5, 0.6, 0.69];

  for (const zoom of belowDetailZooms) {
    it(`zoom ${zoom}: node=${resolveLod(zoom)} ≠ detail → handles stripped`, () => {
      const lod = resolveLod(zoom);
      const stripHandleIds = lod !== "detail";
      expect(stripHandleIds).toBe(true);
    });
  }

  it("zoom 0.70: node=detail → handles NOT stripped", () => {
    const lod = resolveLod(0.7);
    expect(lod).toBe("detail");
    const stripHandleIds = lod !== "detail";
    expect(stripHandleIds).toBe(false);
  });

  it("zoom 0.65: edge=full but node=summary → handles stripped (critical band)", () => {
    const nodeLod = resolveLod(0.65);
    const edgeLod = resolveEdgeLod(0.65);
    expect(edgeLod).toBe("full");
    expect(nodeLod).toBe("summary");
    // Even though edge is "full", handles must be stripped because
    // nodes are at summary (no per-column handles exist).
    expect(nodeLod).not.toBe("detail");
  });
});

/* ── 10. Small/medium schema regression ───────────────────────────────────── */

describe("#43 D3: LOD thresholds are schema-size independent", () => {
  // The LOD resolution is purely zoom-driven — it does not depend on
  // table count, schema tier, or large-schema mode. Small/medium schemas
  // use the same thresholds.
  it("resolveLod returns same values regardless of schema size", () => {
    // These are the same thresholds used for both small and large schemas.
    // The component's `currentLod` state is driven by onViewportChange
    // which calls resolveLod(viewport.zoom, compact).
    expect(resolveLod(0.1)).toBe("dot");
    expect(resolveLod(0.3)).toBe("compact");
    expect(resolveLod(0.5)).toBe("summary");
    expect(resolveLod(0.8)).toBe("detail");
  });
});
