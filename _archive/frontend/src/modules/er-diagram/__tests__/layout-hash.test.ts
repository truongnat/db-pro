import { describe, it, expect } from "vitest";
import { computeLayoutHash } from "../utils/layout-hash";
import type { LayoutInput } from "../utils/layout";

const input: LayoutInput = {
  nodes: [
    { id: "a", height: 100 },
    { id: "b", height: 120 },
  ],
  edges: [{ source: "a", target: "b" }],
};

describe("computeLayoutHash", () => {
  it("is deterministic for the same input + options", () => {
    expect(computeLayoutHash(input, {})).toBe(computeLayoutHash(input, {}));
    expect(computeLayoutHash(input, { direction: "TB" })).toBe(
      computeLayoutHash(input, { direction: "TB" }),
    );
  });

  it("is independent of the order nodes/edges are listed", () => {
    // Same graph (same edge direction!), different listing order.
    const shuffled: LayoutInput = {
      nodes: [input.nodes[1], input.nodes[0]],
      edges: [input.edges[0]],
    };
    expect(computeLayoutHash(shuffled, {})).toBe(computeLayoutHash(input, {}));
  });

  it("distinguishes edge direction (a->b vs b->a are different graphs)", () => {
    const reversed: LayoutInput = {
      nodes: input.nodes,
      edges: [{ source: "b", target: "a" }],
    };
    expect(computeLayoutHash(reversed, {})).not.toBe(computeLayoutHash(input, {}));
  });

  it("changes when node sizes change", () => {
    const taller: LayoutInput = {
      nodes: [{ id: "a", height: 200 }, input.nodes[1]],
      edges: input.edges,
    };
    expect(computeLayoutHash(taller, {})).not.toBe(computeLayoutHash(input, {}));
  });

  it("changes when topology changes", () => {
    const extraEdge: LayoutInput = {
      nodes: input.nodes,
      edges: [
        { source: "a", target: "b" },
        { source: "b", target: "a" },
      ],
    };
    expect(computeLayoutHash(extraEdge, {})).not.toBe(computeLayoutHash(input, {}));
  });

  it("changes when options change", () => {
    expect(computeLayoutHash(input, { direction: "TB" })).not.toBe(
      computeLayoutHash(input, { direction: "LR" }),
    );
    expect(computeLayoutHash(input, { rankSep: 200 })).not.toBe(
      computeLayoutHash(input, { rankSep: 100 }),
    );
  });

  it("is a stable 64-bit hex string", () => {
    const hash = computeLayoutHash(input, {});
    expect(hash).toMatch(/^[0-9a-f]{16}$/);
  });
});
