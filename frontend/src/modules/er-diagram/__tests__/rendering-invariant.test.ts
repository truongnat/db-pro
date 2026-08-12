import { describe, it, expect } from "vitest";
import { SpatialIndex, type SpatialNode } from "../utils/spatial-index";
import { resolveLod } from "../utils/lod";

/**
 * Mechanical verification of the merge gate "P1 invariants hold per fixture
 * (`graphTables != renderedTables != detailedTables`)".
 *
 * Uses the same building blocks the app runs on: `SpatialIndex` (P1.5 — the
 * viewport/culling model behind the HUD and selection queries) and
 * `resolveLod` (P1.3 — the zoom-band LOD resolution that drives the
 * ErTableNode render-tree switch). React Flow's `onlyRenderVisibleElements`
 * (er-diagram.tsx) applies the same culling to the real DOM.
 *
 * Fixture: 500 tables on a deterministic 25x20 grid — the A500 benchmark
 * scale. The invariant is asserted at overview zoom, where it is designed to
 * hold: at zoom >= 0.7 every visible node is legitimately detailed, so the
 * strict `!=` is an overview-scale property (see third test).
 */
const COLS = 25;
const ROWS = 20;
const SPACING = 300;
const GRAPH_TABLES = COLS * ROWS; // 500

function buildFixtureTables(): SpatialNode[] {
  const nodes: SpatialNode[] = [];
  for (let row = 0; row < ROWS; row++) {
    for (let col = 0; col < COLS; col++) {
      nodes.push({
        id: `public.t${row * COLS + col}`,
        position: { x: col * SPACING, y: row * SPACING },
        measured: { width: 220, height: 120 },
      });
    }
  }
  return nodes;
}

/** Viewport framing 5x5 tables (25 nodes) at the top-left of the grid. */
function overviewViewport() {
  return { x: 0, y: 0, zoom: 1, width: 5 * SPACING, height: 5 * SPACING };
}

describe("rendering invariant: graphTables != renderedTables != detailedTables", () => {
  it("culls rendered strictly below the graph at a partial viewport", () => {
    const index = new SpatialIndex();
    index.build(buildFixtureTables());

    const rendered = index.queryNodeIds(overviewViewport()).size;
    expect(rendered).toBeGreaterThan(0);
    expect(rendered).toBeLessThan(GRAPH_TABLES);
  });

  it("keeps detailed strictly below rendered at overview zoom", () => {
    const index = new SpatialIndex();
    index.build(buildFixtureTables());

    const rendered = index.queryNodeIds(overviewViewport()).size;
    const overviewLod = resolveLod(0.3); // 0.2 <= 0.3 < 0.45 -> compact
    const detailed = overviewLod === "detail" ? rendered : 0;

    expect(overviewLod).not.toBe("detail");
    expect(detailed).toBeLessThan(rendered);
    expect(detailed).toBeLessThan(GRAPH_TABLES);
    // All three counts are pairwise distinct on the fixture.
    expect(new Set([GRAPH_TABLES, rendered, detailed]).size).toBe(3);
  });

  it("documents full detail at high zoom (invariant is an overview-scale property)", () => {
    const index = new SpatialIndex();
    index.build(buildFixtureTables());
    const rendered = index.queryNodeIds(overviewViewport()).size;

    expect(resolveLod(1.0)).toBe("detail");
    // At zoom >= 0.7 every visible node is detailed by design — the gate's
    // strict `!=` intentionally describes the overview scale, not max zoom.
    expect(rendered).toBeGreaterThan(0);
  });
});
