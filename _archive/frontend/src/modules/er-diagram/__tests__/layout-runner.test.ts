import { describe, it, expect } from "vitest";
import { createFallbackLayoutRunner } from "../utils/layout-runner";
import {
  computeLayoutPositions,
  layoutGraph,
  layoutNodeHeight,
  type LayoutInput,
} from "../utils/layout";
import type { Node, Edge } from "@xyflow/react";

const input: LayoutInput = {
  nodes: [
    { id: "a", height: layoutNodeHeight(0, false) },
    { id: "b", height: layoutNodeHeight(5, false) },
    { id: "c", height: layoutNodeHeight(2, false) },
  ],
  edges: [
    { source: "a", target: "b" },
    { source: "b", target: "c" },
  ],
};

describe("layoutNodeHeight", () => {
  it("uses compact height when compact", () => {
    expect(layoutNodeHeight(20, true)).toBe(50);
  });

  it("scales with column count when expanded", () => {
    const h0 = layoutNodeHeight(0, false);
    const h10 = layoutNodeHeight(10, false);
    expect(h10 - h0).toBe(10 * 20);
  });
});

describe("computeLayoutPositions", () => {
  it("returns a position for every node id", () => {
    const positions = computeLayoutPositions(input, {});
    expect(positions.size).toBe(3);
    for (const n of input.nodes) expect(positions.has(n.id)).toBe(true);
  });

  it("matches layoutGraph's output for the same input", () => {
    // RF nodes must carry the same column counts the pure input heights encode.
    const colCounts = [0, 5, 2];
    const nodes: Node[] = input.nodes.map((n, i) => ({
      id: n.id,
      position: { x: 0, y: 0 },
      data: {
        label: n.id,
        columns: Array.from({ length: colCounts[i] }, (_, c) => ({ name: "c" + c })),
        compact: false,
      },
    }));
    const edges: Edge[] = input.edges.map((e, i) => ({
      id: `e${i}`,
      source: e.source,
      target: e.target,
    }));

    const fromGraph = layoutGraph(nodes, edges, { direction: "LR" });
    const fromPure = computeLayoutPositions(input, { direction: "LR" });

    for (const n of fromGraph) {
      const expected = fromPure.get(n.id);
      expect(expected).toBeDefined();
      expect(n.position.x).toBeCloseTo(expected!.x, 6);
      expect(n.position.y).toBeCloseTo(expected!.y, 6);
    }
  });
});

describe("createFallbackLayoutRunner", () => {
  it("resolves with deterministic positions matching computeLayoutPositions", async () => {
    const runner = createFallbackLayoutRunner();
    const result = await runner.run({ requestId: 7, input, options: {} });
    expect(result.requestId).toBe(7);
    expect(result.layoutMs).toBeGreaterThanOrEqual(0);

    const positions = computeLayoutPositions(input, {});
    expect(result.positions.a.x).toBeCloseTo(positions.get("a")!.x, 6);
    expect(result.positions.b.y).toBeCloseTo(positions.get("b")!.y, 6);
    expect(result.positions.c).toBeDefined();
  });

  it("handles empty input", async () => {
    const runner = createFallbackLayoutRunner();
    const result = await runner.run({
      requestId: 1,
      input: { nodes: [], edges: [] },
      options: {},
    });
    expect(Object.keys(result.positions)).toHaveLength(0);
  });
});
