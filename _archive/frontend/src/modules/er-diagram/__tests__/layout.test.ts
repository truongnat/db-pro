import { describe, it, expect } from "vitest";
import type { Node, Edge } from "@xyflow/react";
import { layoutGraph } from "../utils/layout";

describe("layoutGraph", () => {
  it("returns nodes with positions", () => {
    const nodes: Node[] = [
      { id: "a", position: { x: 0, y: 0 }, data: { label: "A", columns: [], compact: false } },
      { id: "b", position: { x: 0, y: 0 }, data: { label: "B", columns: [], compact: false } },
    ];
    const edges: Edge[] = [{ id: "e1", source: "a", target: "b" }];

    const result = layoutGraph(nodes, edges);

    expect(result).toHaveLength(2);
    // Nodes should have non-zero positions after layout
    expect(result[0].position).toBeDefined();
    expect(result[1].position).toBeDefined();
    // They should not overlap
    const [n1, n2] = result;
    const sameX = Math.abs(n1.position.x - n2.position.x) < 10;
    const sameY = Math.abs(n1.position.y - n2.position.y) < 10;
    expect(sameX && sameY).toBe(false);
  });

  it("handles empty graph", () => {
    const result = layoutGraph([], []);
    expect(result).toHaveLength(0);
  });

  it("handles single node", () => {
    const nodes: Node[] = [
      {
        id: "solo",
        position: { x: 0, y: 0 },
        data: { label: "Solo", columns: [], compact: false },
      },
    ];
    const result = layoutGraph(nodes, []);
    expect(result).toHaveLength(1);
    expect(result[0].position.x).toBeGreaterThanOrEqual(0);
    expect(result[0].position.y).toBeGreaterThanOrEqual(0);
  });

  it("respects layout direction", () => {
    const nodes: Node[] = [
      { id: "a", position: { x: 0, y: 0 }, data: { label: "A", columns: [], compact: false } },
      { id: "b", position: { x: 0, y: 0 }, data: { label: "B", columns: [], compact: false } },
    ];
    const edges: Edge[] = [{ id: "e1", source: "a", target: "b" }];

    const lr = layoutGraph(nodes, edges, { direction: "LR" });
    const tb = layoutGraph(nodes, edges, { direction: "TB" });

    // LR: b should be to the right of a
    const lrRight =
      lr.find((n) => n.id === "b")!.position.x > lr.find((n) => n.id === "a")!.position.x;
    // TB: b should be below a
    const tbBelow =
      tb.find((n) => n.id === "b")!.position.y > tb.find((n) => n.id === "a")!.position.y;

    expect(lrRight).toBe(true);
    expect(tbBelow).toBe(true);
  });
});
